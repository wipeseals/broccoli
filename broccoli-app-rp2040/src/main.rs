#![feature(never_type)]
#![allow(unused, dead_code)]
#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]

mod cpu;
mod shared;
mod storage;
mod usb;

use core::default::Default;
use core::marker::Sized;

use cpu::{cpu0, cpu1};
use defmt::*;
use embassy_executor::{Executor, Spawner};
use embassy_rp::bind_interrupts;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::multicore::{spawn_core1, Stack};
use embassy_rp::peripherals::USB;
use embassy_rp::usb::{Driver, InterruptHandler};
use shared::constant::CORE1_TASK_STACK_SIZE;
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => InterruptHandler<USB>;
});

/// Core1 Task Stack
static mut CORE1_STACK: Stack<CORE1_TASK_STACK_SIZE> = Stack::new();
/// Core0 Executor
static EXECUTOR0: StaticCell<Executor> = StaticCell::new();
/// Core1 Executor
static EXECUTOR1: StaticCell<Executor> = StaticCell::new();

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    defmt::info!("broccoli - Toy USB Mass Storage Device");

    let p = embassy_rp::init(Default::default());
    let led = Output::new(p.PIN_25, Level::High);

    spawn_core1(
        p.CORE1,
        unsafe { &mut *core::ptr::addr_of_mut!(CORE1_STACK) },
        move || {
            let executor1 = EXECUTOR1.init(Executor::new());
            executor1.run(|spawner| unwrap!(spawner.spawn(cpu1::main_task(led))));
        },
    );

    let driver = Driver::new(p.USB, Irqs);
    let executor0 = EXECUTOR0.init(Executor::new());
    executor0.run(|spawner| unwrap!(spawner.spawn(cpu0::main_task(driver))));
}
