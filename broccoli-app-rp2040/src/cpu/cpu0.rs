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

use crate::ftl::ramdisk::RamDisk;
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
    DataRequest<MscDataTransferTag, USB_MSC_LOGICAL_BLOCK_SIZE>,
    CHANNEL_BULK_TO_DATA_REQUEST_N,
> = Channel::new();

/// Internal Request -> Bulk Transfer Channel
static CHANNEL_MSC_RESPONSE_TO_BULK: Channel<
    CriticalSectionRawMutex,
    DataResponse<MscDataTransferTag, USB_MSC_LOGICAL_BLOCK_SIZE>,
    CHANNEL_DATA_RESPONSE_TO_BULK_N,
> = Channel::new();

/// Set FAT12 Data to RAM Disk
/// refs. https://github.com/hathach/tinyusb/blob/master/examples/device/cdc_msc/src/msc_disk.c#L52
#[rustfmt::skip]
fn set_fat12_data<'a>(
    ramdisk: &'a mut RamDisk<
        MscDataTransferTag,
        USB_MSC_LOGICAL_BLOCK_SIZE,
        USB_MSC_TOTAL_CAPACITY_BYTES,
    >,
) {
    let readme_contents = b"Hello, broccoli!\n";
    // LBA0: MBR
    ramdisk.set_data(
        0,
        &[
        /// |  0|    1|    2|    3|    4|    5|    6|    7|    8|    9|  0xa| 0xb|  0xc|  0xd|  0xe|  0xf|
            0xEB, 0x3C, 0x90, 0x4D, 0x53, 0x44, 0x4F, 0x53, 0x35, 0x2E, 0x30, 0x00, 0x02, 0x01, 0x01, 0x00, // 0x00
            0x01, 0x10, 0x00, 0x10, 0x00, 0xF8, 0x01, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, // 0x10
            0x00, 0x00, 0x00, 0x00, 0x80, 0x00, 0x29, 0x34, 0x12, 0x00, 0x00, b'B', b'r', b'o', b'c', b'c', // 0x20
            b'o', b'l', b'i', b'M', b'S', b'C', 0x46, 0x41, 0x54, 0x31, 0x32, 0x20, 0x20, 0x20, 0x00, 0x00, // 0x30
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 0x40
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 0x50
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 0x60
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 0x70
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 0x80
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 0x90
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 0xa0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 0xb0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 0xc0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 0xd0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 0xe0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x55, 0xaa, // 0xf0
        ],
    );
    // LBA1: FAT12 Table
    ramdisk.set_data(512, &[0xF8, 0xFF, 0xFF, 0x00, 0x00]);
    // LBA2: Root Directory
    let flen = (readme_contents.len() - 1) as u8;
    ramdisk.set_data(
        1024,
        &[
        /// first entry is volume label
        /// |  0|    1|    2|    3|    4|    5|    6|    7|    8|    9|  0xa| 0xb|  0xc|  0xd|  0xe|  0xf|
            b'B', b'r', b'o', b'c', b'c', b'o', b'l', b'i', b'M', b'S', b'C', 0x08, 0x00, 0x00, 0x00, 0x00, // volume label
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x4F, 0x6D, 0x65, 0x43, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // readme file
            b'R', b'E', b'A', b'D', b'M', b'E', b' ', b' ', b'T', b'X', b'T', 0x20, 0x00, 0xC6, 0x52, 0x6D, // readme file
            b'e', b'C', b'e', b'C', 0x00, 0x00, 0x88, 0x6D, 0x65, 0x43, 0x02, 0x00, flen, 0x00, 0x00, 0x00, // readme file
        ],
    );
    // lba3 readme file
    ramdisk.set_data(1536, readme_contents);
}
#[rustfmt::skip]


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
        let mut ramdisk: RamDisk<
            MscDataTransferTag,
            USB_MSC_LOGICAL_BLOCK_SIZE,
            USB_MSC_TOTAL_CAPACITY_BYTES,
        > = RamDisk::new(
            CHANNEL_MSC_TO_DATA_REQUEST.dyn_receiver(),
            CHANNEL_MSC_RESPONSE_TO_BULK.dyn_sender(),
        );
        set_fat12_data(&mut ramdisk);
        let ramdisk_fut = ramdisk.run();
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
