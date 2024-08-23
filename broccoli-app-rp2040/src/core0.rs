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
use static_cell::StaticCell;

use crate::channel::{LedState, LEDCONTROLCHANNEL};

/// USB Mass Storage Class Bulk-Only Transport Status
enum BulkTransferState {
    WaitCommand,
    BulkOutData,
    BulkInData,
}

/// Bulk Transport command block wrapper
#[repr(u32)]
enum BulkTransportSignature {
    CommandBlockWrapper = 0x43425355,
    CommandStatusWrapper = 0x53425355,
    DataBlockWrapper = 0x44425355,
}

/// Bulk Transport command block wrapper packet
#[derive(Debug, Copy, Clone, defmt::Format)]
struct CommandBlockWrapperPacket {
    signature: u32,
    tag: u32,
    data_transfer_length: u32,
    flags: u8,
    lun: u8,
    command_length: u8,
    command: [u8; 16],
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
    fn from_data(data: &[u8]) -> Self {
        let packet_data = CommandBlockWrapperPacket {
            signature: LittleEndian::read_u32(&data[0..4]),
            tag: LittleEndian::read_u32(&data[4..8]),
            data_transfer_length: LittleEndian::read_u32(&data[8..12]),
            flags: data[12],
            lun: data[13],
            command_length: data[14],
            command: data[15..31].try_into().unwrap(),
        };
        packet_data
    }
}

/// Bulk Transport command status wrapper packet
#[derive(Debug, Copy, Clone, defmt::Format)]
struct CommandStatusWrapperPacket {
    signature: u32,
    tag: u32,
    data_residue: u32,
    status: u8,
}

impl CommandStatusWrapperPacket {
    fn new() -> Self {
        Self {
            signature: BulkTransportSignature::CommandStatusWrapper as u32,
            tag: 0,
            data_residue: 0,
            status: 0,
        }
    }

    /// Convert to byte array
    fn to_data(&self) -> [u8; 13] {
        let mut data = [0; 13];
        LittleEndian::write_u32(&mut data[0..4], self.signature);
        LittleEndian::write_u32(&mut data[4..8], self.tag);
        LittleEndian::write_u32(&mut data[8..12], self.data_residue);
        data[12] = self.status;
        data
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
    // let mut handler = ControlHandler {
    //     if_num: InterfaceNumber(0),
    // };
    // handler.if_num = interface.interface_number();
    // builder.handler(&mut handler);

    drop(function);

    let mut usb = builder.build();
    let usb_fut = usb.run();

    // Bulk Transport Summary
    //
    // Command Transport
    //   Host->Device: CommandTransport(CBW): flags bit7=direction (bulk-in=1, bulk-out=0)
    //
    // Data Transport
    //   if bulk-in:
    //     Host->Device: bulk-in
    //   else: (bulk-out)
    //     Device->Host: bulk-out
    //
    // Status Transport
    //   Device->Host: StatusTransport(CSW)

    let transport_fut = async {
        loop {
            read_ep.wait_enabled().await;
            debug!("Connected");
            loop {
                let mut transfer_state = BulkTransferState::WaitCommand;
                let mut read_data = [0u8; 64];
                let mut write_data = [0u8; 64];
                match read_ep.read(&mut read_data).await {
                    Ok(n) => {
                        // LED Indicator
                        if !LEDCONTROLCHANNEL.is_full() {
                            LEDCONTROLCHANNEL.send(LedState::Toggle).await;
                        }
                        // Command -> Data -> Status
                        match transfer_state {
                            BulkTransferState::WaitCommand => {
                                // Parse CBW
                                let signature = LittleEndian::read_u32(&read_data[..4]);
                                if signature != (BulkTransportSignature::CommandBlockWrapper as u32)
                                {
                                    error!("Unknown signature: {:#x}", signature);
                                    continue;
                                }
                                let cbw_packet = CommandBlockWrapperPacket::from_data(&read_data);
                                debug!("Got CBW: {:#x}", cbw_packet);

                                // TODO: Parse SCSI Command
                            }
                            BulkTransferState::BulkInData => {
                                self::todo!("Not implemented yet");
                            }
                            BulkTransferState::BulkOutData => {
                                self::todo!("Not implemented yet");
                            }
                        }
                    }
                    Err(err) => {
                        error!("Read EP Error: {:?}", err);
                        break;
                    }
                }
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
