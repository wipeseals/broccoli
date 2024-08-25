#![cfg_attr(not(test), no_std)]

use core::future::Future;

use crate::address::Address;
use bit_field::BitField;
use bitflags::bitflags;

#[cfg(test)]
use async_mock::async_mock;
use async_trait::async_trait;

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
pub const ID_READ_EXPECT_DATA: [u8; ID_READ_CMD_BYTES] = [0x98, 0xF1, 0x80, 0x15, 0x72];

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

bitflags! {
    #[derive(Default, Clone, Copy, PartialEq)]
    pub struct StatusOutput: u8 {
        const CHIP_STATUS0_FAIL = 0b0000_0001;
        const CHIP_STATUS1_FAIL = 0b0000_0010;
        const PAGE_BUFFER_READY = 0b0010_0000;
        const DATA_CACHE_READY = 0b0100_0000;
        const WRITE_PROTECT_DISABLE = 0b1000_0000;
    }
}

impl StatusOutput {
    pub fn is_pass(&self, chip_num: u32) -> bool {
        match chip_num {
            0 => (*self & StatusOutput::CHIP_STATUS0_FAIL).is_empty(),
            1 => (*self & StatusOutput::CHIP_STATUS1_FAIL).is_empty(),
            _ => core::unreachable!("Invalid chip number"),
        }
    }

    /// Check if page buffer is ready
    pub fn is_page_buffer_ready(&self) -> bool {
        !(*self & StatusOutput::PAGE_BUFFER_READY).is_empty()
    }

    /// Check if data cache is ready
    pub fn is_data_cache_ready(&self) -> bool {
        !(*self & StatusOutput::DATA_CACHE_READY).is_empty()
    }

    /// Check if write protect is enabled
    pub fn is_write_protect(&self) -> bool {
        !(*self & StatusOutput::WRITE_PROTECT_DISABLE).is_empty()
    }
}

pub enum Error {
    Common,
    Timeout,
}

#[cfg_attr(test, async_mock)]
#[cfg_attr(test, async_trait)]
pub trait Driver {
    /// Initialize all pins
    fn init_pins<'a>(&'a mut self);
    async fn init_pins_async<'a>(&'a mut self);

    /// Set write protect
    fn set_write_protect<'a>(&'a mut self, enable: bool);
    async fn set_write_protect_async<'a>(&'a mut self, enable: bool);

    /// Reset NAND IC
    fn reset<'a>(&'a mut self, cs_index: usize);
    async fn reset_async<'a>(&'a mut self, cs_index: usize);

    /// Read NAND IC ID
    fn read_id<'a>(&'a mut self, cs_index: usize) -> (bool, [u8; ID_READ_CMD_BYTES]);
    async fn read_id_async<'a>(
        &'a mut self,
        cs_index: usize,
    ) -> (bool, [u8; ID_READ_CMD_BYTES]);

    /// Read NAND IC data
    fn read_data<'a>(
        &'a mut self,
        cs_index: usize,
        address: Address,
        read_data_ref: &mut [u8],
        read_bytes: usize,
    ) -> Result<(), Error>;
    async fn read_data_async<'a>(
        &'a mut self,
        cs_index: usize,
        address: Address,
        read_data_ref: &mut [u8],
        read_bytes: usize,
    ) -> Result<(), Error>;

    /// Read NAND IC status
    fn read_status<'a>(&'a mut self, cs_index: usize) -> StatusOutput;
    async fn read_status_async<'a>(&'a mut self, cs_index: usize) -> StatusOutput;

    /// Erase NAND IC block
    fn erase_block<'a>(
        &'a mut self,
        cs_index: usize,
        address: Address,
    ) -> Result<StatusOutput, Error>;
    async fn erase_block_async<'a>(
        &'a mut self,
        cs_index: usize,
        address: Address,
    ) -> Result<StatusOutput, Error>;

    /// Write NAND IC data
    fn write_data<'a>(
        &'a mut self,
        cs_index: usize,
        address: Address,
        write_data_ref: &[u8],
        write_bytes: usize,
    ) -> Result<StatusOutput, Error>;
    async fn write_data_async<'a>(
        &'a mut self,
        cs_index: usize,
        address: Address,
        write_data_ref: &[u8],
        write_bytes: usize,
    ) -> Result<StatusOutput, Error>;
}


mod tests {
    use super::*;

    #[test]
    fn test_status_output_with_different_values() {
        let status = StatusOutput::from_bits_truncate(0b00000000);
        assert!(status.is_pass(0));
        assert!(status.is_pass(1));
        assert!(!status.is_page_buffer_ready());
        assert!(!status.is_data_cache_ready());
        assert!(!status.is_write_protect());

        let status = StatusOutput::from_bits_truncate(0b11111111);
        assert!(!status.is_pass(0));
        assert!(!status.is_pass(1));
        assert!(status.is_page_buffer_ready());
        assert!(status.is_data_cache_ready());
        assert!(status.is_write_protect());

        let status = StatusOutput::from_bits_truncate(0b10101010);
        assert!(status.is_pass(0));
        assert!(!status.is_pass(1));
        assert!(status.is_page_buffer_ready());
        assert!(!status.is_data_cache_ready());
        assert!(status.is_write_protect());

        let status = StatusOutput::from_bits_truncate(0b01010101);
        assert!(!status.is_pass(0));
        assert!(status.is_pass(1));
        assert!(!status.is_page_buffer_ready());
        assert!(status.is_data_cache_ready());
        assert!(!status.is_write_protect());
    }
}
