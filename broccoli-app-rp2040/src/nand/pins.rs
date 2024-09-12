#![allow(unused, dead_code)]
#![cfg_attr(not(test), no_std)]

use defmt::*;
use defmt_rtt as _;
use embedded_hal::digital::v2::{InputPin, OutputPin};
use panic_probe as _;

use rp_pico as bsp;

/// Error Type
pub enum Error {
    Common,
    Timeout,
}
/// NAND I/O Pins
pub struct NandIoPins<'a> {
    pub io0: &'a mut bsp::hal::gpio::Pin<
        bsp::hal::gpio::bank0::Gpio0,
        bsp::hal::gpio::FunctionSioOutput,
        bsp::hal::gpio::PullDown,
    >,
    pub io1: &'a mut bsp::hal::gpio::Pin<
        bsp::hal::gpio::bank0::Gpio1,
        bsp::hal::gpio::FunctionSioOutput,
        bsp::hal::gpio::PullDown,
    >,
    pub io2: &'a mut bsp::hal::gpio::Pin<
        bsp::hal::gpio::bank0::Gpio2,
        bsp::hal::gpio::FunctionSioOutput,
        bsp::hal::gpio::PullDown,
    >,
    pub io3: &'a mut bsp::hal::gpio::Pin<
        bsp::hal::gpio::bank0::Gpio3,
        bsp::hal::gpio::FunctionSioOutput,
        bsp::hal::gpio::PullDown,
    >,
    pub io4: &'a mut bsp::hal::gpio::Pin<
        bsp::hal::gpio::bank0::Gpio4,
        bsp::hal::gpio::FunctionSioOutput,
        bsp::hal::gpio::PullDown,
    >,
    pub io5: &'a mut bsp::hal::gpio::Pin<
        bsp::hal::gpio::bank0::Gpio5,
        bsp::hal::gpio::FunctionSioOutput,
        bsp::hal::gpio::PullDown,
    >,
    pub io6: &'a mut bsp::hal::gpio::Pin<
        bsp::hal::gpio::bank0::Gpio6,
        bsp::hal::gpio::FunctionSioOutput,
        bsp::hal::gpio::PullDown,
    >,
    pub io7: &'a mut bsp::hal::gpio::Pin<
        bsp::hal::gpio::bank0::Gpio7,
        bsp::hal::gpio::FunctionSioOutput,
        bsp::hal::gpio::PullDown,
    >,
    pub ceb0: &'a mut bsp::hal::gpio::Pin<
        bsp::hal::gpio::bank0::Gpio8,
        bsp::hal::gpio::FunctionSioOutput,
        bsp::hal::gpio::PullDown,
    >,
    pub ceb1: &'a mut bsp::hal::gpio::Pin<
        bsp::hal::gpio::bank0::Gpio9,
        bsp::hal::gpio::FunctionSioOutput,
        bsp::hal::gpio::PullDown,
    >,
    pub cle: &'a mut bsp::hal::gpio::Pin<
        bsp::hal::gpio::bank0::Gpio10,
        bsp::hal::gpio::FunctionSioOutput,
        bsp::hal::gpio::PullDown,
    >,
    pub ale: &'a mut bsp::hal::gpio::Pin<
        bsp::hal::gpio::bank0::Gpio11,
        bsp::hal::gpio::FunctionSioOutput,
        bsp::hal::gpio::PullDown,
    >,
    pub wpb: &'a mut bsp::hal::gpio::Pin<
        bsp::hal::gpio::bank0::Gpio12,
        bsp::hal::gpio::FunctionSioOutput,
        bsp::hal::gpio::PullDown,
    >,
    pub web: &'a mut bsp::hal::gpio::Pin<
        bsp::hal::gpio::bank0::Gpio13,
        bsp::hal::gpio::FunctionSioOutput,
        bsp::hal::gpio::PullDown,
    >,
    pub reb: &'a mut bsp::hal::gpio::Pin<
        bsp::hal::gpio::bank0::Gpio14,
        bsp::hal::gpio::FunctionSioOutput,
        bsp::hal::gpio::PullDown,
    >,
    pub rbb: &'a mut bsp::hal::gpio::Pin<
        bsp::hal::gpio::bank0::Gpio15,
        bsp::hal::gpio::FunctionSioInput,
        bsp::hal::gpio::PullUp,
    >,
}

