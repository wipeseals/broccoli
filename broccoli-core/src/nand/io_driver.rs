#![cfg_attr(not(test), no_std)]

use core::future::Future;

use crate::nand::address::NandAddress;
use bit_field::BitField;
use bitflags::bitflags;

#[cfg(test)]
use async_mock::async_mock;
use async_trait::async_trait;

use trait_variant;

/// NAND IC Command ID
#[repr(u8)]
pub enum NandCommandId {
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
    pub struct NandStatusOutput: u8 {
        const CHIP_STATUS0_FAIL = 0b0000_0001;
        const CHIP_STATUS1_FAIL = 0b0000_0010;
        const PAGE_BUFFER_READY = 0b0010_0000;
        const DATA_CACHE_READY = 0b0100_0000;
        const WRITE_PROTECT_DISABLE = 0b1000_0000;
    }
}

impl NandStatusOutput {
    pub fn is_pass(&self, chip_num: u32) -> bool {
        match chip_num {
            0 => (*self & NandStatusOutput::CHIP_STATUS0_FAIL).is_empty(),
            1 => (*self & NandStatusOutput::CHIP_STATUS1_FAIL).is_empty(),
            _ => core::unreachable!("Invalid chip number"),
        }
    }

    /// Check if page buffer is ready
    pub fn is_page_buffer_ready(&self) -> bool {
        !(*self & NandStatusOutput::PAGE_BUFFER_READY).is_empty()
    }

    /// Check if data cache is ready
    pub fn is_data_cache_ready(&self) -> bool {
        !(*self & NandStatusOutput::DATA_CACHE_READY).is_empty()
    }

    /// Check if write protect is enabled
    pub fn is_write_protect(&self) -> bool {
        !(*self & NandStatusOutput::WRITE_PROTECT_DISABLE).is_empty()
    }
}

pub enum NandIoError {
    /// Communication Timeout
    Timeout,
    /// IdRead failed. (Device not found)
    IdReadFailed,
}

#[cfg_attr(test, async_mock)]
#[cfg_attr(test, async_trait)]
#[trait_variant::make(Send)]
pub trait NandIoDriver {
    /// Initialize all pins
    async fn setup(&mut self);
    /// Set write protect
    async fn set_write_protect(&mut self, enable: bool);
    /// Reset NAND IC
    async fn reset(&mut self, cs_index: usize);
    /// Check NAND IC ID Succeed
    async fn read_id(&mut self, cs_index: usize) -> bool;
    /// Read NAND IC status
    async fn read_status(&mut self, cs_index: usize) -> NandStatusOutput;
    /// Read NAND IC data
    async fn read_data<'d>(
        &mut self,
        cs_index: usize,
        address: NandAddress,
        read_data_ref: &'d mut [u8],
        read_bytes: usize,
    ) -> Result<(), NandIoError>;
    /// Erase NAND IC block
    async fn erase_block(
        &mut self,
        cs_index: usize,
        address: NandAddress,
    ) -> Result<NandStatusOutput, NandIoError>;
    /// Write NAND IC data
    async fn write_data(
        &mut self,
        cs_index: usize,
        address: NandAddress,
        write_data_ref: &[u8],
        write_bytes: usize,
    ) -> Result<NandStatusOutput, NandIoError>;
}

mod tests {
    use super::*;

    #[test]
    fn test_status_output_with_different_values() {
        let status = NandStatusOutput::from_bits_truncate(0b00000000);
        assert!(status.is_pass(0));
        assert!(status.is_pass(1));
        assert!(!status.is_page_buffer_ready());
        assert!(!status.is_data_cache_ready());
        assert!(!status.is_write_protect());

        let status = NandStatusOutput::from_bits_truncate(0b11111111);
        assert!(!status.is_pass(0));
        assert!(!status.is_pass(1));
        assert!(status.is_page_buffer_ready());
        assert!(status.is_data_cache_ready());
        assert!(status.is_write_protect());

        let status = NandStatusOutput::from_bits_truncate(0b10101010);
        assert!(status.is_pass(0));
        assert!(!status.is_pass(1));
        assert!(status.is_page_buffer_ready());
        assert!(!status.is_data_cache_ready());
        assert!(status.is_write_protect());

        let status = NandStatusOutput::from_bits_truncate(0b01010101);
        assert!(!status.is_pass(0));
        assert!(status.is_pass(1));
        assert!(!status.is_page_buffer_ready());
        assert!(status.is_data_cache_ready());
        assert!(!status.is_write_protect());
    }
}
