use byteorder::{ByteOrder, LittleEndian};
use defmt::*;
use embassy_executor::{Executor, Spawner};
use embassy_futures::join::join;
use embassy_rp::bind_interrupts;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::interrupt;
use embassy_rp::multicore::{spawn_core1, Stack};
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

use crate::ftl::request::{DataRequest, DataRequestError, DataRequestId, DataResponse};
use crate::shared::constant::*;
use crate::shared::datatype::{LedState, MscDataTransferTag};
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
    DataRequest<MscDataTransferTag, USB_BLOCK_SIZE>,
    CHANNEL_BULK_TO_DATA_REQUEST_N,
> = Channel::new();

/// Internal Request -> Bulk Transfer Channel
static CHANNEL_MSC_RESPONSE_TO_BULK: Channel<
    CriticalSectionRawMutex,
    DataResponse<MscDataTransferTag, USB_BLOCK_SIZE>,
    CHANNEL_DATA_RESPONSE_TO_BULK_N,
> = Channel::new();

/// USB Bulk Transfer to Internal Request Channel
/// TODO: broccoli-core に移動?
async fn data_request_task() {
    // TODO: RAM Diskパターンは実装丸ごとわけないと分岐多くてやりづらいかもしれない
    //       とりあえず、RAM Diskパターンのみ実装するが、Executor二登録する関数単位で分けるといいかもしれない
    // RAM Disk Buffer for Debug
    let mut ram_disk = [0; USB_TOTAL_SIZE];
    debug!("RAM Disk Size: {}", ram_disk.len());

    loop {
        let request = CHANNEL_MSC_TO_DATA_REQUEST.receive().await;
        debug!("DataRequest: {:?}", request);

        match request.req_id {
            DataRequestId::Setup => {
                // Setup
                // RAM Diskでは何もしない
                CHANNEL_MSC_RESPONSE_TO_BULK
                    .send(DataResponse::setup(request.req_tag))
                    .await
            }
            DataRequestId::Echo => {
                CHANNEL_MSC_RESPONSE_TO_BULK
                    .send(DataResponse::echo(request.req_tag))
                    .await
            }
            DataRequestId::Read => {
                let mut resp = DataResponse::read(request.req_tag, [0; USB_BLOCK_SIZE]);

                let ram_offset_start = request.lba * USB_BLOCK_SIZE;
                let ram_offset_end = ram_offset_start + USB_BLOCK_SIZE;

                if ram_offset_end > ram_disk.len() {
                    crate::error!("Write out of range. lba: {}", request.lba);
                    resp.error = Some(DataRequestError::OutOfRange { lba: request.lba });
                } else {
                    // データをRAM Diskからコピー
                    resp.data
                        .as_mut()
                        .unwrap()
                        .copy_from_slice(&ram_disk[ram_offset_start..ram_offset_end]);
                }
                // 応答
                CHANNEL_MSC_RESPONSE_TO_BULK.send(resp).await;
            }
            DataRequestId::Write => {
                let mut resp = DataResponse::write(request.req_tag);

                let ram_offset_start = request.lba * USB_BLOCK_SIZE;
                let ram_offset_end = ram_offset_start + USB_BLOCK_SIZE;

                // 範囲外応答
                if ram_offset_end > ram_disk.len() {
                    crate::error!("Write out of range. lba: {}", request.lba);
                    resp.error = Some(DataRequestError::OutOfRange { lba: request.lba })
                } else {
                    // データをRAM Diskにコピーしてから応答
                    ram_disk[ram_offset_start..ram_offset_end]
                        .copy_from_slice(request.data.unwrap().as_ref());
                }
                // 応答
                CHANNEL_MSC_RESPONSE_TO_BULK.send(resp).await;
            }
            DataRequestId::Flush => {
                // Flush
                // RAM Diskでは何もしない
                CHANNEL_MSC_RESPONSE_TO_BULK
                    .send(DataResponse::flush(request.req_tag))
                    .await;
            }
        };
    }
}

/// USB Control Transfer and Bulk Transfer Channel
async fn usb_transport_task(driver: Driver<'static, USB>) {
    // Create embassy-usb Config
    let mut config = Config::new(USB_VID, USB_PID);
    config.manufacturer = Some(USB_MANUFACTURER);
    config.product = Some(USB_PRODUCT);
    config.serial_number = Some(USB_SERIAL_NUMBER);
    config.max_power = USB_MAX_POWER;
    config.max_packet_size_0 = USB_MAX_PACKET_SIZE;

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
            USB_BLOCK_SIZE,
        ),
        CHANNEL_CTRL_TO_BULK.dyn_receiver(),
        CHANNEL_MSC_TO_DATA_REQUEST.dyn_sender(),
        CHANNEL_MSC_RESPONSE_TO_BULK.dyn_receiver(),
    );
    ctrl_handler.build(&mut builder, config, &mut bulk_handler);

    let mut usb = builder.build();
    let usb_fut = usb.run();
    let bulk_fut = bulk_handler.run();

    // Run everything concurrently.
    join(usb_fut, bulk_fut).await;
}

#[embassy_executor::task]
pub async fn main_task(driver: Driver<'static, USB>) {
    let usb_transport_fut = usb_transport_task(driver);
    let internal_request_fut = data_request_task();
    join(usb_transport_fut, internal_request_fut).await;
}
