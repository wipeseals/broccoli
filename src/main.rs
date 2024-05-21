#![allow(unused, dead_code)]
#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]

extern crate broccoli_nandio;
use broccoli_nandio::init_nandio_pins;
use broccoli_nandio::pins::NandIoPins;

use bsp::entry;

use defmt::*;
use defmt_rtt as _;
use embedded_hal::digital::v2::OutputPin;
use panic_probe as _;

use rp_pico as bsp;

use bsp::hal::{
    clocks::{init_clocks_and_plls, Clock},
    pac,
    sio::Sio,
    watchdog::Watchdog,
};

#[entry]
fn main() -> ! {
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
    // assign LED pin (gpio25)
    let mut led_pin = pins.led.into_push_pull_output();
    // assign nandio pins (gpio0~gpio15)
    let mut nandio_pins = init_nandio_pins!(pins);
    let mut command = broccoli_nandio::cmd::Command {
        nandio_pins: &mut nandio_pins,
        delay: &mut delay,
    };
    command.init_pins();

    for cs_index in 0..2 {
        // Reset
        command.reset(cs_index);
        // ID Read
        let (read_ok, id_read_data) = command.id_read(cs_index);

        // check ID
        info!(
            "ID Read {} CS={} [{:02x}, {:02x}, {:02x}, {:02x}, {:02x}]",
            if read_ok { "OK" } else { "Fail" },
            cs_index,
            id_read_data[0],
            id_read_data[1],
            id_read_data[2],
            id_read_data[3],
            id_read_data[4]
        );
    }

    loop {
        led_pin.set_high().unwrap();
        delay.delay_ms(1000);
        led_pin.set_low().unwrap();
        delay.delay_ms(1000);
    }
}
