use core::borrow::{Borrow, BorrowMut};

use byteorder::{ByteOrder, LittleEndian};
use defmt::*;
use embassy_executor::{Executor, Spawner};
use embassy_futures::join::join;
use embassy_rp::bind_interrupts;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::interrupt;
use embassy_rp::multicore::{spawn_core1, Stack};
use embassy_rp::peripherals::USB;
use embassy_rp::usb::{In, Instance, InterruptHandler, Out};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::channel::{Channel, DynamicReceiver, DynamicSender, Receiver, Sender};
use embassy_time::{Timer, WithTimeout};
use embassy_usb::control::{InResponse, OutResponse, Recipient, Request, RequestType};
use embassy_usb::driver::{Driver, Endpoint, EndpointIn, EndpointOut};
use embassy_usb::msos::{self, windows_version};
use embassy_usb::types::InterfaceNumber;
use embassy_usb::{Builder, Config, Handler};
use export::debug;
use static_cell::StaticCell;

use crate::ftl::buffer::{BufferIdentify, BufferStatus};
use crate::ftl::interface::{FtlReq, FtlReqId, FtlResp, FtlRespStatus};
use crate::resouce::{LedState, CHANNEL_USB_TO_LEDCTRL};
use crate::usb::scsi::*;

// interfaceClass: 0x08 (Mass Storage)
const MSC_INTERFACE_CLASS: u8 = 0x08;
// interfaceSubClass: 0x06 (SCSI Primary Commands)
const MSC_INTERFACE_SUBCLASS: u8 = 0x06;
// interfaceProtocol: 0x50 (Bulk Only Transport)
const MSC_INTERFACE_PROTOCOL: u8 = 0x50;
// CBW dCBWDataTransferLength
const BULK_TRANSFER_MAX_DATA_TRANSFER_LENGTH: usize = 256;

#[repr(u8)]
#[derive(Debug, Copy, Clone, defmt::Format)]
enum ClassSpecificRequest {
    MassStorageReset = 0xff,
    GetMaxLun = 0xfe,
}

/// Bulk Transport command block wrapper
#[repr(u32)]
#[derive(Debug, Copy, Clone, defmt::Format)]
enum BulkTransportSignature {
    CommandBlock = 0x43425355,
    CommandStatus = 0x53425355,
    DataBlock = 0x44425355,
}

/// Bulk Transport command block wrapper packet
#[derive(Debug, Copy, Clone, defmt::Format)]
struct CommandBlockWrapperPacket {
    /// Signature: 0x43425355
    signature: u32,
    /// Tag: Unique identifier for the command block sent by the host
    tag: u32,
    /// Data Transfer Length: Length of the data transfer on the bulk endpoint
    data_transfer_length: u32,
    /// Flags: Bit7=Data In (bulk-in=1, bulk-out=0)
    ///        Bit6: Obsolete (reserved)
    ///        Bit5-0=Reserved
    flags: u8,
    /// LUN: Logical Unit Number
    lun: u8,
    /// Command Length: Length of the command block
    command_length: u8,
    /// Command: SCSI Command Block
    command: [u8; 16],
}

/// Bulk Transport data block wrapper packet
#[repr(u8)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, defmt::Format)]
enum DataDirection {
    HostToDevice,
    DeviceToHost,
}

/// Bulk Transport command status
#[repr(u8)]
#[derive(Debug, Copy, Clone, defmt::Format)]
enum CommandBlockStatus {
    CommandPassed = 0x00,
    CommandFailed = 0x01,
    PhaseError = 0x02,
    Reserved { value: u8 },
}
impl CommandBlockWrapperPacket {
    fn new() -> Self {
        Self {
            signature: BulkTransportSignature::CommandBlock as u32,
            tag: 0,
            data_transfer_length: 0,
            flags: 0,
            lun: 0,
            command_length: 0,
            command: [0; 16],
        }
    }

