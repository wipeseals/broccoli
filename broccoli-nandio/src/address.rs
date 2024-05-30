#![allow(unused, dead_code)]
#![cfg_attr(not(test), no_std)]

extern crate bitfield;
use bitfield::bitfield;

/// Usable NAND Page Size
pub const DATA_BYTES_PER_PAGE: usize = 2048;
/// Metadata on NAND Page
pub const SPARE_BYTES_PER_PAGE: usize = 128;
/// Page/Block
pub const PAGES_PER_BLOCK: usize = 64;
/// Total Blocks per IC
pub const MAX_BLOCKS_PER_IC: usize = 1024;
/// Minimum Blocks per IC
pub const MIN_BLOCKS_PER_IC: usize = 1004;
/// minimum number of IC
pub const MIN_IC: usize = 1;
/// Maximum number of IC
pub const MAX_IC: usize = 2;

/// Total NAND Page Size (Data + Spare = 2176 bytes)
pub const TOTAL_BYTES_PER_PAGE: usize = DATA_BYTES_PER_PAGE + SPARE_BYTES_PER_PAGE;
/// Total Bytes per Block (2176 * 64 = 139264 bytes)
pub const BYTES_PER_BLOCK: usize = TOTAL_BYTES_PER_PAGE * PAGES_PER_BLOCK;
/// Maximum Pages per IC (64 * 1024 = 65536 pages)
pub const MAX_PAGES_PER_IC: usize = MAX_BLOCKS_PER_IC * PAGES_PER_BLOCK;
/// Maximum Bytes per IC (139264 * 1024 = 142606336 bytes = 142.6MB)
pub const MAX_BYTES_PER_IC: usize = MAX_BLOCKS_PER_IC * BYTES_PER_BLOCK;
/// Minimum Pages per IC (64 * 1004 = 64256 pages)
pub const MIN_PAGS_PER_IC: usize = MIN_BLOCKS_PER_IC * PAGES_PER_BLOCK;
/// Minimum Bytes per IC (139264 * 1004 = 140000256 bytes = 140MB)
pub const MIN_BYTES_PER_IC: usize = MIN_BLOCKS_PER_IC * BYTES_PER_BLOCK;

/// Address for NAND
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

bitfield! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
    pub struct Address(u32);
    pub column, set_column: 11,0;
    pub reserved, _: 15,12;
    pub page, set_page: 21,16;
    pub block, set_block: 31,22;
}

impl Address {
    /// Pack Address into slice.
    pub fn to_full_slice(&self) -> [u8; 4] {
        let data = self.0;
        let mut slice = [0u8; 4];
        slice[0] = data as u8;
        slice[1] = (data >> 8) as u8;
        slice[2] = (data >> 16) as u8;
        slice[3] = (data >> 24) as u8;
        slice
    }

    /// Unpack slice into Address.
    pub fn from_full_slice(slice: &[u8; 4]) -> Self {
        let data = (slice[0] as u32)
            | ((slice[1] as u32) << 8)
            | ((slice[2] as u32) << 16)
            | ((slice[3] as u32) << 24);
        Address(data)
    }

    /// Pack Page Address into slice.
    pub fn to_page_slice(&self) -> [u8; 2] {
        let data = self.0;
        let mut slice = [0u8; 2];
        // PA7~PA0
        slice[0] = (data >> 16) as u8;
        // PA15~PA8
        slice[1] = (data >> 24) as u8;
        slice
    }

    /// Unpack slice into Page Address.
    pub fn from_page_slice(slice: &[u8; 2]) -> Self {
        let data = ((slice[0] as u32) << 16) | ((slice[1] as u32) << 24);
        Address(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_full_slice() {
        let mut address = Address::default();
        address.set_column(0b101010101010);
        address.set_page(0b110011001100);
        address.set_block(0b111100001111);
        let packed = address.to_full_slice();
        let expect_value: u32 = 0b_1100001111_001100_0000_101010101010;
        //                         block      page  rsv   column
        //                         10bit      6bit  4bit  12bit
        let expect_packed = [
            (expect_value & 0xFF) as u8,
            ((expect_value >> 8) & 0xFF) as u8,
            ((expect_value >> 16) & 0xFF) as u8,
            ((expect_value >> 24) & 0xFF) as u8,
        ];
        assert_eq!(packed, expect_packed);
    }

    #[test]
    fn test_from_full_slice() {
        let packed = [0b10101010, 0b11001100, 0b11110000, 0b11111111];
        //                     column[7:0]  column[12:9]  block[1:0] block[15:2]
        //                                                page[5:0]
        let address = Address::from_full_slice(&packed);
        assert_eq!(address.column(), 0b0000_1100_10101010);
        assert_eq!(address.page(), 0b110000);
        assert_eq!(address.block(), 0b11111111_11);
    }
}
