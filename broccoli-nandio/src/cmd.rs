#![allow(unused, dead_code)]
#![cfg_attr(not(test), no_std)]

use crate::pins::NandIoPins;

/// Delay for command latch
/// t_XXX worst (w/o t_RST) = 100ns
pub const DELAY_US_FOR_COMMAND_LATCH: u32 = 1;
/// Delay for reset
/// t_RST = ~500us
pub const DELAY_US_FOR_RESET: u32 = 500;
/// ID read bytes
pub const ID_READ_CMD_BYTES: usize = 5;
/// ID read expect data
pub const ID_READ_EXPECT_DATA: [u8; 5] = [0x98, 0xF1, 0x80, 0x15, 0x72];

/// NAND IC Command ID
#[repr(u8)]
pub enum CommandId {
    Reset = 0xff,
    IdRead = 0x90,
    StatusRead = 0x70,
    ReadFirst = 0x00,
    ReadSecond = 0x30,
    AutoPageProgramFirst = 0x80,
    AutoPageProgramSecond = 0x10,
    AutoBlockEraseFirst = 0x60,
    AutoBlockEraseSecond = 0xd0,
}
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
        self.nandio_pins.input_command(CommandId::Reset as u8, || {
            self.delay.delay_us(DELAY_US_FOR_COMMAND_LATCH)
        });
        self.nandio_pins.deassert_cs();
        self.delay.delay_us(DELAY_US_FOR_RESET);
    }

    /// Read NAND IC ID by fw
    pub fn id_read(&mut self, cs_index: u32) -> (bool, [u8; 5]) {
        let mut id_read_results = [0x00, 0x00, 0x00, 0x00, 0x00];

        self.nandio_pins.assert_cs(cs_index);
        self.nandio_pins.input_command(CommandId::IdRead as u8, || {
            self.delay.delay_us(DELAY_US_FOR_COMMAND_LATCH)
        });
        self.nandio_pins
            .input_address(&[0x00], || self.delay.delay_us(DELAY_US_FOR_COMMAND_LATCH));
        self.nandio_pins
            .output_data(&mut id_read_results, ID_READ_CMD_BYTES, || {
                self.delay.delay_us(DELAY_US_FOR_COMMAND_LATCH)
            });
        self.nandio_pins.deassert_cs();

        (id_read_results == ID_READ_EXPECT_DATA, id_read_results)
    }
}