    /// Convert to byte array
    fn from_data(data: &[u8]) -> Option<Self> {
        // Check if the data length is valid
        if data.len() < 31 {
            return None;
        }
        // Parse the data
        let packet_data = CommandBlockWrapperPacket {
            signature: LittleEndian::read_u32(&data[0..4]),
            tag: LittleEndian::read_u32(&data[4..8]),
            data_transfer_length: LittleEndian::read_u32(&data[8..12]),
            flags: data[12],
            lun: data[13],
            command_length: data[14],
            command: data[15..31].try_into().unwrap(),
        };
        Some(packet_data)
    }

    fn is_valid_signature(&self) -> bool {
        self.signature == (BulkTransportSignature::CommandBlock as u32)
    }

    /// Convert to byte array
    fn to_data(self) -> [u8; 31] {
        let mut data = [0; 31];
        LittleEndian::write_u32(&mut data[0..4], self.signature);
        LittleEndian::write_u32(&mut data[4..8], self.tag);
        LittleEndian::write_u32(&mut data[8..12], self.data_transfer_length);
        data[12] = self.flags;
        data[13] = self.lun;
        data[14] = self.command_length;
        data[15..31].copy_from_slice(&self.command);
        data
    }

    /// CBWFlags Bit7:
    ///  0: Data Out (Host->Device)
    /// 1: Data In (Device->Host)
    fn data_direction(&self) -> DataDirection {
        if self.flags & 0x80 == 0 {
            DataDirection::HostToDevice
        } else {
            DataDirection::DeviceToHost
        }
    }

    /// get command block
    fn get_commands(&self) -> &[u8] {
        &self.command[..self.command_length as usize]
    }
}

/// Bulk Transport command status wrapper packet
#[derive(Debug, Copy, Clone, defmt::Format)]
struct CommandStatusWrapperPacket {
    /// Signature: 0x53425355
    signature: u32,
    /// Tag: Unique identifier for the command block sent by the host
    tag: u32,
    /// Data Residue: Amount of data not transferred
    data_residue: u32,
    /// Status: Command status
    status: CommandBlockStatus,
}

impl CommandStatusWrapperPacket {
    fn new() -> Self {
        Self {
            signature: BulkTransportSignature::CommandStatus as u32,
            tag: 0,
            data_residue: 0,
            status: CommandBlockStatus::CommandPassed,
        }
    }

    /// Convert to byte array
    fn from_data(data: &[u8]) -> Option<Self> {
        // Check if the data length is valid
        if data.len() < 13 {
            return None;
        }
        // Parse the data
        let packet_data = CommandStatusWrapperPacket {
            signature: LittleEndian::read_u32(&data[0..4]),
            tag: LittleEndian::read_u32(&data[4..8]),
            data_residue: LittleEndian::read_u32(&data[8..12]),
            status: match data[12] {
                0x00 => CommandBlockStatus::CommandPassed,
                0x01 => CommandBlockStatus::CommandFailed,
                0x02 => CommandBlockStatus::PhaseError,
                _ => CommandBlockStatus::Reserved { value: data[12] },
            },
        };
        Some(packet_data)
    }

    /// Convert to byte array
    fn to_data(self) -> [u8; 13] {
        let mut data = [0; 13];
        LittleEndian::write_u32(&mut data[0..4], self.signature);
        LittleEndian::write_u32(&mut data[4..8], self.tag);
        LittleEndian::write_u32(&mut data[8..12], self.data_residue);
        data[12] = match self.status {
            CommandBlockStatus::CommandPassed => 0x00,
            CommandBlockStatus::CommandFailed => 0x01,
            CommandBlockStatus::PhaseError => 0x02,
            CommandBlockStatus::Reserved { value } => value,
        };
        data
    }

    /// Check if the signature is valid
    fn is_valid_signature(&self) -> bool {
        self.signature == (BulkTransportSignature::CommandStatus as u32)
    }
}

