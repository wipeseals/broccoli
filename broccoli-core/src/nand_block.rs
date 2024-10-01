use crate::common::io_address::IoAddress;

/// NAND Block State
#[derive(Copy, Clone, Eq, PartialEq)]
#[repr(u8)]
#[cfg_attr(test, derive(Debug))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum NandBlockState {
    /// Unknown
    Unknown,
    /// Not mounted
    NotMounted,
    /// Bad Block (Initial)
    InitialBad,
    /// Bad Block (Initial by other)
    InitialBadByOtherError,
    /// Bad Block (Erase Failed)
    EraseFailedBad,
    /// Bad Block (Write Failed)
    WriteFailedBad,
    /// Bad Block (Read Failed)
    ReadFailedBad,
    /// Erased Block
    Erased,
    /// Writting Block
    Writing,
    /// Written Block
    Written,
    /// Free Block (Reusable)
    Free,

    /// Max Value
    MaxIndexEntry,
}

impl Default for NandBlockState {
    fn default() -> Self {
        Self::new()
    }
}

impl NandBlockState {
    /// Create a new NandBlockState
    pub const fn new() -> Self {
        Self::Unknown
    }

    /// Check if the block is reusable
    pub fn is_reusable(&self) -> bool {
        matches!(self, Self::Free)
    }

    /// Check if the block is bad
    pub fn is_bad(&self) -> bool {
        matches!(
            self,
            Self::InitialBad | Self::EraseFailedBad | Self::WriteFailedBad | Self::ReadFailedBad
        )
    }

    /// Check if the block is bad by other error
    pub const fn valid_entry_num() -> u8 {
        NandBlockState::MaxIndexEntry as u8
    }
}

/// NAND Block State
#[derive(Copy, Clone, Eq, PartialEq)]
#[cfg_attr(test, derive(Debug))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct NandBlockInfo {
    /// Block State
    state: NandBlockState,
    /// Active Data Reference Count
    ref_count: u32,
}

impl Default for NandBlockInfo {
    fn default() -> Self {
        Self::new()
    }
}

impl NandBlockInfo {
    /// Create a new NandBlockInfo
    pub const fn new() -> Self {
        Self {
            state: NandBlockState::new(),
            ref_count: 0,
        }
    }

    /// Get the state
    pub fn state(&self) -> NandBlockState {
        self.state
    }

    /// Get the reference count
    pub fn ref_count(&self) -> u32 {
        self.ref_count
    }

    /// Set the state
    pub fn set_state(&mut self, state: NandBlockState) {
        self.state = state;
    }

    /// Set the reference count
    pub fn set_ref_count(&mut self, ref_count: u32) {
        self.ref_count = ref_count;
    }

    /// Increment the reference count
    pub fn inc_ref_count(&mut self) {
        self.ref_count += 1;
    }

    /// Decrement the reference count
    pub fn dec_ref_count(&mut self) {
        self.ref_count -= 1;
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
#[cfg_attr(test, derive(Debug))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct NandBlockStats {
    /// Counts by State
    counts_by_state: [u32; NandBlockState::valid_entry_num() as usize],
}

impl Default for NandBlockStats {
    fn default() -> Self {
        Self::new()
    }
}

impl NandBlockStats {
    /// Create a new NandBlockStats
    pub const fn new() -> Self {
        Self {
            counts_by_state: [0; NandBlockState::valid_entry_num() as usize],
        }
    }

    /// Update the stats
    pub fn update(&mut self, old_state: Option<NandBlockState>, new_state: NandBlockState) {
        // Update the counts
        // Unknown -> InitialBad/InitialBadByOtherError/Free 遷移時など、old_stateのカウントがいないケースが有る
        if let Some(old_state) = old_state {
            self.counts_by_state[old_state as usize] -= 1;
        }
        self.counts_by_state[new_state as usize] += 1;
    }

    /// Get the Free Block Count
    pub fn free_count(&self) -> u32 {
        self.counts_by_state[NandBlockState::Free as usize]
    }
}

/// NAND Block Allocator/Manager
#[cfg_attr(test, derive(Debug))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct NandBlockAllocator<
    Addr: IoAddress + Copy + Clone + Eq + PartialEq,
    const MAX_CHIP_NUM: usize,
    const NAND_BLOCKS_PER_CHIP: usize,
> {
    /// Block Infos
    info_list: [[NandBlockInfo; NAND_BLOCKS_PER_CHIP]; MAX_CHIP_NUM],
    /// Initial Block Stats
    init_stats: NandBlockStats,
    /// Current Block Stats
    now_stats: NandBlockStats,

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
            info_list: [[NandBlockInfo::default(); NAND_BLOCKS_PER_CHIP]; MAX_CHIP_NUM],
            init_stats: NandBlockStats::new(),
            now_stats: NandBlockStats::new(),
            _phantom: core::marker::PhantomData,
        }
    }

    /// Update Block State
    pub fn change_state(&mut self, addr: Addr, new_state: NandBlockState, is_initial: bool) {
        let chip = addr.chip() as usize;
        let block = addr.block() as usize;

        let old_state = self.info_list[chip][block].state();
        let old_state = if old_state == NandBlockState::Unknown {
            None
        } else {
            Some(old_state)
        };
        self.info_list[chip][block].set_state(new_state);
        self.now_stats.update(old_state, new_state);
        // 初回だけ更新
        if is_initial {
            self.init_stats.update(None, new_state);
        }
    }

    /// Allocate a Block
    /// Return the address of the allocated block
    /// If no block is available, return None
    pub fn allocate(&mut self) -> Option<Addr> {
        // 総当たりで空きブロックを探す
        for chip in 0..MAX_CHIP_NUM {
            for block in 0..NAND_BLOCKS_PER_CHIP {
                if self.info_list[chip][block].state().is_reusable() {
                    return Some(Addr::from_block(chip as u32, block as u32));
                }
            }
        }
        None
    }
}
