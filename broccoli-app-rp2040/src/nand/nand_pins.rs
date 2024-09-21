use broccoli_core::nand::io_driver::NandIoError;
use cortex_m::delay;
use embassy_rp::gpio::{Flex, Input, Output};
use embassy_rp::gpio::{Level, Pull};
use embassy_rp::Peripherals;
use embassy_time::Timer;
use embedded_hal_1::digital::OutputPin;

use embedded_hal_1::digital::PinState;

/// NAND I/O Pins for RP2040 (JISC-SSD)
pub struct NandIoPins<'d> {
    /// I/O Pins
    io0: Flex<'d>,

    io1: Flex<'d>,
    io2: Flex<'d>,
    io3: Flex<'d>,
    io4: Flex<'d>,
    io5: Flex<'d>,
    io6: Flex<'d>,
    io7: Flex<'d>,
    ceb0: Output<'d>,
    ceb1: Output<'d>,
    cle: Output<'d>,
    ale: Output<'d>,
    wpb: Output<'d>,
    web: Output<'d>,
    reb: Output<'d>,
    rbb: Input<'d>,
}

impl<'p> NandIoPins<'p> {
    pub fn new(
        io0: Flex<'p>,
        io1: Flex<'p>,
        io2: Flex<'p>,
        io3: Flex<'p>,
        io4: Flex<'p>,
        io5: Flex<'p>,
        io6: Flex<'p>,
        io7: Flex<'p>,
        ceb0: Output<'p>,
        ceb1: Output<'p>,
        cle: Output<'p>,
        ale: Output<'p>,
        wpb: Output<'p>,
        web: Output<'p>,
        reb: Output<'p>,
        rbb: Input<'p>,
    ) -> Self {
        Self {
            io0,
            io1,
            io2,
            io3,
            io4,
            io5,
            io6,
            io7,
            ceb0,
            ceb1,
            cle,
            ale,
            wpb,
            web,
            reb,
            rbb,
        }
    }

    /// Init NAND I/O Pins
    pub async fn setup(&mut self) {
        // bidirectional. default: output, low
        self.set_data_dir(true);
        self.set_data(0x00);

        // output
        self.deassert_cs();
        self.set_func_pins(false, false, false, false);
        self.set_write_protect(false);

        crate::trace!("Init All pins");
    }

    /// Set PinDir
    /// pin_dir: pin direction. 00: input, 01: output
    pub async fn set_data_dir(&mut self, is_output: bool) {
        if is_output {
            self.io0.set_as_output();
            self.io1.set_as_output();
            self.io2.set_as_output();
            self.io3.set_as_output();
            self.io4.set_as_output();
            self.io5.set_as_output();
            self.io6.set_as_output();
            self.io7.set_as_output();
        } else {
            self.io0.set_as_input();
            self.io1.set_as_input();
            self.io2.set_as_input();
            self.io3.set_as_input();
            self.io4.set_as_input();
            self.io5.set_as_input();
            self.io6.set_as_input();
            self.io7.set_as_input();
        }

        crate::trace!("Set IO Pin Dir: {}", is_output);
    }

    /// Set data
    /// data: write data to IO pins
    pub async fn set_data(&mut self, data: u8) {
        self.io0
            .set_state(PinState::from(data & 0x01 != 0))
            .unwrap();
        self.io1
            .set_state(PinState::from(data & 0x02 != 0))
            .unwrap();
        self.io2
            .set_state(PinState::from(data & 0x04 != 0))
            .unwrap();
        self.io3
            .set_state(PinState::from(data & 0x08 != 0))
            .unwrap();
        self.io4
            .set_state(PinState::from(data & 0x10 != 0))
            .unwrap();
        self.io5
            .set_state(PinState::from(data & 0x20 != 0))
            .unwrap();
        self.io6
            .set_state(PinState::from(data & 0x40 != 0))
            .unwrap();
        self.io7
            .set_state(PinState::from(data & 0x80 != 0))
            .unwrap();
    }

    /// Get data
    /// return: read data from IO pins
    pub async fn get_data(&mut self) -> u8 {
        let mut data: u8 = 0;
        data |= if self.io0.is_high() { 0x01 } else { 0x00 };
        data |= if self.io1.is_high() { 0x02 } else { 0x00 };
        data |= if self.io2.is_high() { 0x04 } else { 0x00 };
        data |= if self.io3.is_high() { 0x08 } else { 0x00 };
        data |= if self.io4.is_high() { 0x10 } else { 0x00 };
        data |= if self.io5.is_high() { 0x20 } else { 0x00 };
        data |= if self.io6.is_high() { 0x40 } else { 0x00 };
        data |= if self.io7.is_high() { 0x80 } else { 0x00 };
        data
    }

    /// set CS
    /// cs_index: chip select index
    pub async fn assert_cs(&mut self, cs_index: usize) {
        match cs_index {
            0 => {
                self.ceb0.set_state(PinState::Low).unwrap();
                self.ceb1.set_state(PinState::High).unwrap();
            }
            1 => {
                self.ceb0.set_state(PinState::High).unwrap();
                self.ceb1.set_state(PinState::Low).unwrap();
            }
            _ => crate::unreachable!("Invalid CS index"),
        }
        crate::trace!("Assert CS: 0x{:02X}", cs_index);
    }