/// USB Bulk Transfer Request
/// This enum is used to send requests to the USB Bulk Transfer Handler.
#[derive(Debug, Copy, Clone, PartialEq, Eq, defmt::Format)]
pub enum BulkTransferRequest {
    Reset,
}

/// USB Mass Storage Class Control Handler
/// This handler is used to handle the control requests for the Mass Storage Class.
/// It supports the Mass Storage Reset and Get Max LUN requests.
pub struct MscCtrlHandler<'d> {
    /// Interface Number
    if_num: InterfaceNumber,
    /// Bulk Transfer Request Sender (for Mass Storage Reset)
    bulk_request_sender: DynamicSender<'d, BulkTransferRequest>,
}

/// USB Mass Storage Class Bulk Handler Configuration
pub struct MscBulkHandlerConfig {
    pub vendor_id: [u8; 8],
    pub product_id: [u8; 16],
    pub product_revision_level: [u8; 4],
    pub num_blocks: u32,
    pub block_size: u32,
}

/// USB Mass Storage Class Bulk Handler
/// This handler is used to handle the bulk transfers for the Mass Storage Class.
pub struct MscBulkHandler<'d, D: Driver<'d>> {
    /// Bulk Transfer Request Receiver (for Mass Storage Reset)
    bulk_request_receiver: DynamicReceiver<'d, BulkTransferRequest>,
    /// Bulk Endpoint Out
    read_ep: Option<<D as Driver<'d>>::EndpointOut>,
    /// Bulk Endpoint In
    write_ep: Option<<D as Driver<'d>>::EndpointIn>,

    /// Config
    config: MscBulkHandlerConfig,

    /// Request Read/Write to NAND Flash
    internal_request_sender: DynamicSender<'d, FtlReq>,
    /// Response Read/Write to NAND Flash
    internal_request_receiver: DynamicReceiver<'d, FtlResp>,
}

impl<'d> Handler for MscCtrlHandler<'d> {
    fn control_out<'a>(&'a mut self, req: Request, buf: &'a [u8]) -> Option<OutResponse> {
        debug!("Got control_out, request={}, buf={:a}", req, buf);
        None
    }

    /// Respond to DeviceToHost control messages, where the host requests some data from us.
    fn control_in<'a>(&'a mut self, req: Request, buf: &'a mut [u8]) -> Option<InResponse<'a>> {
        debug!("Got control_in, request={}", req);

        // requestType: Class/Interface, host->device
        // request: 0xff (Mass Storage Reset), 0xfe (Get Max LUN)

        if req.request_type != RequestType::Class || req.recipient != Recipient::Interface {
            return None;
        }
        match req.request {
            x if x == ClassSpecificRequest::MassStorageReset as u8 => {
                // Mass Storage Reset
                debug!("Mass Storage Reset");
                match self
                    .bulk_request_sender
                    .try_send(BulkTransferRequest::Reset)
                {
                    Ok(_) => Some(InResponse::Accepted(&buf[..0])),
                    Err(_) => Some(InResponse::Rejected),
                }
            }
            x if x == ClassSpecificRequest::GetMaxLun as u8 && req.length == 1 => {
                // Get Max LUN
                debug!("Get Max LUN");
                buf[0] = 0; // Only one LUN supported
                Some(InResponse::Accepted(&buf[..1]))
            }
            _ => {
                warn!("Unsupported request: {}", req.request);
                Some(InResponse::Rejected)
            }
        }
    }
}

impl<'d> MscCtrlHandler<'d> {
    pub fn new(bulk_request_sender: DynamicSender<'d, BulkTransferRequest>) -> Self {
        Self {
            if_num: InterfaceNumber(0),
            bulk_request_sender,
        }
    }

