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

use crate::channel::{LedState, LEDCONTROLCHANNEL};

// interfaceClass: 0x08 (Mass Storage)
const MSC_INTERFACE_CLASS: u8 = 0x08;
// interfaceSubClass: 0x06 (SCSI Primary Commands)
const MSC_INTERFACE_SUBCLASS: u8 = 0x06;
// interfaceProtocol: 0x50 (Bulk Only Transport)
const MSC_INTERFACE_PROTOCOL: u8 = 0x50;

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
    CommandBlockWrapper = 0x43425355,
    CommandStatusWrapper = 0x53425355,
    DataBlockWrapper = 0x44425355,
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
#[derive(Debug, Copy, Clone, defmt::Format)]
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
            signature: BulkTransportSignature::CommandBlockWrapper as u32,
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
        self.signature == (BulkTransportSignature::CommandBlockWrapper as u32)
    }

    /// Convert to byte array
    fn to_data(&self) -> [u8; 31] {
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
            signature: BulkTransportSignature::CommandStatusWrapper as u32,
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
    fn to_data(&self) -> [u8; 13] {
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
        self.signature == (BulkTransportSignature::CommandStatusWrapper as u32)
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
    if_num: InterfaceNumber,
    sender: DynamicSender<'d, BulkTransferRequest>,
}

/// USB Mass Storage Class Bulk Handler
/// This handler is used to handle the bulk transfers for the Mass Storage Class.
pub struct MscBulkHandler<'d, D: Driver<'d>> {
    receiver: DynamicReceiver<'d, BulkTransferRequest>,
    read_ep: Option<<D as Driver<'d>>::EndpointOut>,
    write_ep: Option<<D as Driver<'d>>::EndpointIn>,
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
                match self.sender.try_send(BulkTransferRequest::Reset) {
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
    pub fn new<const N: usize>(
        channel: &'d Channel<CriticalSectionRawMutex, BulkTransferRequest, N>,
    ) -> Self {
        Self {
            if_num: InterfaceNumber(0),
            sender: channel.dyn_sender(),
        }
    }

    pub fn build<'a, D: Driver<'d>>(
        self: &'d mut Self,
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

impl<'d, D: Driver<'d>> MscBulkHandler<'d, D> {
    pub fn new<const N: usize>(
        channel: &'d Channel<CriticalSectionRawMutex, BulkTransferRequest, N>,
    ) -> Self {
        Self {
            receiver: channel.dyn_receiver(),
            read_ep: None,
            write_ep: None,
        }
    }

    /// Main loop for bulk-only transport
    pub async fn run(&mut self) -> ! {
        crate::assert!(self.read_ep.is_some());
        crate::assert!(self.write_ep.is_some());
        let read_ep = self.read_ep.as_mut().unwrap();
        let write_ep = self.write_ep.as_mut().unwrap();

        'main_loop: loop {
            read_ep.wait_enabled().await;
            debug!("Connected");
            'read_ep_loop: loop {
                // Check if Mass Storage Reset occurred
                if (self.receiver.try_receive() == Ok(BulkTransferRequest::Reset)) {
                    debug!("Mass Storage Reset");
                    break 'read_ep_loop;
                }

                // Command Transport
                let mut read_buf = [0u8; 64];
                let Ok(read_cbw_size) = read_ep.read(&mut read_buf).await else {
                    error!("Read EP Error");
                    break 'read_ep_loop;
                };
                let Some(cbw_packet) = CommandBlockWrapperPacket::from_data(&read_buf) else {
                    error!("Invalid CBW: {:#x}", read_buf);
                    continue;
                };
                if !cbw_packet.is_valid_signature() {
                    error!("Invalid CBW signature: {:#x}", cbw_packet);
                    continue;
                };
                debug!("Got CBW: {:#x}", cbw_packet);

                // TODO: Parse SCSI Command

                // Prepare CSW
                let mut csw_packet = CommandStatusWrapperPacket::new();
                csw_packet.tag = cbw_packet.tag;
                csw_packet.data_residue = 0;
                csw_packet.status = CommandBlockStatus::CommandPassed;

                // Data Transport
                match cbw_packet.data_direction() {
                    DataDirection::HostToDevice => {
                        // Read data from the host
                        let Ok(read_data_size) = read_ep.read(&mut read_buf).await else {
                            error!("Read EP Error");
                            break 'read_ep_loop;
                        };

                        // TODO: Process data. 問題があればstatus更新しておく

                        debug!("Read Data: {:#x}", read_buf);
                    }
                    DataDirection::DeviceToHost => {
                        // Write data to the host
                        let write_length = cbw_packet.data_transfer_length;
                        let mut write_data_buf = [0u8; 64];

                        // TODO: Prepare data
                        for i in 0..64 {
                            write_data_buf[i] = i as u8;
                        }

                        debug!("Write Data: {:#x}", write_data_buf);
                        let Ok(_) = write_ep
                            .write(&write_data_buf[0..write_length as usize])
                            .await
                        else {
                            error!("Write EP Error");
                            break 'read_ep_loop;
                        };

                        // もし送信するデータがCBW指定値より小さい場合、CSWのdata_residueを設定する
                        if write_length < cbw_packet.data_transfer_length {
                            csw_packet.data_residue =
                                (cbw_packet.data_transfer_length - write_length);
                        }
                    }
                }

                // Status Transport
                let csw_data = csw_packet.to_data();
                debug!("Send CSW: {:#x}", csw_packet);
                let Ok(_) = write_ep.write(&csw_data).await else {
                    error!("Write EP Error");
                    break 'read_ep_loop;
                };
            }
            debug!("Disconnected");
        }
    }
}
