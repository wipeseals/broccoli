use embassy_futures::join::join;
use embassy_rp::gpio::{Level, Output};

use crate::shared::{
    constant::*,
    datatype::StorageHandleDispatcher,
    resouce::{CHANNEL_STORAGE_RESPONSE_TO_USB_BULK, CHANNEL_USB_BULK_TO_STORAGE_REQUEST},
};
use broccoli_core::storage::{
    handler::NandStorageHandler, handler_ramdisk::RamDiskHandler, protocol::StorageHandler,
};

/// RAM Disk Debug Enable
async fn ram_dispatch_task() {
    let mut ramdisk: RamDiskHandler<USB_LOGICAL_BLOCK_SIZE, DEBUG_RAM_DISK_TOTAL_SIZE> =
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
    let mut storage: NandStorageHandler<
        USB_LOGICAL_BLOCK_SIZE,
        NAND_PAGE_TOTAL_SIZE,
        NAND_PAGE_SIZE_METADATA,
        NAND_PAGE_READ_BUFFER_N,
        NAND_PAGE_WRITE_BUFFER_N,
    > = NandStorageHandler::new();

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
