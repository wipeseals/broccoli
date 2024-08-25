use byteorder::{ByteOrder, LittleEndian};
use defmt::*;
use embassy_executor::{Executor, Spawner};
use embassy_futures::join::join;
use embassy_rp::bind_interrupts;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::interrupt;
use embassy_rp::multicore::{spawn_core1, Stack};
use embassy_rp::peripherals::USB;
use embassy_rp::usb::{Driver, Endpoint, In, InterruptHandler, Out};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_time::Timer;
use embassy_usb::control::{InResponse, OutResponse, Recipient, Request, RequestType};
use embassy_usb::driver::{Endpoint, EndpointIn, EndpointOut};
use embassy_usb::msos::{self, windows_version};
use embassy_usb::types::InterfaceNumber;
use embassy_usb::{Builder, Config, Handler};
use export::debug;
use static_cell::StaticCell;

use crate::channel::{LedState, LEDCONTROLCHANNEL};
use crate::usb::msc::MscHandler;

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

    let mut msc_handler = MscHandler::new();
    let mut builder = Builder::new(
        driver,
        config,
        &mut config_descriptor,
        &mut bos_descriptor,
        &mut msos_descriptor,
        &mut control_buf,
    );
    msc_handler.build(&mut builder, config);
    let mut usb = builder.build();
    let usb_fut = usb.run();
    let msc_fut = msc_handler.run(); // TODO: おそらくControl,Bulkで構造体自体共有しない方が良い

    // Run everything concurrently.
    join(usb_fut, msc_fut).await;
}

#[embassy_executor::task]
pub async fn core0_main(driver: Driver<'static, USB>) {
    usb_transport_task(driver).await;
}
