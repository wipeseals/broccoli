use crate::share::constant::NAND_TOTAL_ADDR_TRANSFER_BYTES;
use bitfield::bitfield;
use broccoli_core::common::io_address::IoAddress;
use byteorder::{ByteOrder, LittleEndian};

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
bitfield! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
    /// Chip Column Address.
    pub struct NandAddr(u32);
    /// column address: 12+2bit 0 ~ 2176: 12bit (~ 16383: 14bit. reserved)
    pub column, set_column: 11,0;
    /// chip_id: 2bit 0 ~ 3 (実際には0,1しか使わない)
    /// reservedを間借りしており、Addressingの生の値にするときは除外する必要あり
    pub chip, set_chip: 15,12;
    /// page address: 6bit 0 ~ 63
    pub page, set_page: 21,16;
    /// block address: 10bit 0 ~ 1023
    pub block, set_block: 31,22;
}

impl NandAddr {
    /// Create a new NandAddress
    pub fn new() -> Self {
        Self::default()
    }

    /// get raw address
    pub fn raw(&self) -> u32 {
        self.0
    }
}

impl IoAddress for NandAddr {
    fn column(&self) -> u32 {
        self.column()
    }

    fn page(&self) -> u32 {
        self.page()
    }

    fn block(&self) -> u32 {
        self.block()
    }

    fn chip(&self) -> u32 {
        self.chip()
    }

    fn from_block(chip: u32, block: u32) -> Self {
        let mut addr = NandAddr::default();
        addr.set_chip(chip);
        addr.set_block(block);
        addr
    }

    fn from_chip(chip: u32) -> Self {
        let mut addr = NandAddr::default();
        addr.set_chip(chip);
        addr
    }

    /// Pack Address into slice.
    fn to_slice(&self, data_buf: &mut [u8]) {
        crate::assert!(
            data_buf.len() == NAND_TOTAL_ADDR_TRANSFER_BYTES,
            "Invalid data_buf length"
        );

        let data = self.raw();
        LittleEndian::write_u32(data_buf, data);
    }

    fn to_block_slice(&self, data_buf: &mut [u8]) {
        // Auto block Eraseで使用するアドレス。PA7~PA0, PA15~PA8のみを使用する
        // rawの値から後方16bitを取り出す
        let data = self.raw() >> 16;
        LittleEndian::write_u16(data_buf, data as u16);
    }
}
