#![no_std]
#![no_main]

use bsp::entry;
use defmt::*;
use defmt_rtt as _;
use embedded_hal::digital::v2::{InputPin, OutputPin};
use panic_probe as _;

use rp_pico as bsp;

use bsp::hal::{
    clocks::{init_clocks_and_plls, Clock},
    pac,
    sio::Sio,
    watchdog::Watchdog,
};

/// NAND I/O Pins
struct NandIoPins<'a> {
    io0: &'a mut bsp::hal::gpio::Pin<
        bsp::hal::gpio::bank0::Gpio0,
        bsp::hal::gpio::FunctionSioOutput,
        bsp::hal::gpio::PullDown,
    >,
    io1: &'a mut bsp::hal::gpio::Pin<
        bsp::hal::gpio::bank0::Gpio1,
        bsp::hal::gpio::FunctionSioOutput,
        bsp::hal::gpio::PullDown,
    >,
    io2: &'a mut bsp::hal::gpio::Pin<
        bsp::hal::gpio::bank0::Gpio2,
        bsp::hal::gpio::FunctionSioOutput,
        bsp::hal::gpio::PullDown,
    >,
    io3: &'a mut bsp::hal::gpio::Pin<
        bsp::hal::gpio::bank0::Gpio3,
        bsp::hal::gpio::FunctionSioOutput,
        bsp::hal::gpio::PullDown,
    >,
    io4: &'a mut bsp::hal::gpio::Pin<
        bsp::hal::gpio::bank0::Gpio4,
        bsp::hal::gpio::FunctionSioOutput,
        bsp::hal::gpio::PullDown,
    >,
    io5: &'a mut bsp::hal::gpio::Pin<
        bsp::hal::gpio::bank0::Gpio5,
        bsp::hal::gpio::FunctionSioOutput,
        bsp::hal::gpio::PullDown,
    >,
    io6: &'a mut bsp::hal::gpio::Pin<
        bsp::hal::gpio::bank0::Gpio6,
        bsp::hal::gpio::FunctionSioOutput,
        bsp::hal::gpio::PullDown,
    >,
    io7: &'a mut bsp::hal::gpio::Pin<
        bsp::hal::gpio::bank0::Gpio7,
        bsp::hal::gpio::FunctionSioOutput,
        bsp::hal::gpio::PullDown,
    >,
    ceb0: &'a mut bsp::hal::gpio::Pin<
        bsp::hal::gpio::bank0::Gpio8,
        bsp::hal::gpio::FunctionSioOutput,
        bsp::hal::gpio::PullDown,
    >,
    ceb1: &'a mut bsp::hal::gpio::Pin<
        bsp::hal::gpio::bank0::Gpio9,
        bsp::hal::gpio::FunctionSioOutput,
        bsp::hal::gpio::PullDown,
    >,
    cle: &'a mut bsp::hal::gpio::Pin<
        bsp::hal::gpio::bank0::Gpio10,
        bsp::hal::gpio::FunctionSioOutput,
        bsp::hal::gpio::PullDown,
    >,
    ale: &'a mut bsp::hal::gpio::Pin<
        bsp::hal::gpio::bank0::Gpio11,
        bsp::hal::gpio::FunctionSioOutput,
        bsp::hal::gpio::PullDown,
    >,
    wpb: &'a mut bsp::hal::gpio::Pin<
        bsp::hal::gpio::bank0::Gpio12,
        bsp::hal::gpio::FunctionSioOutput,
        bsp::hal::gpio::PullDown,
    >,
    web: &'a mut bsp::hal::gpio::Pin<
        bsp::hal::gpio::bank0::Gpio13,
        bsp::hal::gpio::FunctionSioOutput,
        bsp::hal::gpio::PullDown,
    >,
    reb: &'a mut bsp::hal::gpio::Pin<
        bsp::hal::gpio::bank0::Gpio14,
        bsp::hal::gpio::FunctionSioOutput,
        bsp::hal::gpio::PullDown,
    >,
    rbb: &'a mut bsp::hal::gpio::Pin<
        bsp::hal::gpio::bank0::Gpio15,
        bsp::hal::gpio::FunctionSioInput,
        bsp::hal::gpio::PullUp,
    >,
}
impl NandIoPins<'_> {
    /// Set data
    /// data: write data to IO pins
    pub fn set_io_pin_data(&mut self, data: u8) {
        self.io0
            .into_push_pull_output_in_state(if data & 0x01 != 0 {
                bsp::hal::gpio::PinState::High
            } else {
                bsp::hal::gpio::PinState::Low
            });
        self.io1
            .into_push_pull_output_in_state(if data & 0x02 != 0 {
                bsp::hal::gpio::PinState::High
            } else {
                bsp::hal::gpio::PinState::Low
            });
        self.io2
            .into_push_pull_output_in_state(if data & 0x04 != 0 {
                bsp::hal::gpio::PinState::High
            } else {
                bsp::hal::gpio::PinState::Low
            });
        self.io3
            .into_push_pull_output_in_state(if data & 0x08 != 0 {
                bsp::hal::gpio::PinState::High
            } else {
                bsp::hal::gpio::PinState::Low
            });
        self.io4
            .into_push_pull_output_in_state(if data & 0x10 != 0 {
                bsp::hal::gpio::PinState::High
            } else {
                bsp::hal::gpio::PinState::Low
            });
        self.io5
            .into_push_pull_output_in_state(if data & 0x20 != 0 {
                bsp::hal::gpio::PinState::High
            } else {
                bsp::hal::gpio::PinState::Low
            });
        self.io6
            .into_push_pull_output_in_state(if data & 0x40 != 0 {
                bsp::hal::gpio::PinState::High
            } else {
                bsp::hal::gpio::PinState::Low
            });
        self.io7
            .into_push_pull_output_in_state(if data & 0x80 != 0 {
                bsp::hal::gpio::PinState::High
            } else {
                bsp::hal::gpio::PinState::Low
            });
    }

    /// Get data
    /// return: read data from IO pins
    pub fn get_io_pin_data(&mut self) -> u8 {
        let mut data: u8 = 0;
        data |= if self.io0.into_pull_down_input().is_high().unwrap() {
            0x01
        } else {
            0x00
        };
        data |= if self.io1.into_pull_down_input().is_high().unwrap() {
            0x02
        } else {
            0x00
        };
        data |= if self.io2.into_pull_down_input().is_high().unwrap() {
            0x04
        } else {
            0x00
        };
        data |= if self.io3.into_pull_down_input().is_high().unwrap() {
            0x08
        } else {
            0x00
        };
        data |= if self.io4.into_pull_down_input().is_high().unwrap() {
            0x10
        } else {
            0x00
        };
        data |= if self.io5.into_pull_down_input().is_high().unwrap() {
            0x20
        } else {
            0x00
        };
        data |= if self.io6.into_pull_down_input().is_high().unwrap() {
            0x40
        } else {
            0x00
        };
        data |= if self.io7.into_pull_down_input().is_high().unwrap() {
            0x80
        } else {
            0x00
        };
        data
    }

    /// set CS
    /// cs_index: chip select index
    pub fn assert_cs(&mut self, cs_index: u32) {
        match cs_index {
            0 => {
                self.ceb0
                    .into_push_pull_output_in_state(bsp::hal::gpio::PinState::Low);
                self.ceb1
                    .into_push_pull_output_in_state(bsp::hal::gpio::PinState::High);
            }
            1 => {
                self.ceb0
                    .into_push_pull_output_in_state(bsp::hal::gpio::PinState::High);
                self.ceb1
                    .into_push_pull_output_in_state(bsp::hal::gpio::PinState::Low);
            }
            _ => {
                crate::panic!("Invalid CS index")
            }
        }
    }

    /// deassert CS
    pub fn deassert_cs(&mut self) {
        self.ceb0
            .into_push_pull_output_in_state(bsp::hal::gpio::PinState::High);
        self.ceb1
            .into_push_pull_output_in_state(bsp::hal::gpio::PinState::High);
    }

    /// Set Function Pins
    /// command_latch: command latch (CLE)
    /// address_latch: address latch (ALE)
    /// write_enable: write enable (/WE)
    /// read_enable: read enable (/RE)
    pub fn set_func_pins(
        &mut self,
        command_latch: bool,
        address_latch: bool,
        write_enable: bool,
        read_enable: bool,
    ) {
        // positive logic
        self.cle.into_push_pull_output_in_state(if command_latch {
            bsp::hal::gpio::PinState::High
        } else {
            bsp::hal::gpio::PinState::Low
        });
        self.ale.into_push_pull_output_in_state(if address_latch {
            bsp::hal::gpio::PinState::High
        } else {
            bsp::hal::gpio::PinState::Low
        });

        // negative logic
        self.web.into_push_pull_output_in_state(if write_enable {
            bsp::hal::gpio::PinState::Low
        } else {
            bsp::hal::gpio::PinState::High
        });
        self.reb.into_push_pull_output_in_state(if read_enable {
            bsp::hal::gpio::PinState::Low
        } else {
            bsp::hal::gpio::PinState::High
        });
    }

    /// Clear Function Pins
    pub fn clear_func_pins(&mut self) {
        self.set_func_pins(false, false, false, false);
    }

    /// Command Input
    /// command: command data
    /// delay_f: delay function
    pub fn input_command<F: FnMut()>(&mut self, command: u8, delay_f: Option<F>) {
        // set data
        self.set_io_pin_data(command);

        // latch data
        // CLE=H, ALE=L, /WE=L->H, /RE=H

        // set
        self.set_func_pins(true, false, false, false);
        if let Some(mut f) = delay_f {
            f();
        }

        // latch
        self.set_func_pins(true, false, true, false);
        if let Some(mut f) = delay_f {
            f();
        }

        // clear
        self.clear_func_pins();
        if let Some(mut f) = delay_f {
            f();
        }
    }

    pub fn input_address<F: FnMut()>(&mut self, address_inputs: &[u8], delay_f: Option<F>) {
        // set data
        for (index, address) in address_inputs.iter().enumerate() {
            // set address[index]
            self.set_io_pin_data(*address);

            // latch data
            // CLE=L, ALE=H, /WE=L->H, /RE=H

            // set
            self.set_func_pins(false, true, false, false);
            if let Some(mut f) = delay_f {
                f();
            }

            // latch
            self.set_func_pins(false, true, true, false);
            if let Some(mut f) = delay_f {
                f();
            }

            // /WE=H->Lは次cycのData set時に行う
        }

        // clear
        self.clear_func_pins();
        if let Some(mut f) = delay_f {
            f();
        }
    }

    /// Data Input
    /// data_inputs: data inputs
    /// delay_f: delay function
    fn input_data<F: FnMut()>(&mut self, data_inputs: &[u8], delay_f: Option<F>) {
        // set datas
        for (index, data) in data_inputs.iter().enumerate() {
            // set data[index]
            self.set_io_pin_data(*data);

            // latch data
            // CLE=L, ALE=L, /WE=L->H, /RE=H

            // set
            self.set_func_pins(false, false, false, false);
            if let Some(mut f) = delay_f {
                f();
            }

            // latch
            self.set_func_pins(false, false, true, false);
            if let Some(mut f) = delay_f {
                f();
            }

            // /WE=H->Lは次cycのData set時に行う
        }

        // clear
        self.clear_func_pins();
        if let Some(mut f) = delay_f {
            f();
        }
    }

    /// Data Output
    /// output_data_buf: output data buffer, read data will be stored in this buffer
    /// delay_f: delay function
    pub fn output_data<'a, F: FnMut()>(
        &mut self,
        output_data_buf: &'a mut [u8],
        read_bytes: usize,
        delay_f: Option<F>,
    ) -> &'a [u8] {
        for (index, data) in output_data_buf.iter_mut().enumerate() {
            // slice.len() > read_bytes
            if index >= read_bytes {
                break;
            }
            // CLE=L, ALE=L, /WE=L, /RE=H->L

            // data output from ic
            self.set_func_pins(false, false, false, true);
            if let Some(mut f) = delay_f {
                f();
            }

            // capture & parse data bits
            *data = self.get_io_pin_data();

            // RE
            self.set_func_pins(false, false, false, false);
            if let Some(mut f) = delay_f {
                f();
            }
        }

        // return read data
        output_data_buf[0..read_bytes].as_ref()

        // 最終ループで/RE=Hになっている
    }
}

