use broccoli_core::common::io_driver::{NandIoDriver, NandIoError};

use crate::share::datatype::NandStatusReadBitFlags;

use super::{nand_address::NandAddress, nand_pins::NandIoPins};

/// NAND IC Command Driver for TC58NVG0S3HTA00 (JISC-SSD)
pub struct NandIoPioDriver<'d> {
    pins: NandIoPins<'d>,
}

impl<'d> NandIoPioDriver<'d> {
    /// Create a new NandPioDriver
    pub fn new(pins: NandIoPins<'d>) -> Self {
        Self { pins }
    }
}

impl<'d> NandIoDriver<NandAddress, NandStatusReadBitFlags> for NandIoPioDriver<'d> {
    async fn setup(&mut self) {
        self.pins.setup().await;
    }

    async fn set_write_protect(&mut self, enable: bool) {
        self.pins.set_write_protect(enable).await;
    }

    async fn reset(&mut self, address: NandAddress) {
        defmt::unimplemented!();
    }

    /// Read NAND IC ID
    async fn read_id(&mut self, address: NandAddress) -> bool {
        defmt::unimplemented!();
    }

    /// Read NAND IC status
    async fn read_status(&mut self, address: NandAddress) -> NandStatusReadBitFlags {
        defmt::unimplemented!();
    }

    /// Read NAND IC data
    async fn read_data<'data>(
        &mut self,
        address: NandAddress,
        read_data_ref: &'data mut [u8],
        read_bytes: usize,
    ) -> Result<(), NandIoError> {
        defmt::unimplemented!();
    }

    async fn erase_block(
        &mut self,
        address: NandAddress,
    ) -> Result<NandStatusReadBitFlags, NandIoError> {
        defmt::unimplemented!();
    }

    async fn write_data(
        &mut self,
        address: NandAddress,
        write_data_ref: &[u8],
        write_bytes: usize,
    ) -> Result<NandStatusReadBitFlags, NandIoError> {
        defmt::unimplemented!();
    }
}
