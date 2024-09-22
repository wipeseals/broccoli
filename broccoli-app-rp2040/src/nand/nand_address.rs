use broccoli_core::common::io_address::IoAddress;
use byteorder::{ByteOrder, LittleEndian};

use crate::shared::constant::NAND_TOTAL_ADDR_TRANSFER_BYTES;

/// Chip Column Address.
///
/// Read/Write
/// |              | IO7  | IO6  | IO5  | IO4  | IO3  | IO2  | IO1  | IO0  |
/// | ------------ | ---  | ---  | ---  | ---  | ---  | ---  | ---  | ---  |
/// | First Cycle  | CA7  | CA6  | CA5  | CA4  | CA3  | CA2  | CA1  | CA0  |
/// | Second Cycle | -    | -    | -    | -    | CA11 | CA10 | CA9  | CA8  |
/// | Third Cycle  | PA7  | PA6  | PA5  | PA4  | PA3  | PA2  | PA1  | PA0  |
/// | Fourth Cycle | PA15 | PA14 | PA13 | PA12 | PA11 | PA10 | PA9  | PA8  |
///
/// Auto Block Erase
/// |              | IO7  | IO6  | IO5  | IO4  | IO3  | IO2  | IO1  | IO0  |
/// | ------------ | ---  | ---  | ---  | ---  | ---  | ---  | ---  | ---  |
/// | First Cycle  | PA7  | PA6  | PA5  | PA4  | PA3  | PA2  | PA1  | PA0  |
/// | Second Cycle | PA15 | PA14 | PA13 | PA12 | PA11 | PA10 | PA9  | PA8  |
///
/// CAx: Column Address
/// PAx: Page Address
///   PA15~PA6: Block Address
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct NandAddress {
    column: u16,
    page: u16,
}

impl NandAddress {
    /// Create a new NandAddress
    pub fn new() -> Self {
        Self::default()
    }

    /// get raw address
    pub fn raw(&self) -> u32 {
        ((self.page as u32) << 16) | (self.column as u32)
    }

    /// Set Column Address
    pub fn set_column(&mut self, column: u16) {
        self.column = column;
    }

    /// Set Page Address
    pub fn set_page(&mut self, page: u16) {
        self.page = page;
    }

    /// Set Block Address
    pub fn set_block(&mut self, block: u32) {
        self.page = (block << 6) as u16;
    }
}

impl IoAddress for NandAddress {
    fn column(&self) -> u32 {
        self.column as u32
    }

    fn page(&self) -> u32 {
        self.page as u32
    }

    fn block(&self) -> u32 {
        (self.page >> 6) as u32
    }

    /// address from block
    fn from_block(block: u32) -> Self {
        let mut addr = NandAddress::default();
        addr.set_block(block);
        addr
    }

    /// Pack Address into slice.
    fn to_slice<'d>(&self, data_buf: &'d mut [u8]) {
        crate::assert!(
            data_buf.len() == NAND_TOTAL_ADDR_TRANSFER_BYTES,
            "Invalid data_buf length"
        );

        let data = self.raw();
        LittleEndian::write_u32(data_buf, data);
    }

    fn to_block_slice<'d>(&self, data_buf: &'d mut [u8]) {
        let data = self.block();
        LittleEndian::write_u16(data_buf, data as u16);
    }
}
