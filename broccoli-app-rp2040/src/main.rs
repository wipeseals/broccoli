#![feature(never_type)]
#![allow(unused, dead_code)]
#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]

mod cpu0;
mod cpu1;
mod nand;
mod share;
mod task;
mod usb;

use defmt::*;
use embassy_executor::{Executor, Spawner};
use embassy_rp::bind_interrupts;
use embassy_rp::gpio::{Flex, Input, Level, Output, Pull};
use embassy_rp::multicore::{spawn_core1, Stack};
use embassy_rp::peripherals::USB;
use embassy_rp::usb::{Driver, InterruptHandler};
use nand::nand_pins::NandIoPins;
use share::constant::CORE1_TASK_STACK_SIZE;
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
    let nandio_pins: NandIoPins = NandIoPins::new(
        Flex::new(p.PIN_0),
        Flex::new(p.PIN_1),
        Flex::new(p.PIN_2),
        Flex::new(p.PIN_3),
        Flex::new(p.PIN_4),
        Flex::new(p.PIN_5),
        Flex::new(p.PIN_6),
        Flex::new(p.PIN_7),
        Output::new(p.PIN_8, Level::High),  // deassert
        Output::new(p.PIN_9, Level::High),  // deassert
        Output::new(p.PIN_10, Level::Low),  // disable
        Output::new(p.PIN_11, Level::Low),  // disable
        Output::new(p.PIN_12, Level::High), // WP disable
        Output::new(p.PIN_13, Level::High), // disable
        Output::new(p.PIN_14, Level::High), // disable
        Input::new(p.PIN_15, Pull::Up),     // pullup
    );
    let led = Output::new(p.PIN_25, Level::High);

    spawn_core1(
        p.CORE1,
        unsafe { &mut *core::ptr::addr_of_mut!(CORE1_STACK) },
        move || {
            let executor1 = EXECUTOR1.init(Executor::new());
            executor1.run(|spawner| unwrap!(spawner.spawn(cpu1::main_task(nandio_pins, led))));
        },
    );

    let driver = Driver::new(p.USB, Irqs);
    let executor0 = EXECUTOR0.init(Executor::new());
    executor0.run(|spawner| unwrap!(spawner.spawn(cpu0::main_task(driver))));
}
