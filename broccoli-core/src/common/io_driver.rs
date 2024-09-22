#![cfg_attr(not(test), no_std)]

use core::future::Future;

use bit_field::BitField;
use bitflags::bitflags;

#[cfg(test)]
use async_mock::async_mock;
use async_trait::async_trait;

use trait_variant;

use super::io_address::IoAddress;

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

/// Status Read Result
/// This enum is used to check the result of Status Read
pub trait NandStatusReadResult {
    /// Check if the chip status bit is failed
    fn is_failed(&self) -> bool;
    /// Check if the chip status bit is pass
    fn is_write_protect(&self) -> bool;
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
pub trait NandIoDriver<Addr: IoAddress, Status: NandStatusReadResult> {
    /// Initialize all pins
    async fn setup(&mut self);
    /// Set write protect
    async fn set_write_protect(&mut self, enable: bool);
    /// Reset NAND IC
    async fn reset(&mut self, cs_index: usize);
    /// Check NAND IC ID Succeed
    async fn read_id(&mut self, cs_index: usize) -> bool;
    /// Read NAND IC status
    async fn read_status(&mut self, cs_index: usize) -> Status;
    /// Read NAND IC data
    async fn read_data<'data>(
        &mut self,
        cs_index: usize,
        address: Addr,
        read_data_ref: &'data mut [u8],
        read_bytes: usize,
    ) -> Result<(), NandIoError>;
    /// Erase NAND IC block
    async fn erase_block(&mut self, cs_index: usize, address: Addr) -> Result<Status, NandIoError>;
    /// Write NAND IC data
    async fn write_data(
        &mut self,
        cs_index: usize,
        address: Addr,
        write_data_ref: &[u8],
        write_bytes: usize,
    ) -> Result<Status, NandIoError>;
}