/////////////////////////////////////////////////////////////////////////////////////////////
/// Exec ID Read Operation
fn id_read(
    nandio_pins: &mut NandIoPins,
    delay: &mut cortex_m::delay::Delay,
    cs_index: u32,
) -> [u8; 5] {
    let id_read_size: usize = 5;
    let mut id_read_results = [0x00, 0x00, 0x00, 0x00, 0x00];

    // assert CS
    nandio_pins.assert_cs(cs_index);

    // command latch 0x90 (ID Read)
    nandio_pins.input_command(0x90, Some(|| delay.delay_ms(1)));
    // Address latch 0x00
    nandio_pins.input_address(&[0x00], Some(|| delay.delay_ms(1)));
    // Exec ID Read (read 5 bytes)
    nandio_pins.output_data(
        &mut id_read_results,
        id_read_size,
        Some(|| delay.delay_ms(1)),
    );

    // deassert CS
    nandio_pins.deassert_cs();

    id_read_results
}

#[entry]
fn main() -> ! {
    info!("Program start");
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
    let mut nandio_pins = NandIoPins {
        io0: &mut pins
            .gpio0
            .into_push_pull_output_in_state(bsp::hal::gpio::PinState::Low),
        io1: &mut pins
            .gpio1
            .into_push_pull_output_in_state(bsp::hal::gpio::PinState::Low),
        io2: &mut pins
            .gpio2
            .into_push_pull_output_in_state(bsp::hal::gpio::PinState::Low),
        io3: &mut pins
            .gpio3
            .into_push_pull_output_in_state(bsp::hal::gpio::PinState::Low),
        io4: &mut pins
            .gpio4
            .into_push_pull_output_in_state(bsp::hal::gpio::PinState::Low),
        io5: &mut pins
            .gpio5
            .into_push_pull_output_in_state(bsp::hal::gpio::PinState::Low),
        io6: &mut pins
            .gpio6
            .into_push_pull_output_in_state(bsp::hal::gpio::PinState::Low),
        io7: &mut pins
            .gpio7
            .into_push_pull_output_in_state(bsp::hal::gpio::PinState::Low),
        ceb0: &mut pins
            .gpio8
            .into_push_pull_output_in_state(bsp::hal::gpio::PinState::High),
        ceb1: &mut pins
            .gpio9
            .into_push_pull_output_in_state(bsp::hal::gpio::PinState::High),
        cle: &mut pins
            .gpio10
            .into_push_pull_output_in_state(bsp::hal::gpio::PinState::Low),
        ale: &mut pins
            .gpio11
            .into_push_pull_output_in_state(bsp::hal::gpio::PinState::Low),
        wpb: &mut pins
            .gpio12
            .into_push_pull_output_in_state(bsp::hal::gpio::PinState::Low),
        web: &mut pins
            .gpio13
            .into_push_pull_output_in_state(bsp::hal::gpio::PinState::High),
        reb: &mut pins
            .gpio14
            .into_push_pull_output_in_state(bsp::hal::gpio::PinState::High),
        rbb: &mut pins.gpio15.into_pull_up_input(),
    };

    for cs_index in 0..2 {
        info!("ID Read Test: CS={}", cs_index);
        let read_id_results = id_read(&mut nandio_pins, &mut delay, cs_index);

        // check ID
        if read_id_results[0] == 0x98
            && read_id_results[1] == 0xF1
            && read_id_results[2] == 0x80
            && read_id_results[3] == 0x15
            && read_id_results[4] == 0x72
        {
            info!("ID Read Success CS={}", cs_index);
        } else {
            info!("ID Read Fail CS={}", cs_index);
        }
    }

    loop {
        led_pin.set_high().unwrap();
        delay.delay_ms(1000);
        led_pin.set_low().unwrap();
        delay.delay_ms(1000);
    }
}

// End of file
