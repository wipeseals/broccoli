use broccoli_core::common::io_driver::{NandIoDriver, NandIoError};

use crate::share::datatype::NandStatusFlags;

use super::{address::NandAddr, port::NandIoPort};

/// NAND IC Command Driver for TC58NVG0S3HTA00 (JISC-SSD)
pub struct NandPioDriver<'d> {
    pins: NandIoPort<'d>,
}

impl<'d> NandPioDriver<'d> {
    /// Create a new NandPioDriver
    pub fn new(pins: NandIoPort<'d>) -> Self {
        Self { pins }
    }
}

impl<'d> NandIoDriver<NandAddr, NandStatusFlags> for NandPioDriver<'d> {
    async fn setup(&mut self) {
        self.pins.setup().await;
    }

    async fn set_write_protect(&mut self, enable: bool) {
        self.pins.set_write_protect(enable).await;
    }

    async fn reset(&mut self, address: NandAddr) {
        defmt::unimplemented!();
    }

    /// Read NAND IC ID
    async fn read_id(&mut self, address: NandAddr) -> bool {
        defmt::unimplemented!();
    }

    /// Read NAND IC status
    async fn read_status(&mut self, address: NandAddr) -> NandStatusFlags {
        defmt::unimplemented!();
    }

    /// Read NAND IC data
    async fn read_data<'data>(
        &mut self,
        address: NandAddr,
        read_data_ref: &'data mut [u8],
        read_bytes: usize,
    ) -> Result<(), NandIoError> {
        defmt::unimplemented!();
    }

    async fn erase_block(&mut self, address: NandAddr) -> Result<NandStatusFlags, NandIoError> {
        defmt::unimplemented!();
    }

    async fn write_data(
        &mut self,
        address: NandAddr,
        write_data_ref: &[u8],
        write_bytes: usize,
    ) -> Result<NandStatusFlags, NandIoError> {
        defmt::unimplemented!();
    }
}
