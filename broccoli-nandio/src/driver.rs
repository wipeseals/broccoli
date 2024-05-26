#![allow(unused, dead_code)]
#![cfg_attr(not(test), no_std)]

use crate::address::Address;

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

pub trait Driver {
    /// Initialize all pins
    fn init_pins(&mut self);

    /// Reset NAND IC
    fn reset(&mut self, cs_index: u32);

    /// Read NAND IC ID
    fn id_read(&mut self, cs_index: u32) -> (bool, [u8; 5]);

    /// Read NAND IC status
    fn status_read(&mut self, cs_index: u32) -> StatusOutput;

    /// Read NAND IC data
    fn read_data(
        &mut self,
        cs_index: u32,
        address: Address,
        read_data_ref: &mut [u8],
        read_bytes: u32,
    ) -> Result<(), Error>;
}
