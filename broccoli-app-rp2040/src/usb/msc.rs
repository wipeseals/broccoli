use core::borrow::{Borrow, BorrowMut};
use core::f32::consts::E;
use core::fmt::Error;
use core::mem::drop;
use core::option::{
    Option,
    Option::{None, Some},
};
use core::result::{
    Result,
    Result::{Err, Ok},
};

use byteorder::{ByteOrder, LittleEndian};
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
use embassy_usb::driver::{Driver, Endpoint, EndpointError, EndpointIn, EndpointOut};
use embassy_usb::msos::{self, windows_version};
use embassy_usb::types::InterfaceNumber;
use embassy_usb::{Builder, Config, Handler};
use static_cell::StaticCell;

use crate::shared::constant::*;
use crate::shared::datatype::MscReqTag;
use crate::usb::scsi::*;
use broccoli_core::common::storage_req::{StorageMsgId, StorageRequest, StorageResponse};

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

impl CommandBlockStatus {
    fn from_u8(value: u8) -> Self {
        match value {
            0x00 => CommandBlockStatus::CommandPassed,
            0x01 => CommandBlockStatus::CommandFailed,
            0x02 => CommandBlockStatus::PhaseError,
            _ => CommandBlockStatus::Reserved { value },
        }
    }

    fn from_bool(value: bool) -> Self {
        if value {
            CommandBlockStatus::CommandPassed
        } else {
            CommandBlockStatus::CommandFailed
        }
    }
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
pub struct MscCtrlHandler<'ch> {
    /// Interface Number
    if_num: InterfaceNumber,
    /// Bulk Transfer Request Sender (for Mass Storage Reset)
    bulk_request_sender: DynamicSender<'ch, BulkTransferRequest>,
}

/// USB Mass Storage Class Bulk Handler Configuration
pub struct MscBulkHandlerConfig {
    pub vendor_id: [u8; 8],
    pub product_id: [u8; 16],
    pub product_revision_level: [u8; 4],
    pub num_blocks: usize,
    pub block_size: usize,
}

/// USB Mass Storage Class Bulk Handler
/// This handler is used to handle the bulk transfers for the Mass Storage Class.
pub struct MscBulkHandler<'driver, 'ch, D: Driver<'driver>> {
    /// Bulk Transfer Request Receiver (for Mass Storage Reset)
    ctrl_to_bulk_request_receiver: DynamicReceiver<'ch, BulkTransferRequest>,
    /// Bulk Endpoint Out
    read_ep: Option<<D as Driver<'driver>>::EndpointOut>,
    /// Bulk Endpoint In
    write_ep: Option<<D as Driver<'driver>>::EndpointIn>,

    /// Config
    config: MscBulkHandlerConfig,

    /// Request Read/Write to Flash Translation Layer
    storage_req_sender: DynamicSender<'ch, StorageRequest<MscReqTag, USB_LOGICAL_BLOCK_SIZE>>,
    /// Response Read/Write from Flash Translation Layer
    storage_resp_receiver: DynamicReceiver<'ch, StorageResponse<MscReqTag, USB_LOGICAL_BLOCK_SIZE>>,
}

impl<'ch> Handler for MscCtrlHandler<'ch> {
    fn control_out<'a>(&'a mut self, req: Request, buf: &'a [u8]) -> Option<OutResponse> {
        crate::trace!("Got control_out, request={}, buf={:a}", req, buf);
        None
    }

    /// Respond to DeviceToHost control messages, where the host requests some data from us.
    fn control_in<'a>(&'a mut self, req: Request, buf: &'a mut [u8]) -> Option<InResponse<'a>> {
        crate::trace!("Got control_in, request={}", req);

        // requestType: Class/Interface, host->device
        // request: 0xff (Mass Storage Reset), 0xfe (Get Max LUN)

        if req.request_type != RequestType::Class || req.recipient != Recipient::Interface {
            return None;
        }
        match req.request {
            x if x == ClassSpecificRequest::MassStorageReset as u8 => {
                // Mass Storage Reset
                crate::trace!("Mass Storage Reset");
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
                crate::trace!("Get Max LUN");
                buf[0] = 0; // Only one LUN supported
                Some(InResponse::Accepted(&buf[..1]))
            }
            _ => {
                crate::warn!("Unsupported request: {}", req.request);
                Some(InResponse::Rejected)
            }
        }
    }
}

impl<'ch> MscCtrlHandler<'ch> {
    pub fn new(bulk_request_sender: DynamicSender<'ch, BulkTransferRequest>) -> Self {
        Self {
            if_num: InterfaceNumber(0),
            bulk_request_sender,
        }
    }

