use defmt::*;
use embassy_futures::join::join;
use embassy_rp::gpio::{Level, Output};

use crate::channel::{LedState, LEDCONTROLCHANNEL};

async fn led_task(mut led: Output<'static>) -> ! {
    loop {
        match LEDCONTROLCHANNEL.receive().await {
            LedState::On => led.set_high(),
            LedState::Off => led.set_low(),
            LedState::Toggle => led.toggle(),
        }
    }
}

#[embassy_executor::task]
pub async fn core1_main(led: Output<'static>) {
    led_task(led).await;
    // join(led_task(led)).await;
}
