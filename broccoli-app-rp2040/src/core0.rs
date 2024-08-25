use byteorder::{ByteOrder, LittleEndian};
use defmt::*;
use embassy_executor::{Executor, Spawner};
use embassy_futures::join::join;
use embassy_rp::bind_interrupts;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::interrupt;
use embassy_rp::multicore::{spawn_core1, Stack};
use embassy_rp::peripherals::USB;
use embassy_rp::usb::{Driver, InterruptHandler};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_time::Timer;
use embassy_usb::control::{InResponse, OutResponse, Recipient, Request, RequestType};
use embassy_usb::driver::{Endpoint, EndpointIn, EndpointOut};
use embassy_usb::msos::{self, windows_version};
use embassy_usb::types::InterfaceNumber;
use embassy_usb::{Builder, Config};
use export::debug;
use static_cell::StaticCell;

use crate::channel::{LedState, LEDCONTROLCHANNEL};

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

async fn usb_transport_task(driver: Driver<'static, USB>) {
    // Create embassy-usb Config
    let mut config = Config::new(0xc0de, 0xcafe);
    config.manufacturer = Some("wipeseals");
    config.product = Some("broccoli");
    config.serial_number = Some("snbroccoli");
    config.max_power = 100;
    config.max_packet_size_0 = 64;

    let mut config_descriptor = [0; 256];
    let mut bos_descriptor = [0; 256];
    let mut msos_descriptor = [0; 256];
    let mut control_buf = [0; 64];
    let mut builder = Builder::new(
        driver,
        config,
        &mut config_descriptor,
        &mut bos_descriptor,
        &mut msos_descriptor,
        &mut control_buf,
    );

    // MSOS Descriptorだが一旦無効にしておく
    // builder.msos_descriptor(windows_version::WIN8_1, 0);
    // builder.msos_feature(msos::CompatibleIdFeatureDescriptor::new("WINUSB", ""));
    // const DEVICE_INTERFACE_GUIDS: &[&str] = &["{AFB9A6FB-30BA-44BC-9232-806CFC875321}"];
    // builder.msos_feature(msos::RegistryPropertyFeatureDescriptor::new(
    //     "DeviceInterfaceGUIDs",
    //     msos::PropertyData::RegMultiSz(DEVICE_INTERFACE_GUIDS),
    // ));

    // Bulk Only Transport for Mass Storage
    // interfaceClass: 0x08 (Mass Storage)
    // interfaceSubClass: 0x06 (SCSI Primary Commands)
    // interfaceProtocol: 0x50 (Bulk Only Transport)
    let mut function = builder.function(0x08, 0x06, 0x50);
    let mut interface = function.interface();
    let mut alt = interface.alt_setting(0x08, 0x06, 0x50, None);
    let mut read_ep = alt.endpoint_bulk_out(64);
    let mut write_ep = alt.endpoint_bulk_in(64);

    // TODO: Control Transport for support of Mass Storage Reset and Get Max LUN
    // class command: 0x00 (Mass Storage Reset)
    // class command: 0xfe (Get Max LUN)

    drop(function);

    let mut usb = builder.build();
    let usb_fut = usb.run();

    // Bulk Transport Summary
    //
    // Command Transport
    //   Host->Device: CommandTransport(CBW): flags bit7=direction (0: Host->Device, 1: Device->Host)
    //
    // Data Transport
    //     Host->Device: DataOut  or Device->Host: DataIn
    //
    // Status Transport
    //   Device->Host: StatusTransport(CSW)
    let transport_fut = async {
        'main_loop: loop {
            read_ep.wait_enabled().await;
            debug!("Connected");
            'read_ep_loop: loop {
                // LED Indicator
                if !LEDCONTROLCHANNEL.is_full() {
                    LEDCONTROLCHANNEL.send(LedState::Toggle).await;
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
    };

    // Run everything concurrently.
    // If we had made everything `'static` above instead, we could do this using separate tasks instead.
    join(usb_fut, transport_fut).await;
}

#[embassy_executor::task]
pub async fn core0_main(driver: Driver<'static, USB>) {
    usb_transport_task(driver).await;
}
