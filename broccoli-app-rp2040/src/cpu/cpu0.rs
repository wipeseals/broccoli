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
use crate::shared::datatype::{MscReqTag, StorageHandleDispatcher};
use crate::shared::resouce::{
    CHANNEL_STORAGE_RESPONSE_TO_USB_BULK, CHANNEL_USB_BULK_TO_STORAGE_REQUEST,
};
use crate::storage::protocol::{
    StorageMsgId, StorageRequest, StorageResponse, StorageResponseMetadata,
};
use crate::storage::ramdisk_handler::RamDiskHandler;
use crate::usb::msc::{BulkTransferRequest, MscBulkHandler, MscBulkHandlerConfig, MscCtrlHandler};

// Control Transfer -> Bulk Transfer Channel
static CHANNEL_USB_CTRL_TO_USB_BULK: Channel<
    CriticalSectionRawMutex,
    BulkTransferRequest,
    CHANNEL_CTRL_TO_BULK_N,
> = Channel::new();

/// Setup USB Bulk <---> StorageHandlerDispatcher Channel
async fn setup_storage_request_response_channel(req_tag: MscReqTag) -> usize {
    // wait for StorageHandler to be ready
    let setup_tag = MscReqTag::new(0xaa995566, 0); // cbw_tag: dummy data
    CHANNEL_USB_BULK_TO_STORAGE_REQUEST
        .send(StorageRequest::setup(setup_tag))
        .await;
    let setup_resp = CHANNEL_STORAGE_RESPONSE_TO_USB_BULK.receive().await;

    // setup完了時に報告された有効ブロック数をUSB Descriptorに設定する
    match setup_resp.meta_data {
        Some(StorageResponseMetadata::ReportSetupSuccess { num_blocks }) => num_blocks,
        data => crate::panic!("Setup NG: {:?}", data),
    }
}

/// Create USB Config
fn create_usb_config<'a>() -> Config<'a> {
    let mut config = Config::new(USB_VID, USB_PID);
    config.manufacturer = Some(USB_MANUFACTURER);
    config.product = Some(USB_PRODUCT);
    config.serial_number = Some(USB_SERIAL_NUMBER);
    config.max_power = USB_MAX_POWER;
    config.max_packet_size_0 = USB_MAX_PACKET_SIZE as u8;
    config
}

/// USB Control Transfer and Bulk Transfer Channel
async fn usb_transport_task(driver: Driver<'static, USB>) {
    // wait for StorageHandler to be ready
    crate::info!("Send StorageRequest(Seup) to StorageHandler");
    let num_blocks = setup_storage_request_response_channel(MscReqTag::new(0xaa995566, 0)).await;

    // Create embassy-usb Config
    crate::info!("Setup USB Ctrl/Bulk Endpoint (num_blocks: {})", num_blocks);
    let mut config = create_usb_config();

    // Create USB Handler
    let mut config_descriptor = [0; 256];
    let mut bos_descriptor = [0; 256];
    let mut msos_descriptor = [0; 256];
    let mut control_buf = [0; 64];

    let mut ctrl_handler = MscCtrlHandler::new(CHANNEL_USB_CTRL_TO_USB_BULK.dyn_sender());
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
            num_blocks,
            USB_LOGICAL_BLOCK_SIZE,
        ),
        CHANNEL_USB_CTRL_TO_USB_BULK.dyn_receiver(),
        CHANNEL_USB_BULK_TO_STORAGE_REQUEST.dyn_sender(),
        CHANNEL_STORAGE_RESPONSE_TO_USB_BULK.dyn_receiver(),
    );
    ctrl_handler.build(&mut builder, config, &mut bulk_handler);

    // Run USB Handler
    let mut usb = builder.build();
    let usb_fut = usb.run();
    let bulk_fut = bulk_handler.run();

    join(usb_fut, bulk_fut).await;
}

#[embassy_executor::task]
pub async fn main_task(driver: Driver<'static, USB>) {
    let usb_transport_fut = usb_transport_task(driver);
    usb_transport_fut.await
}
