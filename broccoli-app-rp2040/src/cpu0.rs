use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_rp::peripherals::USB;
use embassy_rp::usb::Driver;

use crate::nand::nand_pins::NandIoPins;
use crate::share::constant::DEBUG_ENABLE_RAM_DISK;
use crate::task::{ramdisk_task, storage_task, usb_task};

#[embassy_executor::task]
pub async fn main_task(usb_driver: Driver<'static, USB>, nandio_pins: NandIoPins<'static>) {
    let usb_transport_fut = usb_task::handle_usb_transport(usb_driver);

    if DEBUG_ENABLE_RAM_DISK {
        join(usb_transport_fut, ramdisk_task::handle_ram_storage()).await;
    } else {
        join(
            usb_transport_fut,
            storage_task::handle_storage_task(nandio_pins),
        )
        .await;
    }
}
