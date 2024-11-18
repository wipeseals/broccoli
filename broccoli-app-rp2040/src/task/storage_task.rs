use crate::nand::fw_driver::NandIoFwDriver;
use crate::nand::address::NandAddr;
use crate::nand::port::NandIoPort;

use crate::share::datatype::NandStatusFlags;
use crate::share::{
    constant::*,
    datatype::StorageHandleDispatcher,
    resouce::{CHANNEL_STORAGE_RESPONSE_TO_USB_BULK, CHANNEL_USB_BULK_TO_STORAGE_REQUEST},
};
use broccoli_core::storage_handler::NandStorageHandler;

/// Core Storage Handler Task
pub async fn handle_storage_task(nandio_pins: NandIoPort<'static>) {
    // Physical Command Driver
    let mut fw_driver = NandIoFwDriver::new(nandio_pins);

    // Request Handler
    // 2IC, 1024Blocks/IC扱うことができるNandStorageHandlerを作成
    let mut storage: NandStorageHandler<
        NandAddr,
        NandStatusFlags,
        NandIoFwDriver,
        NAND_MAX_CHIP_NUM,
        MAX_NAND_BLOCKS_PER_CHIP,
    > = NandStorageHandler::new(&mut fw_driver);

    // Channel Msg <---> Request Handler
    let mut dispatcher = StorageHandleDispatcher::new(
        storage,
        CHANNEL_USB_BULK_TO_STORAGE_REQUEST.dyn_receiver(),
        CHANNEL_STORAGE_RESPONSE_TO_USB_BULK.dyn_sender(),
    );
    dispatcher.run().await;
}
