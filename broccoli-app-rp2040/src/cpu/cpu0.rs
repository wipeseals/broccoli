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

use crate::ftl::request::{DataRequest, DataRequestError, DataResponse};
use crate::shared::constant::*;
use crate::shared::datatype::{LedState, MscDataTransferTag};
use crate::shared::resouce::{CHANNEL_USB_TO_LEDCTRL, LOGICAL_BLOCK_SHARED_BUFFER_MANAGER};
use crate::usb::msc::{BulkTransferRequest, MscBulkHandler, MscBulkHandlerConfig, MscCtrlHandler};

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

        match request {
            DataRequest::Setup { req_tag } => {
                // Setup
                // RAM Diskでは何もしない
                CHANNEL_MSC_RESPONSE_TO_BULK
                    .send(DataResponse::Setup {
                        req_tag,
                        error: None,
                    })
                    .await
            }
            DataRequest::Echo { req_tag } => {
                CHANNEL_MSC_RESPONSE_TO_BULK
                    .send(DataResponse::Echo {
                        req_tag,
                        error: None,
                    })
                    .await
            }
            DataRequest::Read {
                req_tag,
                lba,
                transfer_length: block_count,
            } => {
                // block_count分のデータをRAM Diskから読み出してShared Bufferにコピー
                // block_count=0の場合は何もしない
                for block_index in 0..block_count {
                    let read_buf_id = {
                        // TODO: spinlockうまく行っていないかもしれない
                        let mut buffer_manager = LOGICAL_BLOCK_SHARED_BUFFER_MANAGER.lock().await;

                        // Allocate Shared Buffer
                        let Some(read_buf_id) = buffer_manager
                            .allocate_with_retry(
                                req_tag,
                                || async {
                                    Timer::after_micros(BUFFER_ALLOCATION_FAIL_RETRY_DURATION_US)
                                        .await
                                },
                                BUFFER_ALLOCATION_FAIL_RETRY_COUNT_MAX,
                            )
                            .await
                        else {
                            crate::unreachable!(
                                "allocate_with_retry failed for Read. req_tag: {:?}",
                                req_tag
                            );
                        };

                        read_buf_id
                    };

                    // 読み出し先決定
                    let ram_offset = (lba + block_index) * USB_BLOCK_SIZE;
                    // 範囲外応答
                    if ram_offset + USB_BLOCK_SIZE > ram_disk.len() {
                        crate::error!(
                            "Read out of range. lba: {}, block_index: {}",
                            lba,
                            block_index
                        );
                        // 応答
                        CHANNEL_MSC_RESPONSE_TO_BULK
                            .send(DataResponse::Read {
                                req_tag,
                                read_buf_id,
                                transfer_count: block_index,
                                error: Some(DataRequestError::OutOfRange {
                                    lba: lba + block_index,
                                }),
                            })
                            .await;
                    } else {
                        // データをShared Bufferにコピー
                        let mut buffer_manager = LOGICAL_BLOCK_SHARED_BUFFER_MANAGER.lock().await;
                        buffer_manager
                            .lock_buffer(read_buf_id)
                            .await
                            .copy_from_slice(&ram_disk[ram_offset..ram_offset + USB_BLOCK_SIZE]);
                        // 応答
                        CHANNEL_MSC_RESPONSE_TO_BULK
                            .send(DataResponse::Read {
                                req_tag,
                                read_buf_id,
                                transfer_count: block_index,
                                error: None,
                            })
                            .await;
                    }
                }
            }
            DataRequest::Write {
                req_tag,
                lba,
                write_buf_id,
            } => {
                // 書き込み先決定
                let ram_offset = lba * USB_BLOCK_SIZE;
                // 範囲外応答
                if ram_offset + USB_BLOCK_SIZE > ram_disk.len() {
                    crate::error!("Write out of range. lba: {}", lba);
                    // Bufferを解放
                    {
                        let mut buffer_manager = LOGICAL_BLOCK_SHARED_BUFFER_MANAGER.lock().await;
                        buffer_manager.free(write_buf_id).await;
                    }
                    // 応答
                    CHANNEL_MSC_RESPONSE_TO_BULK
                        .send(DataResponse::Write {
                            req_tag,
                            error: Some(DataRequestError::OutOfRange { lba }),
                        })
                        .await;
                } else {
                    {
                        let mut buffer_manager = LOGICAL_BLOCK_SHARED_BUFFER_MANAGER.lock().await;
                        // データをRAM Diskにコピーしてから応答
                        ram_disk[ram_offset..ram_offset + USB_BLOCK_SIZE].copy_from_slice(
                            buffer_manager.lock_buffer(write_buf_id).await.as_ref(),
                        );
                        // Bufferを解放
                        buffer_manager.free(write_buf_id).await;
                    }
                    // 応答
                    CHANNEL_MSC_RESPONSE_TO_BULK
                        .send(DataResponse::Write {
                            req_tag,
                            error: None,
                        })
                        .await;
                }
            }
            DataRequest::Flush { req_tag } => {
                // Flush
                // RAM Diskでは何もしない
                CHANNEL_MSC_RESPONSE_TO_BULK
                    .send(DataResponse::Flush {
                        req_tag,
                        error: None,
                    })
                    .await
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
