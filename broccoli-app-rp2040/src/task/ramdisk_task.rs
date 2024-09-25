use crate::share::{
    constant::*,
    datatype::StorageHandleDispatcher,
    resouce::{CHANNEL_STORAGE_RESPONSE_TO_USB_BULK, CHANNEL_USB_BULK_TO_STORAGE_REQUEST},
};
use broccoli_core::ramdisk_handler::RamDiskHandler;

/// handle RAM Disk Storage Task for Debug
pub async fn handle_ram_storage() {
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
