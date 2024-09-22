use core::mem;

use crate::commander::NandCommander;
use crate::common::io_address::IoAddress;
use crate::common::io_driver::{NandIoDriver, NandStatusReadResult};

use crate::common::storage_req::{StorageHandler, StorageMsgId, StorageRequest, StorageResponse};

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

impl<Error: Copy + Clone + Eq + PartialEq> CacheStatus<Error> {
    /// Create a new CacheStatus
    pub const fn new() -> Self {
        Self::Initial
    }

    /// Check if the buffer is reusable
    /// 初期状態、読み込み完了、書き込み完了の場合は再利用可能
    pub fn is_reusable(&self) -> bool {
        match self {
            Self::Initial => true,
            Self::ReadComplete { error: _ } => true,
            Self::WriteComplete { error: _ } => true,
            _ => false,
        }
    }

    /// Check if the buffer is valid
    /// 読み込み完了、データ変更、書き込み完了の場合はデータが有効
    pub fn is_data_unchanged(&self) -> bool {
        match self {
            Self::ReadComplete { error: _ } => true,
            Self::Changed => true,
            Self::WriteComplete { error: _ } => true,
            _ => false,
        }
    }

    /// Check if the buffer is clean
    /// 初期状態、読み込み完了、書き込み完了の場合はデータがRAM上で変更されている
    pub fn is_data_changed(&self) -> bool {
        match self {
            Self::Changed => true,
            Self::EncodingBeforeWrite => true,
            Self::Writing => true,
            _ => false,
        }
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

/// Flash Storage Controller for FTL
/// Buffer Size == Nand Page Size
/// Logical Block Size <= Nand Page Size
pub struct NandStorageHandler<
    'd,
    Addr: IoAddress,
    Status: NandStatusReadResult,
    Driver: NandIoDriver<Addr, Status>,
    const MAX_IC_NUM: usize,
> {
    /// NAND IO Commander
    commander: NandCommander<'d, Addr, Status, Driver, MAX_IC_NUM>,
    // TODO: Add NAND Map
    // TODO: Add NAND Block Assignment
    // TODO: Channel for NAND Controller, ...
}

impl<
        'd,
        Addr: IoAddress,
        Status: NandStatusReadResult,
        Driver: NandIoDriver<Addr, Status>,
        const MAX_IC_NUM: usize,
    > NandStorageHandler<'d, Addr, Status, Driver, MAX_IC_NUM>
{
    /// Create a new NandStorageHandler
    pub fn new(driver: &'d mut Driver) -> Self {
        Self {
            commander: NandCommander::new(driver),
        }
    }
}

impl<
        'd,
        Addr: IoAddress,
        Status: NandStatusReadResult,
        Driver: NandIoDriver<Addr, Status>,
        const MAX_IC_NUM: usize,
        ReqTag: Eq + PartialEq,
        const LOGICAL_BLOCK_SIZE: usize,
    > StorageHandler<ReqTag, LOGICAL_BLOCK_SIZE>
    for NandStorageHandler<'d, Addr, Status, Driver, MAX_IC_NUM>
{
    /// Request handler
    async fn request(
        &mut self,
        request: StorageRequest<ReqTag, LOGICAL_BLOCK_SIZE>,
    ) -> StorageResponse<ReqTag, LOGICAL_BLOCK_SIZE> {
        match request.message_id {
            StorageMsgId::Setup => {
                // setup NAND Commander(Driver)
                match self.commander.setup().await {
                    Ok(num_cs) => {
                        StorageResponse::report_setup_success(
                            request.req_tag,
                            (num_cs * (1024 - 100) * 64 * 2048 / LOGICAL_BLOCK_SIZE), // TODO: 仮の値. NANDの容量とブロックサイズ、管理データ向けに割り当てた容量から計算する
                        )
                    }
                    Err(_) => StorageResponse::report_setup_failed(request.req_tag),
                }
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
