/// The trait for the IO address
/// The IO address is used to represent the address of the NAND flash
pub trait IoAddress {
    /// Get the column number
    fn column(&self) -> u32;

    /// Get the page number
    fn page(&self) -> u32;

    /// Get the block number
    fn block(&self) -> u32;

    /// Create an address from the block number
    fn from_block(block: u32) -> Self;

    /// Get the raw address
    fn to_slice<'d>(&self, data_buf: &'d mut [u8]);

    /// Create an address from the block number
    fn to_block_slice<'d>(&self, data_buf: &'d mut [u8]);
}
