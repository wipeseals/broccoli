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

#[allow(dead_code)]
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

        trace!("Init All pins");
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

        trace!("Set IO Pin Dir: 0x{:02X}", data);
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
    pub fn assert_cs(&mut self, cs_index: u32) {
        match cs_index {
            0 => {
                self.ceb0.set_state(bsp::hal::gpio::PinState::Low).unwrap();
                self.ceb1.set_state(bsp::hal::gpio::PinState::High).unwrap();
            }
            1 => {
                self.ceb0.set_state(bsp::hal::gpio::PinState::High).unwrap();
                self.ceb1.set_state(bsp::hal::gpio::PinState::Low).unwrap();
            }
            _ => {
                crate::panic!("Invalid CS index")
            }
        }
        trace!("Assert CS: 0x{:02X}", cs_index);
    }

    /// deassert CS
    pub fn deassert_cs(&mut self) {
        self.ceb0.set_state(bsp::hal::gpio::PinState::High).unwrap();
        self.ceb1.set_state(bsp::hal::gpio::PinState::High).unwrap();

        trace!("Deassert CS");
    }

    /// Wait for busy
    /// delay_f: delay function
    /// timeout: timeout value
    /// return: true if busy is low, false if timeout
    pub fn wait_for_busy<F: FnMut()>(&mut self, mut delay_f: F, retry_count: u32) -> bool {
        let mut busy: bool = self.rbb.is_high().unwrap();
        let mut count: u32 = 0;
        while busy {
            count += 1;
            // timeout
            if count >= retry_count {
                warn!("Wait for Busy: Timeout: {}", count);
                return false;
            }
            delay_f();
            busy = self.rbb.is_high().unwrap();
        }
        trace!("Wait for Busy: count: {}", count);
        true
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

        trace!(
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

        trace!("Command Input[{}]: 0x{:02X}", 0, command);
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

            trace!("Address Input[{}]: 0x{:02X}", index, *address);
        }
    }

    /// Data Input
    /// data_inputs: data inputs
    /// delay_f: delay function
    fn input_data<F: FnMut()>(&mut self, data_inputs: &[u8], mut delay_f: F) {
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

            trace!("Data Input[{}]: 0x{:02X}", index, *data);
        }
    }

    /// Data Output
    /// output_data_buf: output data buffer, read data will be stored in this buffer
    /// delay_f: delay function
    pub fn output_data<'a, F: FnMut()>(
        &mut self,
        output_data_buf: &'a mut [u8],
        read_bytes: usize,
        mut delay_f: F,
    ) -> &'a [u8] {
        // read datas
        for (index, data) in output_data_buf.iter_mut().enumerate() {
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

            trace!("Data Output[{}]: 0x{:02X}", index, *data);
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

    // initialize
    nandio_pins.init_all_pin();

    // command latch 0xff (Reset)
    nandio_pins.assert_cs(cs_index);
    nandio_pins.input_command(0xff, || delay.delay_ms(1));
    nandio_pins.deassert_cs();
    delay.delay_ms(10);

    // command latch 0x90 (ID Read)
    // Address latch 0x00
    // Exec ID Read (read 5 bytes)
    nandio_pins.assert_cs(cs_index);
    nandio_pins.input_command(0x90, || delay.delay_ms(1));
    nandio_pins.input_address(&[0x00], || delay.delay_ms(1));
    nandio_pins.output_data(&mut id_read_results, id_read_size, || delay.delay_ms(1));
    nandio_pins.deassert_cs();

    // finalize
    nandio_pins.init_all_pin();

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
    let mut nandio_pins = NandIoPins {
        io0: &mut pins.gpio0.into_push_pull_output(),
        io1: &mut pins.gpio1.into_push_pull_output(),
        io2: &mut pins.gpio2.into_push_pull_output(),
        io3: &mut pins.gpio3.into_push_pull_output(),
        io4: &mut pins.gpio4.into_push_pull_output(),
        io5: &mut pins.gpio5.into_push_pull_output(),
        io6: &mut pins.gpio6.into_push_pull_output(),
        io7: &mut pins.gpio7.into_push_pull_output(),
        ceb0: &mut pins.gpio8.into_push_pull_output(),
        ceb1: &mut pins.gpio9.into_push_pull_output(),
        cle: &mut pins.gpio10.into_push_pull_output(),
        ale: &mut pins.gpio11.into_push_pull_output(),
        wpb: &mut pins.gpio12.into_push_pull_output(),
        web: &mut pins.gpio13.into_push_pull_output(),
        reb: &mut pins.gpio14.into_push_pull_output(),
        rbb: &mut pins.gpio15.into_pull_up_input(),
    };

    for cs_index in 0..2 {
        let read_id_results = id_read(&mut nandio_pins, &mut delay, cs_index);

        // check ID
        if read_id_results[0] == 0x98
            && read_id_results[1] == 0xF1
            && read_id_results[2] == 0x80
            && read_id_results[3] == 0x15
            && read_id_results[4] == 0x72
        {
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

// End of file
