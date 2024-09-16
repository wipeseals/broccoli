use byteorder::{ByteOrder, LittleEndian};
use defmt::*;
use embassy_executor::{Executor, Spawner};
use embassy_futures::join::join;
use embassy_rp::bind_interrupts;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::interrupt;
use embassy_rp::multicore::{spawn_core1, Stack};
use embassy_rp::pac::usb;
use embassy_rp::peripherals::USB;
use embassy_rp::usb::{Driver, In, InterruptHandler, Out};
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

use crate::shared::constant::*;
use crate::shared::datatype::{LedState, MscDataTransferTag, StorageHandleDispatcher};
use crate::storage::protocol::{DataRequestError, StorageMsgId, StorageRequest, StorageResponse};
use crate::storage::ramdisk_handler::RamDiskHandler;
use crate::usb::msc::{BulkTransferRequest, MscBulkHandler, MscBulkHandlerConfig, MscCtrlHandler};

// Control Transfer -> Bulk Transfer Channel
static CHANNEL_CTRL_TO_BULK: Channel<
    CriticalSectionRawMutex,
    BulkTransferRequest,
    CHANNEL_CTRL_TO_BULK_N,
> = Channel::new();

/// Bulk Transfer -> Internal Request Channel
static CHANNEL_MSC_TO_DATA_REQUEST: Channel<
    CriticalSectionRawMutex,
    StorageRequest<MscDataTransferTag, USB_MSC_LOGICAL_BLOCK_SIZE>,
    CHANNEL_BULK_TO_DATA_REQUEST_N,
> = Channel::new();

/// Internal Request -> Bulk Transfer Channel
static CHANNEL_MSC_RESPONSE_TO_BULK: Channel<
    CriticalSectionRawMutex,
    StorageResponse<MscDataTransferTag, USB_MSC_LOGICAL_BLOCK_SIZE>,
    CHANNEL_DATA_RESPONSE_TO_BULK_N,
> = Channel::new();

/// USB Control Transfer and Bulk Transfer Channel
async fn usb_transport_task(driver: Driver<'static, USB>) {
    // Create embassy-usb Config
    let mut config = Config::new(USB_VID, USB_PID);
    config.manufacturer = Some(USB_MANUFACTURER);
    config.product = Some(USB_PRODUCT);
    config.serial_number = Some(USB_SERIAL_NUMBER);
    config.max_power = USB_MAX_POWER;
    config.max_packet_size_0 = USB_MAX_PACKET_SIZE as u8;

    let mut config_descriptor = [0; 256];
    let mut bos_descriptor = [0; 256];
    let mut msos_descriptor = [0; 256];
    let mut control_buf = [0; 64];

    let mut ctrl_handler = MscCtrlHandler::new(CHANNEL_CTRL_TO_BULK.dyn_sender());
    let mut builder = Builder::new(
        driver,
        config,
        &mut config_descriptor,
        &mut bos_descriptor,
        &mut msos_descriptor,
        &mut control_buf,
    );
    let mut bulk_handler = MscBulkHandler::new(
        MscBulkHandlerConfig::new(
            USB_VENDOR_ID,
            USB_PRODUCT_ID,
            USB_PRODUCT_DEVICE_VERSION,
            USB_NUM_BLOCKS,
            USB_MSC_LOGICAL_BLOCK_SIZE,
        ),
        CHANNEL_CTRL_TO_BULK.dyn_receiver(),
        CHANNEL_MSC_TO_DATA_REQUEST.dyn_sender(),
        CHANNEL_MSC_RESPONSE_TO_BULK.dyn_receiver(),
    );
    ctrl_handler.build(&mut builder, config, &mut bulk_handler);

    let mut usb = builder.build();
    let usb_fut = usb.run();
    let bulk_fut = bulk_handler.run();

    // Run ramdisk for debug
    if DEBUG_ENABLE_RAM_DISK {
        let mut ramdisk: RamDiskHandler<USB_MSC_LOGICAL_BLOCK_SIZE, USB_MSC_TOTAL_CAPACITY_BYTES> =
            RamDiskHandler::new();
        ramdisk.set_fat12_sample_data();
        let mut dispatcher = StorageHandleDispatcher::new(
            ramdisk,
            CHANNEL_MSC_TO_DATA_REQUEST.dyn_receiver(),
            CHANNEL_MSC_RESPONSE_TO_BULK.dyn_sender(),
        );
        let ramdisk_fut = dispatcher.run();
        join(join(usb_fut, bulk_fut), ramdisk_fut).await;
    } else {
        // TODO: RAM Disk以外のデバイスを実装
        join(usb_fut, bulk_fut).await;
    }
}

#[embassy_executor::task]
pub async fn main_task(driver: Driver<'static, USB>) {
    let usb_transport_fut = usb_transport_task(driver);
    usb_transport_fut.await
}
