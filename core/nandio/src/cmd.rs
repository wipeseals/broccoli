#![allow(unused, dead_code)]
#![cfg_attr(not(test), no_std)]

use crate::pins::NandIoPins;

/// NAND IC Command Driver
pub struct Command<'a> {
    pub nandio_pins: &'a mut NandIoPins<'a>,
    pub delay: &'a mut cortex_m::delay::Delay,
}
impl Command<'_> {
    /// Initialize all pins by fw
    pub fn init_pins(&mut self) {
        self.nandio_pins.init_all_pin();
    }

    /// Reset NAND IC by fw
    pub fn reset(&mut self, cs_index: u32) {
        self.nandio_pins.assert_cs(cs_index);
        self.nandio_pins
            .input_command(0xff, || self.delay.delay_us(1)); // t_XXX worst (w/o t_RST) = 100ns
        self.nandio_pins.deassert_cs();
        self.delay.delay_us(500); // t_RST = ~500us
    }

    /// Read NAND IC ID by fw
    pub fn id_read(&mut self, cs_index: u32) -> (bool, [u8; 5]) {
        let id_read_size: usize = 5;
        let id_read_expect_data = [0x98, 0xF1, 0x80, 0x15, 0x72];
        let mut id_read_results = [0x00, 0x00, 0x00, 0x00, 0x00];

        self.nandio_pins.assert_cs(cs_index);
        self.nandio_pins
            .input_command(0x90, || self.delay.delay_us(1));
        self.nandio_pins
            .input_address(&[0x00], || self.delay.delay_us(1));
        self.nandio_pins
            .output_data(&mut id_read_results, id_read_size, || {
                self.delay.delay_us(1)
            });
        self.nandio_pins.deassert_cs();

        (id_read_results == id_read_expect_data, id_read_results)
    }
}
