#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]

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
use embassy_usb::driver::{Endpoint, EndpointIn, EndpointOut};
use embassy_usb::{Builder, Config};
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => InterruptHandler<USB>;
});

///////////////////////////////////////////////////////////////////////////////
/// shared resources
///////////////////////////////////////////////////////////////////////////////

enum LedState {
    On,
    Off,
    Toggle,
}
static CHANNEL: Channel<CriticalSectionRawMutex, LedState, 1> = Channel::new();

///////////////////////////////////////////////////////////////////////////////
/// core1 task
///////////////////////////////////////////////////////////////////////////////

#[embassy_executor::task]
async fn core1_led_task(mut led: Output<'static>) {
    info!("Hello from core 1");
    loop {
        match CHANNEL.receive().await {
            LedState::On => led.set_high(),
            LedState::Off => led.set_low(),
            LedState::Toggle => led.toggle(),
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
/// core0 task
///////////////////////////////////////////////////////////////////////////////

#[embassy_executor::task]
async fn core0_usb_task(mut driver: Driver<'static, USB>) {
    // Create embassy-usb Config
    let mut config = Config::new(0xc0de, 0xcafe);
    config.manufacturer = Some("wipeseals");
    config.product = Some("broccoli");
    config.serial_number = Some("snbroccoli");
    config.max_power = 100;
    config.max_packet_size_0 = 64;

    // Create embassy-usb DeviceBuilder using the driver and config.
    // It needs some buffers for building the descriptors.
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

    // interfaceClass: 0x08 (Mass Storage)
    // interfaceSubClass: 0x06 (SCSI Primary Commands)
    // interfaceProtocol: 0x50 (Bulk Only Transport)
    let mut function = builder.function(0x08, 0x06, 0x50);
    let mut interface = function.interface();
    let mut alt = interface.alt_setting(0x08, 0x06, 0x50, None);
    let mut read_ep = alt.endpoint_bulk_out(64);
    let mut write_ep = alt.endpoint_bulk_in(64);
    drop(function);

    let mut usb = builder.build();
    let usb_fut = usb.run();
    let usb_scsi_bbb_fut = async {
        loop {
            read_ep.wait_enabled().await;
            info!("Connected");
            loop {
                CHANNEL.send(LedState::Toggle).await;
                let mut data = [0; 64];
                match read_ep.read(&mut data).await {
                    Ok(n) => {
                        CHANNEL.send(LedState::Toggle).await;
                        info!("Got bulk: {:x}", data[..n]);

                        let signature = LittleEndian::read_u32(&data[..4]);
                        match signature {
                            x if x == BulkTransportSignature::CommandBlockWrapper as u32 => {
                                let packet_data = CommandBlockWrapperPacket {
                                    signature,
                                    tag: LittleEndian::read_u32(&data[4..8]),
                                    data_transfer_length: LittleEndian::read_u32(&data[8..12]),
                                    flags: data[12],
                                    lun: data[13],
                                    command_length: data[14],
                                    command: data[15..31].try_into().unwrap(),
                                };
                                info!("Got CBW: {:#x}", packet_data);
                            }
                            x if x == BulkTransportSignature::CommandStatusWrapper as u32 => {
                                info!("Got CSW");
                            }
                            x if x == BulkTransportSignature::DataBlockWrapper as u32 => {
                                info!("Got DBW");
                            }
                            _ => {
                                info!("Unknown signature");
                            }
                        }
                        // Echo back to the host:
                        write_ep.write(&data[..n]).await.ok();
                    }
                    Err(_) => break,
                }
            }
            info!("Disconnected");
        }
    };

    // Run everything concurrently.
    // If we had made everything `'static` above instead, we could do this using separate tasks instead.
    join(usb_fut, usb_scsi_bbb_fut).await;
}

///////////////////////////////////////////////////////////////////////////////
/// main entry point for the application.
/// core0 will run this function.
///////////////////////////////////////////////////////////////////////////////

static mut CORE1_STACK: Stack<4096> = Stack::new();
static EXECUTOR0: StaticCell<Executor> = StaticCell::new();
static EXECUTOR1: StaticCell<Executor> = StaticCell::new();

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Hello there!");

    let p = embassy_rp::init(Default::default());
    let led = Output::new(p.PIN_25, Level::High);

    spawn_core1(
        p.CORE1,
        unsafe { &mut *core::ptr::addr_of_mut!(CORE1_STACK) },
        move || {
            let executor1 = EXECUTOR1.init(Executor::new());
            executor1.run(|spawner| unwrap!(spawner.spawn(core1_led_task(led))));
        },
    );

    let driver = Driver::new(p.USB, Irqs);
    let executor0 = EXECUTOR0.init(Executor::new());
    executor0.run(|spawner| unwrap!(spawner.spawn(core0_usb_task(driver))));
}

/// SCSI command block wrapper
#[repr(u32)]
enum BulkTransportSignature {
    CommandBlockWrapper = 0x43425355,
    CommandStatusWrapper = 0x53425355,
    DataBlockWrapper = 0x44425355,
}

/// SCSI command block wrapper packet
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