    /// deassert CS
    pub async fn deassert_cs(&mut self) {
        self.ceb0.set_state(PinState::High).unwrap();
        self.ceb1.set_state(PinState::High).unwrap();

        crate::trace!("Deassert CS");
    }

    /// Wait for busy
    pub async fn wait_for_busy(
        &mut self,
        poll_us: u64,
        timeout_us: u64,
    ) -> Result<(), NandIoError> {
        let mut busy: bool = self.rbb.is_low();
        let mut total_wait_us: u64 = 0;
        while busy {
            // timeout
            if total_wait_us >= timeout_us {
                crate::warn!("Wait for Busy: timeout");
                return Err(NandIoError::Timeout);
            }

            // polling
            Timer::after_micros(poll_us).await;
            total_wait_us += poll_us;
            busy = self.rbb.is_low();
        }
        crate::trace!("Wait for Busy: total_wait_us: {}", total_wait_us);
        Ok(())
    }

    /// Set Write Protect Enable
    /// enable: write protect enable
    pub async fn set_write_protect(&mut self, enable: bool) {
        // /WPなのでEnalbe時にLow
        self.wpb.set_state(PinState::from(!enable)).unwrap();
        crate::trace!("Set Write Protect Enable: {}", enable);
    }

    /// Set Function Pins
    /// command_latch: command latch (CLE)
    /// address_latch: address latch (ALE)
    /// write_enable: write enable (/WE)
    /// read_enable: read enable (/RE)
    pub async fn set_func_pins(
        &mut self,
        command_latch: bool,
        address_latch: bool,
        write_enable: bool,
        read_enable: bool,
    ) {
        // positive logic
        self.cle.set_state(PinState::from(command_latch)).unwrap();
        self.ale.set_state(PinState::from(address_latch)).unwrap();

        // negative logic
        self.web.set_state(PinState::from(!write_enable)).unwrap();
        self.reb.set_state(PinState::from(!read_enable)).unwrap();

        crate::trace!(
            "Set Func Pins: CLE={}, ALE={}, /WE={}, /RE={}",
            command_latch,
            address_latch,
            write_enable,
            read_enable
        )
    }

    /// Clear Function Pins
    pub async fn reset_func_pins(&mut self) {
        self.set_func_pins(false, false, false, false);
    }

    /// Command Input
    /// command: command data
    pub async fn input_command(&mut self, command: u8, delay_us: u64) {
        // latch data
        // CLE=H, ALE=L, /WE=L->H, /RE=H

        // set
        self.set_data_dir(true);
        self.set_data(command);
        self.set_func_pins(true, false, false, false);
        Timer::after_micros(delay_us).await;

        // latch
        self.set_func_pins(true, false, true, false);
        Timer::after_micros(delay_us).await;

        crate::trace!("Command Input[{}]: 0x{:02X}", 0, command);
    }

    /// Address Input
    /// address_inputs: address inputs
    pub async fn input_address(&mut self, address_inputs: &[u8], delay_us: u64) {
        for (index, address) in address_inputs.iter().enumerate() {
            // latch data
            // CLE=L, ALE=H, /WE=L->H, /RE=H

            // set
            self.set_data_dir(true);
            self.set_data(*address);
            self.set_func_pins(false, true, false, false);
            Timer::after_micros(delay_us).await;

            // latch
            self.set_func_pins(false, true, true, false);
            Timer::after_micros(delay_us).await;

            crate::trace!("Address Input[{}]: 0x{:02X}", index, *address);
        }
    }

    /// Data Input
    /// data_inputs: data inputs
    pub async fn write_data(&mut self, data_inputs: &[u8], delay_us: u64) {
        // set datas
        for (index, data) in data_inputs.iter().enumerate() {
            // latch data
            // CLE=L, ALE=L, /WE=L->H, /RE=H

            // set
            self.set_data_dir(true);
            self.set_data(*data);
            self.set_func_pins(false, false, false, false);
            Timer::after_micros(delay_us).await;

            // latch
            self.set_func_pins(false, false, true, false);
            Timer::after_micros(delay_us).await;

            crate::trace!("Data Input[{}]: 0x{:02X}", index, *data);
        }
    }

    /// Data Output
    /// output_data_buf: output data buffer, read data will be stored in this buffer
    pub async fn read_data<'d>(
        &mut self,
        output_data_ref: &'d mut [u8],
        read_bytes: usize,
        delay_us: u64,
    ) {
        // read datas
        for (index, data) in output_data_ref.iter_mut().enumerate() {
            // slice.len() > read_bytes
            if index >= read_bytes {
                break;
            }

            // data output from ic
            // CLE=L, ALE=L, /WE=L, /RE=H->L

            self.set_data_dir(false);
            self.set_func_pins(false, false, false, true);
            Timer::after_micros(delay_us).await;

            // capture & parse data bits
            *data = self.get_data().await;

            // RE
            self.set_func_pins(false, false, false, false);
            Timer::after_micros(delay_us).await;

            crate::trace!("Data Output[{}]: 0x{:02X}", index, *data);
        }
    }
}
