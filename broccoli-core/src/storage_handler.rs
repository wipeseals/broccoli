use core::mem;

use crate::commander::NandCommander;
use crate::common::io_address::IoAddress;
use crate::common::io_driver::{NandIoDriver, NandStatusReadResult};

use crate::common::storage_req::{
    StorageHandler, StorageMsgId, StorageRequest, StorageResponse, StorageResponseReport,
};

/// Buffer Assigning Type for NandStorageHandler
#[derive(Copy, Clone, Eq, PartialEq)]
#[cfg_attr(test, derive(Debug))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum CacheDataType {
    /// Empty Buffer
    Empty,
    /// Buffer is being used by Host Data
    HostData,
    /// Buffer is being used by Map Data
    MapData,
}

impl Default for CacheDataType {
    fn default() -> Self {
        Self::new()
    }
}

impl CacheDataType {
    /// Create a new CacheDataType
    pub const fn new() -> Self {
        Self::Empty
    }
}

/// Buffer Progress
/// This enum is used to check the progress of Buffer
/// - Initial -> Reading -> DecodingAfterRead -> ReadComplete -> Initial
/// - Initial -> EncodingBeforeWrite -> Writing -> WriteComplete -> Initial
#[derive(Copy, Clone, Eq, PartialEq)]
#[cfg_attr(test, derive(Debug))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum CacheStatus<Error: Copy + Clone + Eq + PartialEq> {
    /// Buffer is Empty (data invalid)
    Initial,
    /// Buffer is being Read (data invalid)
    Reading,
    /// Buffer Read is Complete (data valid, but not yet decoded)
    DecodingAfterRead,
    /// Buffer Read is Complete (data valid)
    ReadComplete { error: Error },
    /// Buffer data has been changed (data valid)
    Changed,
    /// Buffer is being Written (data valid, but not yet encoded)
    EncodingBeforeWrite,
    /// Buffer is being Written (data valid, but not yet written)
    Writing,
    /// Buffer Write is Complete (data valid, written)
    WriteComplete { error: Error },
}

impl<Error: Copy + Clone + Eq + PartialEq> Default for CacheStatus<Error> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Error: Copy + Clone + Eq + PartialEq> CacheStatus<Error> {
    /// Create a new CacheStatus
    pub const fn new() -> Self {
        Self::Initial
    }

    /// Check if the buffer is reusable
    /// 初期状態、読み込み完了、書き込み完了の場合は再利用可能
    pub fn is_reusable(&self) -> bool {
        matches!(
            self,
            Self::Initial | Self::ReadComplete { .. } | Self::WriteComplete { .. }
        )
    }

    /// Check if the buffer is valid
    /// 読み込み完了、データ変更、書き込み完了の場合はデータが有効
    pub fn is_data_unchanged(&self) -> bool {
        matches!(
            self,
            Self::ReadComplete { .. } | Self::Changed | Self::WriteComplete { .. }
        )
    }

    /// Check if the buffer is clean
    /// 初期状態、読み込み完了、書き込み完了の場合はデータがRAM上で変更されている
    pub fn is_data_changed(&self) -> bool {
        matches!(
            self,
            Self::Changed | Self::EncodingBeforeWrite | Self::Writing
        )
    }
}

/// Buffer Status
/// This struct is used to check the status of Buffer
#[derive(Eq, PartialEq)]
#[cfg_attr(test, derive(Debug))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CacheBuffer<
    LogicalAddr: Copy + Clone + Eq + PartialEq,
    NandAddr: Copy + Clone + Eq + PartialEq + Copy,
    Error: Copy + Clone + Eq + PartialEq,
    const LOGICAL_BLOCK_SIZE: usize,
> {
    /// Logical Address
    pub logical_addr: Option<LogicalAddr>,

    /// Physical Address
    pub nand_addr: Option<NandAddr>,

    /// Buffer Status
    pub status: CacheStatus<Error>,

    /// Buffer Type
    pub buffer_type: CacheDataType,

    /// Buffer Data
    pub data: [u8; LOGICAL_BLOCK_SIZE],
}

impl<
        LogicalAddr: Copy + Clone + Eq + PartialEq,
        NandAddr: Copy + Clone + Eq + PartialEq,
        Error: Copy + Clone + Eq + PartialEq,
        const LOGICAL_BLOCK_SIZE: usize,
    > Default for CacheBuffer<LogicalAddr, NandAddr, Error, LOGICAL_BLOCK_SIZE>
{
    fn default() -> Self {
        Self::new()
    }
}

impl<
        LogicalAddr: Copy + Clone + Eq + PartialEq,
        NandAddr: Copy + Clone + Eq + PartialEq,
        Error: Copy + Clone + Eq + PartialEq,
        const LOGICAL_BLOCK_SIZE: usize,
    > CacheBuffer<LogicalAddr, NandAddr, Error, LOGICAL_BLOCK_SIZE>
{
    /// Create a new CacheBuffer
    pub const fn new() -> Self {
        Self {
            logical_addr: None,
            nand_addr: None,
            status: CacheStatus::new(),
            buffer_type: CacheDataType::new(),
            data: [0; LOGICAL_BLOCK_SIZE],
        }
    }
}

///////////////////////////////////////////////////////////////////////////////

/// NAND Block State
#[derive(Copy, Clone, Eq, PartialEq)]
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
}

/// NAND Block State
#[derive(Copy, Clone, Eq, PartialEq)]
#[cfg_attr(test, derive(Debug))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct NandBlockInfo {
    /// Block State
    pub state: NandBlockState,
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
        }
    }
}

