use embassy_rp::gpio::Output;

use crate::nand::port::NandIoPort;
use crate::task::{ramdisk_task, storage_task};

use crate::share::constant::*;

#[embassy_executor::task]
pub async fn main_task(nandio_pins: NandIoPort<'static>, led: Output<'static>) {
    if DEBUG_ENABLE_RAM_DISK {
        crate::info!("RAM Disk Enabled");
        ramdisk_task::handle_ram_storage().await;
    } else {
        crate::info!("RAM Disk Disabled");
        storage_task::handle_storage_task(nandio_pins).await;
    }
}
