use embassy_rp::gpio::Output;
use embassy_time::Timer;

use crate::share::constant::*;

#[embassy_executor::task]
pub async fn main_task(led: Output<'static>) {
    loop {
        // NOP
        Timer::after_micros(1000).await;
    }
}