/// Flash Storage Controller for FTL
pub struct NandStorageHandler<
    'd,
    Addr: IoAddress + Copy + Clone + Eq + PartialEq,
    Status: NandStatusReadResult,
    Driver: NandIoDriver<Addr, Status>,
    const MAX_CHIP_NUM: usize,
    const NAND_BLOCKS_PER_CHIP: usize,
> {
    /// NAND IO Commander
    commander: NandCommander<'d, Addr, Status, Driver, MAX_CHIP_NUM>,
    /// NAND Block Information
    block_infos: [[NandBlockInfo; NAND_BLOCKS_PER_CHIP]; MAX_CHIP_NUM],

    /// BadBlock Check passed count
    initial_goodblock_count: u32,
    /// BadBlock Check failed count
    initial_badblock_count: u32,
    /// BadBlock Check failed count (by other error)
    initial_badblock_by_other_error_count: u32,
    // TODO: Add NAND Map
    // TODO: Add NAND Block Assignment
    // TODO: Channel for NAND Controller, ...
}

impl<
        'd,
        Addr: IoAddress + Copy + Clone + Eq + PartialEq,
        Status: NandStatusReadResult,
        Driver: NandIoDriver<Addr, Status>,
        const MAX_CHIP_NUM: usize,
        const NAND_BLOCKS_PER_CHIP: usize,
    > NandStorageHandler<'d, Addr, Status, Driver, MAX_CHIP_NUM, NAND_BLOCKS_PER_CHIP>
{
    /// Create a new NandStorageHandler
    pub fn new(driver: &'d mut Driver) -> Self {
        Self {
            commander: NandCommander::new(driver),
            block_infos: [[NandBlockInfo::default(); NAND_BLOCKS_PER_CHIP]; MAX_CHIP_NUM],
            initial_goodblock_count: 0,
            initial_badblock_count: 0,
            initial_badblock_by_other_error_count: 0,
        }
    }

    /// Check bad block for initialization
    async fn setup_all_blocks(&mut self) -> Result<(), StorageResponseReport> {
        // setup NAND Commander(Driver)
        let Ok(num_cs) = self.commander.setup().await else {
            return Err(StorageResponseReport::NandError);
        };
        // BadBlockの情報を取得
        for chip in 0..num_cs {
            for block in 0..NAND_BLOCKS_PER_CHIP {
                let addr = Addr::from_block(chip as u32, block as u32);
                match self.commander.check_badblock(addr).await {
                    Ok(is_bad) => {
                        if is_bad {
                            self.block_infos[chip][block].state = NandBlockState::InitialBad;
                            self.initial_badblock_count += 1;
                        } else {
                            self.block_infos[chip][block].state = NandBlockState::Free;
                            self.initial_goodblock_count += 1;
                        }
                    }
                    Err(_) => {
                        // エラーが発生した場合は、一応BadBlockに割り当てておく
                        self.block_infos[chip][block].state =
                            NandBlockState::InitialBadByOtherError;
                        self.initial_badblock_by_other_error_count += 1;
                    }
                }
            }
        }
        // CS1が見つからない場合、NotMountedで埋めておく
        for chip in num_cs..MAX_CHIP_NUM {
            for block in 0..NAND_BLOCKS_PER_CHIP {
                self.block_infos[chip][block].state = NandBlockState::NotMounted;
            }
        }

        Ok(())
    }
}

impl<
        'd,
        Addr: IoAddress + Copy + Clone + Eq + PartialEq,
        Status: NandStatusReadResult,
        Driver: NandIoDriver<Addr, Status>,
        const MAX_IC_NUM: usize,
        ReqTag: Eq + PartialEq,
        const LOGICAL_BLOCK_SIZE: usize,
        const NAND_BLOCKS_PER_CHIP: usize,
    > StorageHandler<ReqTag, LOGICAL_BLOCK_SIZE>
    for NandStorageHandler<'d, Addr, Status, Driver, MAX_IC_NUM, NAND_BLOCKS_PER_CHIP>
{
    /// Request handler
    async fn request(
        &mut self,
        request: StorageRequest<ReqTag, LOGICAL_BLOCK_SIZE>,
    ) -> StorageResponse<ReqTag, LOGICAL_BLOCK_SIZE> {
        match request.message_id {
            StorageMsgId::Setup => {
                // TODO: 不揮発データから初回Setup要否切り替え
                let is_need_first_setup = true;
                // TODO: 仮の値. NANDの容量とブロックサイズ、管理データ向けに割り当てた容量から計算する
                let num_blocks = ((1024 - 100) * 64 * 2048 / LOGICAL_BLOCK_SIZE);

                if !is_need_first_setup {
                    // TODO: 2回目以降のSetup処理. 保存した不揮発データのsignature checkなりnum_csの一致などは見ておく
                    return StorageResponse::report_setup_success(request.req_tag, num_blocks);
                }

                // 初回セットアップしてから容量を報告
                if let Err(report) = self.setup_all_blocks().await {
                    return StorageResponse::report_setup_failed(request.req_tag, report);
                }
                StorageResponse::report_setup_success(request.req_tag, num_blocks)
            }
            StorageMsgId::Echo => {
                // Echoは何もしない
                StorageResponse::echo(request.req_tag)
            }
            StorageMsgId::Read => {
                // Read
                // TODO: NANDからデータを読み出す処理
                StorageResponse::read(request.req_tag, [0; LOGICAL_BLOCK_SIZE])
            }
            StorageMsgId::Write => {
                // Write
                // TODO: NANDにデータを書き込む処理

                StorageResponse::write(request.req_tag)
            }
            StorageMsgId::Flush => {
                // Flush
                // TODO: WriteBufferの内容をNANDに書き込む処理

                StorageResponse::flush(request.req_tag)
            }
        }
    }
}
