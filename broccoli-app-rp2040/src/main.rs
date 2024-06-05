#![allow(unused, dead_code)]
#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]

use broccoli_nandio_rp2040::{driver::Rp2040FwDriver, init_nandio_pins};
use bsp::entry;

use defmt::*;
use defmt_rtt as _;
use embedded_hal::digital::v2::OutputPin;
use panic_probe as _;

use broccoli_nandio::{commander::Commander, driver::Driver};
use broccoli_nandio_rp2040::pins::NandIoPins;
use rp_pico as bsp;

use bsp::hal::{
    clocks::{init_clocks_and_plls, Clock},
    pac,
    sio::Sio,
    watchdog::Watchdog,
};

#[entry]
async fn main() -> ! {
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

    // setup gpio
    let pins = bsp::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );
    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

    // assign LED pin (gpio25)
    let mut led_pin = pins.led.into_push_pull_output();
    led_pin.set_high().unwrap();
    // assign nandio pins (gpio0~gpio15)
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
            info!("{:?}", bitarr.get(i));
        }
    }

    loop {}
}