    pub fn build<'a, D: Driver<'d>>(
        &'d mut self,
        builder: &mut Builder<'d, D>,
        config: Config<'d>,
        bulk_handler: &'a mut MscBulkHandler<'d, D>,
    ) {
        // Bulk Only Transport for Mass Storage
        let mut function = builder.function(
            MSC_INTERFACE_CLASS,
            MSC_INTERFACE_SUBCLASS,
            MSC_INTERFACE_PROTOCOL,
        );
        let mut interface = function.interface();
        let mut alt = interface.alt_setting(
            MSC_INTERFACE_CLASS,
            MSC_INTERFACE_SUBCLASS,
            MSC_INTERFACE_PROTOCOL,
            None,
        );
        bulk_handler.read_ep = Some(alt.endpoint_bulk_out(64));
        bulk_handler.write_ep = Some(alt.endpoint_bulk_in(64));

        drop(function);
        builder.handler(self);
    }
}

impl MscBulkHandlerConfig {
    pub fn new(
        vendor_id: [u8; 8],
        product_id: [u8; 16],
        product_revision_level: [u8; 4],
        num_blocks: u32,
        block_size: u32,
    ) -> Self {
        Self {
            vendor_id,
            product_id,
            product_revision_level,
            num_blocks,
            block_size,
        }
    }
}

