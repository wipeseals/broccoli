#![allow(unused, dead_code)]
#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]

use core::borrow::BorrowMut;
use core::future::Future;
use core::ops::DerefMut;
use core::pin::Pin;
use core::task::{Context, Poll};
use core::task::{RawWaker, RawWakerVTable, Waker};

use cortex_m::delay::Delay;
use heapless::Vec;

use bsp::entry;
use defmt::*;
use defmt_rtt as _;
use panic_probe as _;
use rp_pico as bsp;

use embedded_hal::digital::v2::OutputPin;

use bsp::hal::{
    clocks::{init_clocks_and_plls, Clock},
    pac,
    sio::Sio,
    watchdog::Watchdog,
};

use broccoli_nandio::{commander::Commander, driver::Driver};
use broccoli_nandio_rp2040::{driver::Rp2040FwDriver, init_nandio_pins, pins::NandIoPins};

struct TaskExecutor {
    waker: Waker,
}

impl TaskExecutor {
    /// Create a simple waker
    unsafe fn create_waker() -> Waker {
        const VTABLE: RawWakerVTable = RawWakerVTable::new(
            raw_waker_clone,
            raw_waker_wake,
            raw_waker_wake_by_ref,
            raw_waker_drop,
        );

        /// Clone the raw waker
        fn raw_waker_clone(_data: *const ()) -> RawWaker {
            RawWaker::new(core::ptr::null(), &VTABLE)
        }

        /// Wake the task associated with the raw waker
        fn raw_waker_wake(_data: *const ()) {}

        /// Wake the task associated with the raw waker by reference
        fn raw_waker_wake_by_ref(_data: *const ()) {}

        /// Drop the raw waker
        fn raw_waker_drop(_data: *const ()) {}

        let raw_waker = RawWaker::new(core::ptr::null(), &VTABLE);

        unsafe { Waker::from_raw(raw_waker) }
    }

    /// Create TaskExecutor
    fn new() -> Self {
        let waker = unsafe { Self::create_waker() };
        Self { waker }
    }

    /// Run the tasks
    fn run(&mut self, task: impl Future<Output = ()>) {
        let mut pinned_task = core::pin::pin!(task);
        let mut ctx = &mut Context::from_waker(&self.waker);
        loop {
            match pinned_task.as_mut().poll(&mut ctx) {
                Poll::Ready(_) => break,
                _ => continue,
            };
        }
    }
}

#[entry]
fn main() -> ! {
    // run CPU0
    let mut executor = TaskExecutor::new();
    executor.run(async move {
        let mut pac = pac::Peripherals::take().unwrap();
        let core = pac::CorePeripherals::take().unwrap();
        let mut watchdog = Watchdog::new(pac.WATCHDOG);
        let sio = Sio::new(pac.SIO);

        // External high-speed crystal on the pico board is 12Mhz
        let external_xtal_freq_hz = 12_000_000u32;
        let clocks = init_clocks_and_plls(
            external_xtal_freq_hz,
            pac.XOSC,
            pac.CLOCKS,
            pac.PLL_SYS,
            pac.PLL_USB,
            &mut pac.RESETS,
            &mut watchdog,
        )
        .ok()
        .unwrap();
        let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

        // setup gpio
        let pins = bsp::Pins::new(
            pac.IO_BANK0,
            pac.PADS_BANK0,
            sio.gpio_bank0,
            &mut pac.RESETS,
        );
        let mut led_pin = pins.led.into_push_pull_output();
        let mut nandio_pins = init_nandio_pins!(pins);

        // init drivers
        let mut nandio_driver = Rp2040FwDriver {
            nandio_pins: &mut nandio_pins,
            delay: &mut delay,
        };
        nandio_driver.init_pins();
        let mut commander = Commander::new();
        // setup & check badblock
        commander.setup(&mut nandio_driver).await;
        let badblock_bitarrs = commander.create_badblock_bitarr(&mut nandio_driver).await;
        for bitarr in badblock_bitarrs {
            for i in 0..bitarr.data_len() {
                info!("{}: {:?}", i, bitarr.get(i));
            }
        }
    });
    loop {}
}
