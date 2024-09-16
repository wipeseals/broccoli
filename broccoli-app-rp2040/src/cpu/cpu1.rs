use embassy_futures::join::join;
use embassy_rp::gpio::{Level, Output};

use crate::{
    shared::{
        constant::*,
        datatype::StorageHandleDispatcher,
        resouce::{CHANNEL_STORAGE_RESPONSE_TO_USB_BULK, CHANNEL_USB_BULK_TO_STORAGE_REQUEST},
    },
    storage::{
        core_handler::StorageCoreHandler, protocol::StorageHandler, ramdisk_handler::RamDiskHandler,
    },
};

/// RAM Disk Debug Enable
async fn ram_dispatch_task() {
    let mut ramdisk: RamDiskHandler<USB_MSC_LOGICAL_BLOCK_SIZE, USB_MSC_TOTAL_CAPACITY_BYTES> =
        RamDiskHandler::new();
    ramdisk.set_fat12_sample_data();
    let mut dispatcher = StorageHandleDispatcher::new(
        ramdisk,
        CHANNEL_USB_BULK_TO_STORAGE_REQUEST.dyn_receiver(),
        CHANNEL_STORAGE_RESPONSE_TO_USB_BULK.dyn_sender(),
    );
    dispatcher.run().await;
}

/// Core Storage Handler Task
async fn core_dispatch_task() {
    let mut storage: StorageCoreHandler<
        USB_MSC_LOGICAL_BLOCK_SIZE,
        NAND_PAGE_SIZE_USABLE,
        NAND_PAGE_READ_BUFFER_N,
        NAND_PAGE_WRITE_BUFFER_N,
    > = StorageCoreHandler::new();

    // TODO: Implement NAND Flash Communication

    let mut dispatcher = StorageHandleDispatcher::new(
        storage,
        CHANNEL_USB_BULK_TO_STORAGE_REQUEST.dyn_receiver(),
        CHANNEL_STORAGE_RESPONSE_TO_USB_BULK.dyn_sender(),
    );
    dispatcher.run().await;
}

#[embassy_executor::task]
pub async fn main_task(led: Output<'static>) {
    if DEBUG_ENABLE_RAM_DISK {
        crate::info!("RAM Disk Enabled");
        ram_dispatch_task().await;
    } else {
        crate::info!("RAM Disk Disabled");
        core_dispatch_task().await;
    }
}
