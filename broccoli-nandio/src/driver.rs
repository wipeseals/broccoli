#![allow(unused, dead_code)]
#![cfg_attr(not(test), no_std)]

use defmt::{trace, warn};

use crate::{address::Address, pins::NandIoPins};

/// Delay for command latch
/// t_XXX worst (w/o t_RST) = 100ns
pub const DELAY_US_FOR_COMMAND_LATCH: u32 = 1;

/// Delay for reset
/// t_RST = ~500us
pub const DELAY_US_FOR_RESET: u32 = 500;

/// Delay for wait busy (read)
/// t_R=25us,, t_DCBSYR1=25us, t_DCBSYR2=30us,
pub const DELAY_US_FOR_WAIT_BUSY_READ: u32 = 30;

/// Delay for wait busy (write)
/// t_PROG = 700us, t_DCBSYW2 = 700us
pub const DELAY_US_FOR_WAIT_BUSY_WRITE: u32 = 700;

/// Delay for wait busy (erase)
/// t_BERASE = 5ms (5,000us)
pub const DELAY_US_FOR_WAIT_BUSY_ERASE: u32 = 5000;

/// Check RBB (Ready/Busy) status
pub const RESOLUTION_COUNT_FOR_WAIT_BUSY: u32 = 10;

/// Retry count for wait busy
pub const RETRY_LIMIT_COUNT_FOR_WAIT_BUSY: u32 = 10 * RESOLUTION_COUNT_FOR_WAIT_BUSY;

/// ID read bytes
pub const ID_READ_CMD_BYTES: usize = 5;

/// ID read expect data
///
/// | Description            | Hex Data |
/// | ---------------------- | -------- |
/// | Maker Code             | 0x98     |
/// | Device Code            | 0xF1     |
/// | Chip Number, Cell Type | 0x80     |
/// | Page Size, Block Size  | 0x15     |
/// | District Number        | 0x72     |
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

/// NAND IC Status Output
///
/// | Bit | Description            | Value                      |
/// | --- | ---------------------- | -------------------------- |
/// | 0   | Chip Status0           | Pass:0 , Fail: 1           |
/// | 1   | Chip Status1           | Pass:0 , Fail: 1           |
/// | 2   | -                      | -                          |
/// | 3   | -                      | -                          |
/// | 4   | -                      | -                          |
/// | 5   | Page Buffer Ready/Busy | Ready: 1, Busy: 0          |
/// | 6   | Data Cache Ready/Busy  | Ready: 1, Busy: 0          |
/// | 7   | Write Protect          | Not Protect: 1, Protect: 0 |
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StatusOutput {
    pub data: u8,
}

impl StatusOutput {
    /// Check if chip is pass
    ///  - chip_num: 0 or 1
    pub fn is_pass(&self, chip_num: u32) -> bool {
        match chip_num {
            0 => self.data & 0b0000_0001 == 0,
            1 => self.data & 0b0000_0010 == 0,
            _ => core::unreachable!("Invalid chip number"),
        }
    }

    /// Check if page buffer is ready
    pub fn is_page_buffer_ready(&self) -> bool {
        self.data & 0b0010_0000 != 0
    }

    /// Check if data cache is ready
    pub fn is_data_cache_ready(&self) -> bool {
        self.data & 0b0100_0000 != 0
    }

    /// Check if write protect is enabled
    pub fn is_write_protect(&self) -> bool {
        self.data & 0b1000_0000 == 0
    }
}

pub enum Error {
    Common,
    Timeout,
}
/// NAND IC Command Driver
pub struct Driver<'a> {
    pub nandio_pins: &'a mut NandIoPins<'a>,
    pub delay: &'a mut cortex_m::delay::Delay,
}
impl Driver<'_> {
    /// Initialize all pins
    pub fn init_pins(&mut self) {
        self.nandio_pins.init_all_pin();
        trace!("Initialize all pins")
    }

    /// Reset NAND IC
    pub fn reset(&mut self, cs_index: u32) {
        self.nandio_pins.assert_cs(cs_index);
        self.nandio_pins.input_command(CommandId::Reset as u8, || {
            self.delay.delay_us(DELAY_US_FOR_COMMAND_LATCH)
        });
        self.nandio_pins.deassert_cs();
        self.delay.delay_us(DELAY_US_FOR_RESET);
        trace!("Reset NAND IC")
    }

    /// Read NAND IC ID
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

        trace!(
            "ID Read: [{:02x}, {:02x}, {:02x}, {:02x}, {:02x}]",
            id_read_results[0],
            id_read_results[1],
            id_read_results[2],
            id_read_results[3],
            id_read_results[4]
        );
        (id_read_results == ID_READ_EXPECT_DATA, id_read_results)
    }

    /// Read NAND IC status
    pub fn status_read(&mut self, cs_index: u32) -> StatusOutput {
        let mut status = [0x00];

        self.nandio_pins.assert_cs(cs_index);
        self.nandio_pins
            .input_command(CommandId::StatusRead as u8, || {
                self.delay.delay_us(DELAY_US_FOR_COMMAND_LATCH)
            });
        self.nandio_pins.output_data(&mut status, 1, || {
            self.delay.delay_us(DELAY_US_FOR_COMMAND_LATCH)
        });
        self.nandio_pins.deassert_cs();

        trace!("Status Read: {:02x}", status[0]);
        StatusOutput { data: status[0] }
    }

    /// Read NAND IC data
    pub fn read_data(
        &mut self,
        cs_index: u32,
        address: Address,
        read_data_ref: &mut [u8],
        read_bytes: u32,
    ) -> Result<(), Error> {
        self.nandio_pins.assert_cs(cs_index);
        self.nandio_pins
            .input_command(CommandId::ReadFirst as u8, || {
                self.delay.delay_us(DELAY_US_FOR_COMMAND_LATCH)
            });
        self.nandio_pins.input_address(&address.pack_slice(), || {
            self.delay.delay_us(DELAY_US_FOR_COMMAND_LATCH)
        });
        self.nandio_pins
            .input_command(CommandId::ReadSecond as u8, || {
                self.delay.delay_us(DELAY_US_FOR_COMMAND_LATCH)
            });

        match self.nandio_pins.wait_for_busy(
            || self.delay.delay_us(DELAY_US_FOR_WAIT_BUSY_READ),
            RETRY_LIMIT_COUNT_FOR_WAIT_BUSY,
        ) {
            Ok(_) => {
                self.nandio_pins
                    .output_data(read_data_ref, read_bytes as usize, || {
                        self.delay.delay_us(DELAY_US_FOR_COMMAND_LATCH)
                    });
                self.nandio_pins.deassert_cs();

                trace!("Read OK");
                Ok(())
            }
            Err(_) => {
                warn!("Timeout for read data");
                self.nandio_pins.deassert_cs();
                Err(Error::Timeout)
            }
        }
    }
}
