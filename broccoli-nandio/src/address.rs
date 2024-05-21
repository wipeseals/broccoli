#![allow(unused, dead_code)]
#![cfg_attr(not(test), no_std)]

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
/// |              | IO7  | IO6  | IO5  | IO4  | IO3  | IO2  | IO1  | IO0  |
/// | ------------ | ---  | ---  | ---  | ---  | ---  | ---  | ---  | ---  |
/// | First Cycle  | CA7  | CA6  | CA5  | CA4  | CA3  | CA2  | CA1  | CA0  |
/// | Second Cycle | -    | -    | -    | -    | CA11 | CA10 | CA9  | CA8  |
/// | Third Cycle  | PA7  | PA6  | PA5  | PA4  | PA3  | PA2  | PA1  | PA0  |
/// | Fourth Cycle | PA15 | PA14 | PA13 | PA12 | PA11 | PA10 | PA9  | PA8  |
///
/// CAx: Column Address
/// PAx: Page Address
///   PA15~PA8: Block Address
pub struct Address {
    /// Column Address 12bit (15,14,13,12: unused)
    pub column: u16,
    /// Page Address 16bit (Block Address 8bit + Page Address 8bit)
    pub page: u16,
}

impl Address {
    /// Get Block Address (PA15~PA8)
    pub fn to_block_address(&self) -> u8 {
        (self.page >> 8) as u8
    }
    /// Create Address from Block Address
    pub fn from_block_address(block: u8) -> Self {
        Address {
            column: 0,
            page: (block as u16) << 8,
        }
    }
    /// Pack Address into u32 data. (Column: 0~15, Page: 16~31)
    pub fn pack_u32(&self) -> u32 {
        let mut data = 0u32;
        // 1st, 2nd (column)
        data |= (self.column as u32) << 0;
        // 3rd, 4th (page)
        data |= (self.page as u32) << 16;
        data
    }

    /// Unpack u32 data into Address. (Column: 0~15, Page: 16~31)
    pub fn unpack_u32(data: u32) -> Self {
        let column = (data & 0x0000_ffff) as u16;
        let page = ((data & 0xffff_0000) >> 16) as u16;
        Address { column, page }
    }

    /// Pack Address into slice. (Column: 0~15, Page: 16~31)
    pub fn pack_slice(&self) -> [u8; 4] {
        let data = self.pack_u32();
        let mut slice = [0u8; 4];
        slice[0] = (data >> 0) as u8;
        slice[1] = (data >> 8) as u8;
        slice[2] = (data >> 16) as u8;
        slice[3] = (data >> 24) as u8;
        slice
    }

    /// Unpack slice into Address. (Column: 0~15, Page: 16~31)
    pub fn unpack_slice(slice: &[u8; 4]) -> Self {
        let data = (slice[0] as u32)
            | ((slice[1] as u32) << 8)
            | ((slice[2] as u32) << 16)
            | ((slice[3] as u32) << 24);
        Address::unpack_u32(data)
    }
}
