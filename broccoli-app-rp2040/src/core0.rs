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

use crate::constants::*;
use crate::ftl::interface::{
    DataRequest, DataRequestError, DataRequestId, DataResponse, DataResponseStatus,
};
use crate::usb::msc::{BulkTransferRequest, MscBulkHandler, MscBulkHandlerConfig, MscCtrlHandler};

/// Bulk Transfer -> Internal Request Channel
static CHANNEL_BULK_TO_INTERNAL: Channel<
    CriticalSectionRawMutex,
    DataRequest,
    CHANNEL_BULK_TO_INTERNAL_N,
> = Channel::new();

/// Internal Request -> Bulk Transfer Channel
static CHANNEL_INTERNAL_TO_BULK: Channel<
    CriticalSectionRawMutex,
    DataResponse,
    CHANNEL_INTERNAL_TO_BULK_N,
> = Channel::new();

/// USB Bulk Transfer to Internal Request Channel
async fn internal_request_task() {
    loop {
        let request = CHANNEL_BULK_TO_INTERNAL.receive().await;
        match request.req_id {
            DataRequestId::Echo => {
                let response = DataResponse {
                    req_id: DataRequestId::Echo,
                    requester_tag: request.requester_tag,
                    data_buf_id: request.data_buf_id,
                    resp_status: DataResponseStatus::Success,
                };
                CHANNEL_INTERNAL_TO_BULK.send(response).await;
            }
            DataRequestId::Read => {
                let response = DataResponse {
                    req_id: DataRequestId::Read,
                    requester_tag: request.requester_tag,
                    data_buf_id: request.data_buf_id,
                    resp_status: DataResponseStatus::Error {
                        code: DataRequestError::NotImplemented,
                    },
                };
                CHANNEL_INTERNAL_TO_BULK.send(response).await;
            }
            DataRequestId::Write => {
                let response = DataResponse {
                    req_id: DataRequestId::Write,
                    requester_tag: request.requester_tag,
                    data_buf_id: request.data_buf_id,
                    resp_status: DataResponseStatus::Error {
                        code: DataRequestError::NotImplemented,
                    },
                };
                CHANNEL_INTERNAL_TO_BULK.send(response).await;
            }
            DataRequestId::Flush => {
                let response = DataResponse {
                    req_id: DataRequestId::Flush,
                    requester_tag: request.requester_tag,
                    data_buf_id: request.data_buf_id,
                    resp_status: DataResponseStatus::Error {
                        code: DataRequestError::NotImplemented,
                    },
                };
                CHANNEL_INTERNAL_TO_BULK.send(response).await;
            }
        }
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

    // Control Transfer -> Bulk Transfer Channel
    let mut channel_ctrl_to_bulk: Channel<
        CriticalSectionRawMutex,
        BulkTransferRequest,
        CHANNEL_CTRL_TO_BULK_N,
    > = Channel::new();
    let mut ctrl_handler = MscCtrlHandler::new(channel_ctrl_to_bulk.dyn_sender());
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
        channel_ctrl_to_bulk.dyn_receiver(),
        CHANNEL_BULK_TO_INTERNAL.dyn_sender(),
        CHANNEL_INTERNAL_TO_BULK.dyn_receiver(),
    );
    ctrl_handler.build(&mut builder, config, &mut bulk_handler);

    let mut usb = builder.build();
    let usb_fut = usb.run();
    let bulk_fut = bulk_handler.run();

    // Run everything concurrently.
    join(usb_fut, bulk_fut).await;
}

#[embassy_executor::task]
pub async fn core0_main(driver: Driver<'static, USB>) {
    let usb_transport_fut = usb_transport_task(driver);
    let internal_request_fut = internal_request_task();
    join(usb_transport_fut, internal_request_fut).await;
}
