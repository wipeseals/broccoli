use crate::common::io_address::IoAddress;
use bitvec::prelude::*;

/// NAND Block Allocator/Manager
#[cfg_attr(test, derive(Debug))]
pub struct NandBlockAllocator<
    Addr: IoAddress + Copy + Clone + Eq + PartialEq,
    const MAX_CHIP_NUM: usize,
    const NAND_BLOCKS_PER_CHIP: usize,
> {
    /// BadBlock Bitmaps
    bad_blocks: BitArray<[u32; MAX_CHIP_NUM * (NAND_BLOCKS_PER_CHIP + 31) / 32], Lsb0>,
    /// Allocate Block Bitmaps
    /// 0: Free, 1: Allocated
    allocate_blocks: BitArray<[u32; MAX_CHIP_NUM * (NAND_BLOCKS_PER_CHIP + 31) / 32], Lsb0>,

    /// PhantomData to hold the Addr type parameter
    _phantom: core::marker::PhantomData<Addr>,
}

impl<
        Addr: IoAddress + Copy + Clone + Eq + PartialEq,
        const MAX_CHIP_NUM: usize,
        const NAND_BLOCKS_PER_CHIP: usize,
    > Default for NandBlockAllocator<Addr, MAX_CHIP_NUM, NAND_BLOCKS_PER_CHIP>
{
    fn default() -> Self {
        Self::new()
    }
}

impl<
        Addr: IoAddress + Copy + Clone + Eq + PartialEq,
        const MAX_CHIP_NUM: usize,
        const NAND_BLOCKS_PER_CHIP: usize,
    > NandBlockAllocator<Addr, MAX_CHIP_NUM, NAND_BLOCKS_PER_CHIP>
{
    /// Create a new NandBlockAllocator
    pub fn new() -> Self {
        Self {
            bad_blocks: bitarr![u32, Lsb0; 0; MAX_CHIP_NUM * (NAND_BLOCKS_PER_CHIP + 31) / 32],
            allocate_blocks: bitarr![u32, Lsb0; 0; MAX_CHIP_NUM * (NAND_BLOCKS_PER_CHIP + 31) / 32],
            _phantom: core::marker::PhantomData,
        }
    }

    /// Allocate a Block
    /// Return the address of the allocated block
    /// If no block is available, return None
    pub fn allocate(&mut self) -> Option<Addr> {
        // 総当たりで空きブロックを探す
        // TODO:
        None
    }
}
