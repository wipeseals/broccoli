use defmt::*;
use embassy_futures::join::join;
use embassy_rp::gpio::{Level, Output};

use crate::shared::{datatype::LedState, resouce::CHANNEL_USB_TO_LEDCTRL};

async fn led_task(mut led: Output<'static>) -> ! {
    loop {
        match CHANNEL_USB_TO_LEDCTRL.receive().await {
            LedState::On => led.set_high(),
            LedState::Off => led.set_low(),
            LedState::Toggle => led.toggle(),
        }
    }
}

#[embassy_executor::task]
pub async fn main_task(led: Output<'static>) {
    led_task(led).await;
    // join(led_task(led)).await;
}