impl<'d, D: Driver<'d>> MscBulkHandler<'d, D> {
    pub fn new(
        config: MscBulkHandlerConfig,
        bulk_request_receiver: DynamicReceiver<'d, BulkTransferRequest>,
        internal_request_sender: DynamicSender<'d, FtlReq>,
        internal_request_receiver: DynamicReceiver<'d, FtlResp>,
    ) -> Self {
        Self {
            read_ep: None,
            write_ep: None,
            config,
            bulk_request_receiver,
            internal_request_sender,
            internal_request_receiver,
        }
    }

    /// Main loop for bulk-only transport
    /// TODO: 関数肥大化しているので、分割する
    pub async fn run(&mut self) -> ! {
        crate::assert!(self.read_ep.is_some());
        crate::assert!(self.write_ep.is_some());
        let read_ep = self.read_ep.as_mut().unwrap();
        let write_ep = self.write_ep.as_mut().unwrap();
        'main_loop: loop {
            // EndPoint有効待ち
            read_ep.wait_enabled().await;
            debug!("Connected");

            // Request Sense CommandでError reportingが必要なので、前回の情報を保持しておく
            let mut latest_sense_data: Option<RequestSenseData> = None;
            // Phase Error時の対応用
            let mut phase_error_tag: Option<u32> = None;

            'read_ep_loop: loop {
                // Check if Mass Storage Reset occurred
                if (self.bulk_request_receiver.try_receive() == Ok(BulkTransferRequest::Reset)) {
                    debug!("Mass Storage Reset");
                    phase_error_tag = None;
                    break 'read_ep_loop;
                }

                // Command Transport
                let mut read_buf = [0u8; BULK_TRANSFER_MAX_DATA_TRANSFER_LENGTH];
                let Ok(read_cbw_size) = read_ep.read(&mut read_buf).await else {
                    error!("Read EP Error (CBW)");
                    phase_error_tag = None; // unknown tag
                    break 'read_ep_loop;
                };
                let Some(cbw_packet) = CommandBlockWrapperPacket::from_data(&read_buf) else {
                    error!("Invalid CBW: {:#x}", read_buf);
                    phase_error_tag = None; // unknown tag
                    break 'read_ep_loop;
                };
                if !cbw_packet.is_valid_signature() {
                    error!("Invalid CBW signature: {:#x}", cbw_packet);
                    phase_error_tag = None; // unknown tag
                    break 'read_ep_loop;
                };
                if cbw_packet.command_length == 0 {
                    error!("Invalid CBW command length: {:#x}", cbw_packet);
                    phase_error_tag = None; // unknown tag
                    break 'read_ep_loop;
                };
                debug!("Got CBW: {:#x}", cbw_packet);

                // Prepare CSW
                let mut csw_packet = CommandStatusWrapperPacket::new();
                csw_packet.tag = cbw_packet.tag;
                csw_packet.data_residue = 0;
                csw_packet.status = CommandBlockStatus::CommandPassed;

                // HostToDeviceの場合、PhaseError対策に先に読んでデータを保持しておく
                if cbw_packet.data_direction() == DataDirection::HostToDevice {
                    let Ok(read_data_size) = read_ep.read(&mut read_buf).await else {
                        phase_error_tag = Some(cbw_packet.tag);
                        break 'read_ep_loop;
                    };
                }
                // DeviceToHostの場合の書くためのバッファ
                let mut write_buf = [0u8; BULK_TRANSFER_MAX_DATA_TRANSFER_LENGTH];
                let request_write_len = cbw_packet.data_transfer_length as usize;
                let mut actual_write_len = 0usize;

                // Parse SCSI Command
                let scsi_commands = cbw_packet.get_commands();
                let scsi_command = scsi_commands[0];
                match scsi_command {
                    x if x == ScsiCommand::TestUnitReady as u8 => {
                        debug!("Test Unit Ready");
                        // カードの抜き差しなどはないので問題無しで応答
                        csw_packet.status = CommandBlockStatus::CommandPassed;
                    }
                    x if x == ScsiCommand::Inquiry as u8 => {
                        debug!("Inquiry");
                        // Inquiry data. resp fixed data
                        actual_write_len = INQUIRY_COMMAND_DATA_SIZE;
                        let inquiry_data = InquiryCommandData::new(
                            self.config.vendor_id,
                            self.config.product_id,
                            self.config.product_revision_level,
                        );
                        inquiry_data.prepare_to_buf(&mut write_buf[0..actual_write_len]);
                    }
                    x if x == ScsiCommand::ReadFormatCapacities as u8 => {
                        debug!("Read Format Capacities");
                        // Read Format Capacities data. resp fixed data
                        actual_write_len = READ_FORMAT_CAPACITIES_DATA_SIZE;
                        let read_format_capacities_data = ReadFormatCapacitiesData::new(
                            self.config.num_blocks,
                            self.config.block_size,
                        );
                        read_format_capacities_data
                            .prepare_to_buf(&mut write_buf[0..actual_write_len]);
                    }
                    x if x == ScsiCommand::ReadCapacity as u8 => {
                        debug!("Read Capacity");
                        // Read Capacity data. resp fixed data
                        actual_write_len = READ_CAPACITY_16_DATA_SIZE;
                        let read_capacity_data =
                            ReadCapacityData::new(self.config.num_blocks, self.config.block_size);
                        read_capacity_data.prepare_to_buf(&mut write_buf[0..actual_write_len]);
                    }
                    x if x == ScsiCommand::ModeSense6 as u8 => {
                        debug!("Mode Sense 6");
                        // Mode Sense 6 data. resp fixed data
                        actual_write_len = MODE_SENSE_6_DATA_SIZE;
                        let mode_sense_data = ModeSense6Data::new();
                        mode_sense_data.prepare_to_buf(&mut write_buf[0..actual_write_len]);
                    }
                    x if x == ScsiCommand::RequestSense as u8 => {
                        debug!("Request Sense");
                        // Error reporting
                        actual_write_len = REQUEST_SENSE_DATA_SIZE;
                        if latest_sense_data.is_none() {
                            latest_sense_data = Some(RequestSenseData::from(
                                SenseKey::NoSense,
                                AdditionalSenseCodeType::NoAdditionalSenseInformation,
                            ));
                        }
                        latest_sense_data
                            .unwrap()
                            .prepare_to_buf(&mut write_buf[0..actual_write_len]);
                        latest_sense_data = None;
                    }
                    // x if x == ScsiCommand::Read10 as u8 => {
                    //     debug!("Read 10");
                    //     if cbw_packet.command_length < READ_10_DATA_SIZE as u8 {
                    //         error!("Invalid Read 10 Command Length: {:#x}", cbw_packet);
                    //         latest_sense_data = Some(RequestSenseData::from(
                    //             SenseKey::IllegalRequest,
                    //             AdditionalSenseCodeType::IllegalRequestParameterLengthError,
                    //         ));
                    //         csw_packet.status = CommandBlockStatus::CommandFailed;
                    //     } else {
                    //         // Parse Cmd
                    //         let cmd = Read10Command::from_data(scsi_commands);
                    //         let lba = cmd.lba;
                    //         let transfer_len = cmd.transfer_length;
                    //         // TODO: Read Request積むのと、Read Response待つのを同時にやる
                    //         for transfer_count in 0..transfer_len {
                    //             // TODO: Buffer管理機構からBufferを借用して、ここでは使わない
                    //             let mut read_data = [0u8; 512];
                    //             self.internal_request_sender
                    //                 .send(InternalTransferRequest::new(
                    //                     InternalTransferRequestId::Read,
                    //                     cbw_packet.tag,
                    //                     Some(DataBufferIdentify { tag: 0 }),
                    //                 ))
                    //                 .await;
                    //             let response = self.internal_request_receiver.receive().await;
                    //             // Check response
                    //             if response.resp_status != InternalTransferResponseStatus::Success {
                    //                 error!("Read Error: {:?}", response);
                    //                 latest_sense_data = Some(RequestSenseData::from(
                    //                     SenseKey::HardwareError,
                    //                     AdditionalSenseCodeType::HardwareErrorGeneral,
                    //                 ));
                    //                 csw_packet.status = CommandBlockStatus::CommandFailed;
                    //                 break;
                    //             }
                    //             // TODO: Copy data
                    //             // Hostにデータを書き込む
                    //             let Ok(_) = write_ep.write(&read_data).await else {
                    //                 phase_error_tag = Some(cbw_packet.tag);
                    //                 break 'read_ep_loop;
                    //             };
                    //         }
                    //     }
                    // }
                    _ => {
                        error!("Unsupported Command: {:#x}", scsi_command);
                        // save latest sense data
                        latest_sense_data = Some(RequestSenseData::from(
                            SenseKey::IllegalRequest,
                            AdditionalSenseCodeType::IllegalRequestInvalidCommand,
                        ));

                        actual_write_len = 0;
                        csw_packet.status = CommandBlockStatus::CommandFailed;
                    }
                }

                // Data Transport (DeviceToHost)
                if actual_write_len > 0 {
                    // transfer data
                    debug!("Write Data: {:#x}", write_buf[0..actual_write_len]);
                    let Ok(_) = write_ep.write(&write_buf[0..actual_write_len]).await else {
                        phase_error_tag = Some(cbw_packet.tag);
                        break 'read_ep_loop;
                    };
                    // update csw_packet
                    csw_packet.status = CommandBlockStatus::CommandPassed;
                    if actual_write_len < request_write_len {
                        csw_packet.data_residue = (request_write_len - actual_write_len) as u32;
                    }
                }

                // Status Transport
                let csw_data = csw_packet.to_data();
                debug!("Send CSW: {:#x}", csw_packet);
                let Ok(_) = write_ep.write(&csw_data).await else {
                    error!("Write EP Error");
                    break 'read_ep_loop;
                };

                // ループ内の処理をやりきれるケースはPhaseErrorが発生していないので、tagをクリア
                phase_error_tag = None;
            }

            if let Some(tag) = phase_error_tag {
                error!("Phase Error");
                // CSW で Phase Error を返す
                let mut csw_packet = CommandStatusWrapperPacket::new();
                csw_packet.tag = tag;
                csw_packet.data_residue = 0;
                csw_packet.status = CommandBlockStatus::PhaseError;
                let csw_data = csw_packet.to_data();
                // 失敗してもハンドリング無理
                write_ep.write(&csw_data).await;
            }
            debug!("Disconnected");
        }
    }
}