    pub fn build<'a, 'driver, D: Driver<'driver>>(
        &'ch mut self,
        builder: &mut Builder<'driver, D>,
        config: Config<'ch>,
        bulk_handler: &'a mut MscBulkHandler<'driver, 'ch, D>,
    ) where
        'ch: 'driver,
    {
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
        num_blocks: usize,
        block_size: usize,
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

impl<'driver, 'ch, D: Driver<'driver>> MscBulkHandler<'driver, 'ch, D> {
    pub fn new(
        config: MscBulkHandlerConfig,
        ctrl_to_bulk_request_receiver: DynamicReceiver<'ch, BulkTransferRequest>,
        storage_req_sender: DynamicSender<'ch, StorageRequest<MscReqTag, USB_LOGICAL_BLOCK_SIZE>>,
        storage_resp_receiver: DynamicReceiver<
            'ch,
            StorageResponse<MscReqTag, USB_LOGICAL_BLOCK_SIZE>,
        >,
    ) -> Self {
        Self {
            read_ep: None,
            write_ep: None,
            config,
            ctrl_to_bulk_request_receiver,
            storage_req_sender,
            storage_resp_receiver,
        }
    }

    /// Handle response for simple command
    async fn handle_response_single<'a>(
        write_ep: &'a mut <D as Driver<'driver>>::EndpointIn,
        status: CommandBlockStatus,
        write_data: Option<&'a [u8]>,
        cbw_packet: &'a CommandBlockWrapperPacket,
        csw_packet: &'a mut CommandStatusWrapperPacket,
    ) -> Result<(), EndpointError> {
        if let Some(data) = write_data {
            // transfer data
            write_ep.write(data).await?;
            // update csw_packet.data_residue
            if data.len() < cbw_packet.data_transfer_length as usize {
                csw_packet.data_residue =
                    (cbw_packet.data_transfer_length as usize - data.len()) as u32;
            }
        }
        // update csw_packet
        csw_packet.status = status;

        // Status Transport
        let csw_data = csw_packet.to_data();
        crate::trace!("Send CSW: {:#x}", csw_packet);
        write_ep.write(&csw_data).await?;

        Ok(())
    }

    /// Main loop for bulk-only transport
    pub async fn run(&mut self) -> ! {
        crate::assert!(self.read_ep.is_some());
        crate::assert!(self.write_ep.is_some());
        let read_ep = self.read_ep.as_mut().unwrap();
        let write_ep = self.write_ep.as_mut().unwrap();
        'main_loop: loop {
            // EndPoint有効待ち
            read_ep.wait_enabled().await;
            crate::trace!("Connected");

            // Request Sense CommandでError reportingが必要なので、前回の情報を保持しておく
            let mut latest_sense_data: Option<RequestSenseData> = None;
            // Phase Error時の対応用
            let mut phase_error_tag: Option<u32> = None;

            'read_ep_loop: loop {
                // Check if Mass Storage Reset occurred
                if (self.ctrl_to_bulk_request_receiver.try_receive()
                    == Ok(BulkTransferRequest::Reset))
                {
                    crate::trace!("Mass Storage Reset");
                    phase_error_tag = None;
                    break 'read_ep_loop;
                }

                // clear latest sense data
                latest_sense_data = None;

                // Command Transport
                let mut read_buf = [0u8; USB_LOGICAL_BLOCK_SIZE]; // read buffer分確保
                let Ok(read_cbw_size) = read_ep.read(&mut read_buf).await else {
                    crate::error!("Read EP Error (CBW)");
                    phase_error_tag = None; // unknown tag
                    latest_sense_data = Some(RequestSenseData::from(
                        SenseKey::IllegalRequest,
                        AdditionalSenseCodeType::IllegalRequestInvalidCommand,
                    ));
                    break 'read_ep_loop;
                };
                let Some(cbw_packet) = CommandBlockWrapperPacket::from_data(&read_buf) else {
                    crate::error!("Invalid CBW: {:#x}", read_buf);
                    phase_error_tag = None; // unknown tag
                    latest_sense_data = Some(RequestSenseData::from(
                        SenseKey::IllegalRequest,
                        AdditionalSenseCodeType::IllegalRequestInvalidCommand,
                    ));
                    break 'read_ep_loop;
                };
                if !cbw_packet.is_valid_signature() {
                    crate::error!("Invalid CBW signature: {:#x}", cbw_packet);
                    phase_error_tag = None; // unknown tag
                    latest_sense_data = Some(RequestSenseData::from(
                        SenseKey::IllegalRequest,
                        AdditionalSenseCodeType::IllegalRequestInParameters,
                    ));
                    break 'read_ep_loop;
                };
                if cbw_packet.command_length == 0 {
                    crate::error!("Invalid CBW command length: {:#x}", cbw_packet);
                    phase_error_tag = None; // unknown tag
                    latest_sense_data = Some(RequestSenseData::from(
                        SenseKey::IllegalRequest,
                        AdditionalSenseCodeType::IllegalRequestInParameters,
                    ));
                    break 'read_ep_loop;
                };

                // Prepare CSW
                let mut csw_packet = CommandStatusWrapperPacket::new();
                csw_packet.tag = cbw_packet.tag;
                csw_packet.data_residue = 0;
                csw_packet.status = CommandBlockStatus::CommandPassed;

                // Parse SCSI Command
                let scsi_commands = cbw_packet.get_commands();
                let scsi_command = scsi_commands[0];
                // コマンドごとに処理
                let send_resp_status: Result<(), EndpointError> = match scsi_command {
                    x if x == ScsiCommand::TestUnitReady as u8 => {
                        crate::trace!("Test Unit Ready");
                        // カードの抜き差しなどはないので問題無しで応答
                        Self::handle_response_single(
                            write_ep,
                            CommandBlockStatus::CommandPassed,
                            None,
                            &cbw_packet,
                            &mut csw_packet,
                        )
                        .await
                    }
                    x if x == ScsiCommand::Inquiry as u8 => {
                        crate::trace!("Inquiry");
                        // Inquiry data. resp fixed data
                        let inquiry_data = InquiryCommandData::new(
                            self.config.vendor_id,
                            self.config.product_id,
                            self.config.product_revision_level,
                        );

                        let mut write_data = [0u8; INQUIRY_COMMAND_DATA_SIZE];
                        inquiry_data.prepare_to_buf(&mut write_data);
                        Self::handle_response_single(
                            write_ep,
                            CommandBlockStatus::CommandPassed,
                            Some(&write_data),
                            &cbw_packet,
                            &mut csw_packet,
                        )
                        .await
                    }
                    x if x == ScsiCommand::ReadFormatCapacities as u8 => {
                        crate::trace!("Read Format Capacities");
                        // Read Format Capacities data. resp fixed data
                        let read_format_capacities_data = ReadFormatCapacitiesData::new(
                            self.config.num_blocks as u32,
                            self.config.block_size as u32,
                        );

                        let mut write_data = [0u8; READ_FORMAT_CAPACITIES_DATA_SIZE];
                        read_format_capacities_data.prepare_to_buf(&mut write_data);
                        Self::handle_response_single(
                            write_ep,
                            CommandBlockStatus::CommandPassed,
                            Some(&write_data),
                            &cbw_packet,
                            &mut csw_packet,
                        )
                        .await
                    }
                    x if x == ScsiCommand::ReadCapacity as u8 => {
                        crate::trace!("Read Capacity");
                        // Read Capacity data. resp fixed data
                        let read_capacity_data = ReadCapacityData::new(
                            (self.config.num_blocks - 1) as u32,
                            self.config.block_size as u32,
                        );

                        let mut write_data = [0u8; READ_CAPACITY_16_DATA_SIZE];
                        read_capacity_data.prepare_to_buf(&mut write_data);
                        Self::handle_response_single(
                            write_ep,
                            CommandBlockStatus::CommandPassed,
                            Some(&write_data),
                            &cbw_packet,
                            &mut csw_packet,
                        )
                        .await
                    }
                    x if x == ScsiCommand::ModeSense6 as u8 => {
                        crate::trace!("Mode Sense 6");
                        // Mode Sense 6 data. resp fixed data
                        let mode_sense_data = ModeSense6Data::new();

                        let mut write_data = [0u8; MODE_SENSE_6_DATA_SIZE];
                        mode_sense_data.prepare_to_buf(&mut write_data);
                        Self::handle_response_single(
                            write_ep,
                            CommandBlockStatus::CommandPassed,
                            Some(&write_data),
                            &cbw_packet,
                            &mut csw_packet,
                        )
                        .await
                    }
                    x if x == ScsiCommand::RequestSense as u8 => {
                        // Error reporting
                        if latest_sense_data.is_none() {
                            latest_sense_data = Some(RequestSenseData::from(
                                SenseKey::NoSense,
                                AdditionalSenseCodeType::NoAdditionalSenseInformation,
                            ));
                        }
                        crate::trace!("Request Sense Data: {:#x}", latest_sense_data.unwrap());

                        let mut write_data = [0u8; REQUEST_SENSE_DATA_SIZE];
                        latest_sense_data.unwrap().prepare_to_buf(&mut write_data);
                        Self::handle_response_single(
                            write_ep,
                            CommandBlockStatus::CommandPassed,
                            Some(&write_data),
                            &cbw_packet,
                            &mut csw_packet,
                        )
                        .await
                    }
                    x if x == ScsiCommand::Read10 as u8 => {
                        // Read 10 data. resp variable data
                        let read10_data = Read10Command::from_data(scsi_commands);
                        crate::trace!("Read 10 Data: {:#x}", read10_data);
                        let transfer_length = read10_data.transfer_length as usize;

                        // TODO: channelに空きがある場合transfer_length分のRequest投げるTaskと、Responseを受け取るTaskのjoinにする
                        for transfer_index in 0..transfer_length {
                            let lba = read10_data.lba as usize + transfer_index;
                            let req_tag = MscReqTag::new(cbw_packet.tag, transfer_index as u32);
                            let req = StorageRequest::read(req_tag, lba);

                            self.storage_req_sender.send(req).await;
                            let resp = self.storage_resp_receiver.receive().await;

                            // Read処理中にRead以外の応答が来た場合は実装不具合
                            if resp.message_id != StorageMsgId::Read {
                                crate::unreachable!("Invalid Response: {:#x}", resp);
                            }
                            // Check if the response is valid
                            if (req_tag != resp.req_tag) {
                                crate::error!("Invalid Response: {:#x}", resp);
                                latest_sense_data = Some(RequestSenseData::from(
                                    SenseKey::HardwareError,
                                    AdditionalSenseCodeType::HardwareErrorEmbeddedSoftware,
                                ));
                            }
                            // Check if there is an error
                            if let Some(error) = resp.meta_data {
                                crate::error!("Invalid Response: {:#x}", resp);
                                latest_sense_data =
                                    Some(RequestSenseData::from_data_request_error(error));
                            }

                            // transfer read data
                            let read_data = resp.data.as_ref();
                            for packet_i in 0..USB_PACKET_COUNT_PER_LOGICAL_BLOCK {
                                let start_index = (packet_i * USB_MAX_PACKET_SIZE);
                                let end_index = ((packet_i + 1) * USB_MAX_PACKET_SIZE);
                                // 範囲がUSB_BLOCK_SIZEを超えないように修正
                                let end_index = end_index.min(USB_LOGICAL_BLOCK_SIZE);

                                // データを取り出して応答
                                let packet_data = &read_data[start_index..end_index];
                                crate::trace!(
                                    "Send Read Data (LBA: {:#x}, TransferIndex: {:#x}, PacketIndex: {:#x}): {:#x}",
                                    lba, transfer_index, packet_i, packet_data
                                );
                                let Ok(write_resp) = write_ep.write(packet_data).await else {
                                    crate::error!("Write EP Error (Read 10)");
                                    phase_error_tag = Some(cbw_packet.tag);
                                    latest_sense_data = Some(RequestSenseData::from(
                                        SenseKey::IllegalRequest,
                                        AdditionalSenseCodeType::IllegalRequestInvalidCommand,
                                    ));
                                    break 'read_ep_loop;
                                };
                            }
                        }

                        // CSW 応答
                        csw_packet.status =
                            CommandBlockStatus::from_bool(latest_sense_data.is_none());
                        let transfer_bytes = transfer_length * self.config.block_size;
                        if transfer_bytes < cbw_packet.data_transfer_length as usize {
                            csw_packet.data_residue =
                                (cbw_packet.data_transfer_length as usize - transfer_bytes) as u32;
                        }
                        let csw_data = csw_packet.to_data();
                        crate::trace!("Send CSW: {:#x}", csw_packet);
                        write_ep.write(&csw_data).await
                    }
                    x if x == ScsiCommand::Write10 as u8 => {
                        // Write 10 data. resp variable data
                        let write10_data = Write10Command::from_data(scsi_commands);
                        crate::trace!("Write 10 Data: {:#x}", write10_data);
                        let transfer_length = write10_data.transfer_length as usize;

                        for transfer_index in 0..transfer_length {
                            let lba = write10_data.lba as usize + transfer_index;
                            // packet size分のデータを受け取る
                            let req_tag = MscReqTag::new(cbw_packet.tag, transfer_index as u32);
                            let mut req =
                                StorageRequest::write(req_tag, lba, [0u8; USB_LOGICAL_BLOCK_SIZE]);
                            for packet_i in 0..USB_PACKET_COUNT_PER_LOGICAL_BLOCK {
                                let start_index = (packet_i * USB_MAX_PACKET_SIZE);
                                let end_index = ((packet_i + 1) * USB_MAX_PACKET_SIZE);
                                // 範囲がUSB_BLOCK_SIZEを超えないように修正
                                let end_index = end_index.min(USB_LOGICAL_BLOCK_SIZE);

                                // データを受け取る
                                let Ok(read_resp) =
                                    read_ep.read(&mut req.data[start_index..end_index]).await
                                else {
                                    crate::error!("Read EP Error (Write 10)");
                                    phase_error_tag = Some(cbw_packet.tag);
                                    latest_sense_data = Some(RequestSenseData::from(
                                        SenseKey::IllegalRequest,
                                        AdditionalSenseCodeType::IllegalRequestInvalidCommand,
                                    ));
                                    break 'read_ep_loop;
                                };
                            }

                            crate::trace!("Send DataRequest: {:#x}", req);
                            self.storage_req_sender.send(req).await;

                            let resp = self.storage_resp_receiver.receive().await;
                            crate::trace!("Receive DataResponse: {:#x}", resp);

                            // Write処理中にWrite以外の応答が来た場合は実装不具合
                            if resp.message_id != StorageMsgId::Write {
                                crate::unreachable!("Invalid Response: {:#x}", resp);
                            }

                            // Check if the response is valid
                            if (req_tag != resp.req_tag) {
                                crate::error!("Invalid Response: {:#x}", resp);
                                latest_sense_data = Some(RequestSenseData::from(
                                    SenseKey::HardwareError,
                                    AdditionalSenseCodeType::HardwareErrorEmbeddedSoftware,
                                ));
                            }
                            // Check if there is an error
                            if let Some(error) = resp.meta_data {
                                crate::error!("Invalid Response: {:#x}", resp);
                                latest_sense_data =
                                    Some(RequestSenseData::from_data_request_error(error));
                            }
                        }

                        // CSW 応答
                        csw_packet.status =
                            CommandBlockStatus::from_bool(latest_sense_data.is_none());
                        let transfer_bytes = transfer_length * self.config.block_size;
                        if transfer_bytes < cbw_packet.data_transfer_length as usize {
                            csw_packet.data_residue =
                                (cbw_packet.data_transfer_length as usize - transfer_bytes) as u32;
                        }
                        let csw_data = csw_packet.to_data();
                        write_ep.write(&csw_data).await
                    }
                    x if x == ScsiCommand::PreventAllowMediumRemoval as u8 => {
                        crate::trace!("Prevent/Allow Medium Removal");
                        // カードの抜き差しを許可する
                        Self::handle_response_single(
                            write_ep,
                            CommandBlockStatus::CommandPassed,
                            None,
                            &cbw_packet,
                            &mut csw_packet,
                        )
                        .await
                    }
                    _ => {
                        crate::error!("Unsupported Command: {:#x}", scsi_command);
                        // save latest sense data
                        latest_sense_data = Some(RequestSenseData::from(
                            SenseKey::IllegalRequest,
                            AdditionalSenseCodeType::IllegalRequestInvalidCommand,
                        ));

                        Self::handle_response_single(
                            write_ep,
                            CommandBlockStatus::CommandFailed,
                            None,
                            &cbw_packet,
                            &mut csw_packet,
                        )
                        .await
                    }
                };

                // Phase Error時の対応
                if let Err(e) = send_resp_status {
                    crate::error!("Send Response Error: {:?}", e);
                    // Phase Error時の対応用にtagを保持
                    phase_error_tag = Some(cbw_packet.tag);
                    break 'read_ep_loop;
                }
            }

            // CSW で Phase Error を返す
            if let Some(tag) = phase_error_tag {
                crate::error!("Phase Error Tag: {:#x}", tag);
                let mut csw_packet = CommandStatusWrapperPacket::new();
                csw_packet.tag = tag;
                csw_packet.data_residue = 0;
                csw_packet.status = CommandBlockStatus::PhaseError;
                let csw_data = csw_packet.to_data();
                // 失敗してもハンドリング無理
                write_ep.write(&csw_data).await;
            }
            crate::trace!("Disconnected");
        }
    }
}