/// pin initialization macro
#[macro_export]
macro_rules! init_nandio_pins {
    ($pins:expr) => {
        NandIoPins {
            io0: &mut $pins.gpio0.into_push_pull_output(),
            io1: &mut $pins.gpio1.into_push_pull_output(),
            io2: &mut $pins.gpio2.into_push_pull_output(),
            io3: &mut $pins.gpio3.into_push_pull_output(),
            io4: &mut $pins.gpio4.into_push_pull_output(),
            io5: &mut $pins.gpio5.into_push_pull_output(),
            io6: &mut $pins.gpio6.into_push_pull_output(),
            io7: &mut $pins.gpio7.into_push_pull_output(),
            ceb0: &mut $pins.gpio8.into_push_pull_output(),
            ceb1: &mut $pins.gpio9.into_push_pull_output(),
            cle: &mut $pins.gpio10.into_push_pull_output(),
            ale: &mut $pins.gpio11.into_push_pull_output(),
            wpb: &mut $pins.gpio12.into_push_pull_output(),
            web: &mut $pins.gpio13.into_push_pull_output(),
            reb: &mut $pins.gpio14.into_push_pull_output(),
            rbb: &mut $pins.gpio15.into_pull_up_input(),
        }
    };
}

impl NandIoPins<'_> {
    /// Init NAND I/O Pins
    pub fn init_all_pin(&mut self) {
        // bidirectional. default: output, low
        self.set_io_pin_dir(0xff);
        self.set_io_pin_data(0x00);

        // output
        self.ceb0.set_input_enable(false);
        self.ceb1.set_input_enable(false);
        self.deassert_cs();
        self.ceb0.set_output_disable(false);
        self.ceb1.set_output_disable(false);

        self.cle.set_input_enable(false);
        self.ale.set_input_enable(false);
        self.wpb.set_input_enable(false);
        self.web.set_input_enable(false);
        self.reb.set_input_enable(false);
        self.set_func_pins(false, false, false, false);
        self.cle.set_output_disable(false);
        self.ale.set_output_disable(false);
        self.wpb.set_output_disable(false);
        self.web.set_output_disable(false);
        self.reb.set_output_disable(false);

        // input
        self.rbb.set_input_enable(true);
        self.rbb.set_output_disable(true);

        defmt::trace!("Init All pins");
    }

    /// Set PinDir
    /// pin_dir: pin direction. 00: input, 01: output
    pub fn set_io_pin_dir(&mut self, data: u8) {
        // set output enable
        self.io0.set_output_disable((data & 0x01) == 0);
        self.io1.set_output_disable((data & 0x02) == 0);
        self.io2.set_output_disable((data & 0x04) == 0);
        self.io3.set_output_disable((data & 0x08) == 0);
        self.io4.set_output_disable((data & 0x10) == 0);
        self.io5.set_output_disable((data & 0x20) == 0);
        self.io6.set_output_disable((data & 0x40) == 0);
        self.io7.set_output_disable((data & 0x80) == 0);
        // set input enable
        self.io0.set_input_enable((data & 0x01) == 0);
        self.io1.set_input_enable((data & 0x02) == 0);
        self.io2.set_input_enable((data & 0x04) == 0);
        self.io3.set_input_enable((data & 0x08) == 0);
        self.io4.set_input_enable((data & 0x10) == 0);
        self.io5.set_input_enable((data & 0x20) == 0);
        self.io6.set_input_enable((data & 0x40) == 0);
        self.io7.set_input_enable((data & 0x80) == 0);

        defmt::trace!("Set IO Pin Dir: 0x{:02X}", data);
    }

    /// Set data
    /// data: write data to IO pins
    pub fn set_io_pin_data(&mut self, data: u8) {
        self.io0
            .set_state(bsp::hal::gpio::PinState::from(data & 0x01 != 0))
            .unwrap();
        self.io1
            .set_state(bsp::hal::gpio::PinState::from(data & 0x02 != 0))
            .unwrap();
        self.io2
            .set_state(bsp::hal::gpio::PinState::from(data & 0x04 != 0))
            .unwrap();
        self.io3
            .set_state(bsp::hal::gpio::PinState::from(data & 0x08 != 0))
            .unwrap();
        self.io4
            .set_state(bsp::hal::gpio::PinState::from(data & 0x10 != 0))
            .unwrap();
        self.io5
            .set_state(bsp::hal::gpio::PinState::from(data & 0x20 != 0))
            .unwrap();
        self.io6
            .set_state(bsp::hal::gpio::PinState::from(data & 0x40 != 0))
            .unwrap();
        self.io7
            .set_state(bsp::hal::gpio::PinState::from(data & 0x80 != 0))
            .unwrap();
    }

    /// Get data
    /// return: read data from IO pins
    pub fn get_io_pin_data(&mut self) -> u8 {
        let mut data: u8 = 0;
        data |= if self.io0.is_high().unwrap() {
            0x01
        } else {
            0x00
        };
        data |= if self.io1.is_high().unwrap() {
            0x02
        } else {
            0x00
        };
        data |= if self.io2.is_high().unwrap() {
            0x04
        } else {
            0x00
        };
        data |= if self.io3.is_high().unwrap() {
            0x08
        } else {
            0x00
        };
        data |= if self.io4.is_high().unwrap() {
            0x10
        } else {
            0x00
        };
        data |= if self.io5.is_high().unwrap() {
            0x20
        } else {
            0x00
        };
        data |= if self.io6.is_high().unwrap() {
            0x40
        } else {
            0x00
        };
        data |= if self.io7.is_high().unwrap() {
            0x80
        } else {
            0x00
        };
        data
    }

    /// set CS
    /// cs_index: chip select index
    pub fn assert_cs(&mut self, cs_index: usize) {
        match cs_index {
            0 => {
                self.ceb0.set_state(bsp::hal::gpio::PinState::Low).unwrap();
                self.ceb1.set_state(bsp::hal::gpio::PinState::High).unwrap();
            }
            1 => {
                self.ceb0.set_state(bsp::hal::gpio::PinState::High).unwrap();
                self.ceb1.set_state(bsp::hal::gpio::PinState::Low).unwrap();
            }
            _ => defmt::unreachable!("Invalid CS index"),
        }
        defmt::trace!("Assert CS: 0x{:02X}", cs_index);
    }

    /// deassert CS
    pub fn deassert_cs(&mut self) {
        self.ceb0.set_state(bsp::hal::gpio::PinState::High).unwrap();
        self.ceb1.set_state(bsp::hal::gpio::PinState::High).unwrap();

        defmt::trace!("Deassert CS");
    }

    /// Wait for busy
    /// delay_f: delay function
    /// timeout: timeout value
    /// return: true if busy is low, false if timeout
    pub fn wait_for_busy<F: FnMut()>(
        &mut self,
        mut delay_f: F,
        retry_count: u32,
    ) -> Result<(), Error> {
        let mut busy: bool = self.rbb.is_low().unwrap();
        let mut count: u32 = 0;
        while busy {
            count += 1;
            // timeout
            if count >= retry_count {
                defmt::warn!("Wait for Busy: Timeout: {}", count);
                return Err(Error::Timeout);
            }
            delay_f();
            busy = self.rbb.is_low().unwrap();
        }
        defmt::trace!("Wait for Busy: count: {}", count);
        Ok(())
    }

    /// Set Write Protect Enable
    /// enable: write protect enable
    pub fn set_write_protect_enable(&mut self, enable: bool) {
        // /WPなのでEnalbe時にLow
        self.wpb
            .set_state(bsp::hal::gpio::PinState::from(!enable))
            .unwrap();
        defmt::trace!("Set Write Protect Enable: {}", enable);
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
        self.cle
            .set_state(bsp::hal::gpio::PinState::from(command_latch))
            .unwrap();
        self.ale
            .set_state(bsp::hal::gpio::PinState::from(address_latch))
            .unwrap();

        // negative logic
        self.web
            .set_state(bsp::hal::gpio::PinState::from(!write_enable))
            .unwrap();
        self.reb
            .set_state(bsp::hal::gpio::PinState::from(!read_enable))
            .unwrap();

        defmt::trace!(
            "Set Func Pins: CLE={}, ALE={}, /WE={}, /RE={}",
            command_latch,
            address_latch,
            write_enable,
            read_enable
        )
    }

    /// Clear Function Pins
    pub fn clear_func_pins(&mut self) {
        self.set_func_pins(false, false, false, false);
    }

    /// Command Input
    /// command: command data
    /// delay_f: delay function
    pub fn input_command<F: FnMut()>(&mut self, command: u8, mut delay_f: F) {
        // latch data
        // CLE=H, ALE=L, /WE=L->H, /RE=H

        // set
        self.set_io_pin_dir(0xff);
        self.set_io_pin_data(command);
        self.set_func_pins(true, false, false, false);
        delay_f();

        // latch
        self.set_func_pins(true, false, true, false);
        delay_f();

        defmt::trace!("Command Input[{}]: 0x{:02X}", 0, command);
    }

    /// Address Input
    /// address_inputs: address inputs
    /// delay_f: delay function
    pub fn input_address<F: FnMut()>(&mut self, address_inputs: &[u8], mut delay_f: F) {
        for (index, address) in address_inputs.iter().enumerate() {
            // latch data
            // CLE=L, ALE=H, /WE=L->H, /RE=H

            // set
            self.set_io_pin_dir(0xff);
            self.set_io_pin_data(*address);
            self.set_func_pins(false, true, false, false);
            delay_f();

            // latch
            self.set_func_pins(false, true, true, false);
            delay_f();

            defmt::trace!("Address Input[{}]: 0x{:02X}", index, *address);
        }
    }

    /// Data Input
    /// data_inputs: data inputs
    /// delay_f: delay function
    pub fn input_data<F: FnMut()>(&mut self, data_inputs: &[u8], mut delay_f: F) {
        // set datas
        for (index, data) in data_inputs.iter().enumerate() {
            // latch data
            // CLE=L, ALE=L, /WE=L->H, /RE=H

            // set
            self.set_io_pin_dir(0xff);
            self.set_io_pin_data(*data);
            self.set_func_pins(false, false, false, false);
            delay_f();

            // latch
            self.set_func_pins(false, false, true, false);
            delay_f();

            defmt::trace!("Data Input[{}]: 0x{:02X}", index, *data);
        }
    }

    /// Data Output
    /// output_data_buf: output data buffer, read data will be stored in this buffer
    /// delay_f: delay function
    pub fn output_data<'a, F: FnMut()>(
        &mut self,
        output_data_ref: &'a mut [u8],
        read_bytes: usize,
        mut delay_f: F,
    ) -> &'a [u8] {
        // read datas
        for (index, data) in output_data_ref.iter_mut().enumerate() {
            // slice.len() > read_bytes
            if index >= read_bytes {
                break;
            }

            // data output from ic
            // CLE=L, ALE=L, /WE=L, /RE=H->L

            self.set_io_pin_dir(0x00);
            self.set_func_pins(false, false, false, true);
            delay_f();

            // capture & parse data bits
            *data = self.get_io_pin_data();

            // RE
            self.set_func_pins(false, false, false, false);
            delay_f();

            defmt::trace!("Data Output[{}]: 0x{:02X}", index, *data);
        }

        // return read data
        output_data_ref[0..read_bytes].as_ref()
    }
}
