#![allow(unused, dead_code)]
#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]

extern crate nandio;
use nandio::init_nandio_pins;
use nandio::pins::NandIoPins;

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

/////////////////////////////////////////////////////////////////////////////////////////////
/// Initialize pins
fn init_pins(nandio_pins: &mut NandIoPins) {
    nandio_pins.init_all_pin();
}

/// Exec Reset Operation
fn reset(nandio_pins: &mut NandIoPins, delay: &mut cortex_m::delay::Delay, cs_index: u32) -> bool {
    // command latch 0xff (Reset)
    nandio_pins.assert_cs(cs_index);
    nandio_pins.input_command(0xff, || delay.delay_ms(1));
    nandio_pins.deassert_cs();
    delay.delay_ms(10);

    true
}
/// Exec ID Read Operation
fn id_read(
    nandio_pins: &mut NandIoPins,
    delay: &mut cortex_m::delay::Delay,
    cs_index: u32,
) -> [u8; 5] {
    let id_read_size: usize = 5;
    let mut id_read_results = [0x00, 0x00, 0x00, 0x00, 0x00];

    // command latch 0x90 (ID Read)
    // Address latch 0x00
    // Read 5 bytes
    nandio_pins.assert_cs(cs_index);
    nandio_pins.input_command(0x90, || delay.delay_ms(1));
    nandio_pins.input_address(&[0x00], || delay.delay_ms(1));
    nandio_pins.output_data(&mut id_read_results, id_read_size, || delay.delay_ms(1));
    nandio_pins.deassert_cs();

    id_read_results
}

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
    init_pins(&mut nandio_pins);

    let idread_expect_data = [0x98, 0xF1, 0x80, 0x15, 0x72];

    for cs_index in 0..2 {
        // Reset
        let _ = reset(&mut nandio_pins, &mut delay, cs_index);
        // ID Read
        let read_id_results = id_read(&mut nandio_pins, &mut delay, cs_index);

        // check ID
        if read_id_results == idread_expect_data {
            info!(
                "ID Read Success CS={} [{:02x}, {:02x}, {:02x}, {:02x}, {:02x}]",
                cs_index,
                read_id_results[0],
                read_id_results[1],
                read_id_results[2],
                read_id_results[3],
                read_id_results[4]
            );
        } else {
            warn!(
                "ID Read Fail CS={} [{:02x}, {:02x}, {:02x}, {:02x}, {:02x}]",
                cs_index,
                read_id_results[0],
                read_id_results[1],
                read_id_results[2],
                read_id_results[3],
                read_id_results[4]
            );
        }
    }

    loop {
        led_pin.set_high().unwrap();
        delay.delay_ms(1000);
        led_pin.set_low().unwrap();
        delay.delay_ms(1000);
    }
}
