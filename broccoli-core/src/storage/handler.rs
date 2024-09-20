use core::mem;

use super::protocol::{StorageHandler, StorageMsgId, StorageRequest, StorageResponse};

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

pub struct NandAddressMapper<
    /// Logical Address
    LogicalAddr: Copy + Clone + Eq + PartialEq,
    /// NAND Address
    NandAddr: Copy + Clone + Eq + PartialEq,
    /// page bytes (data + spare bytes)
    const PAGE_TOTAL_SIZE: usize,
    /// page bytes (spare bytes)
    const PAGE_META_SIZE: usize,
    /// Number of pages per block
    const NUM_PAGES_PER_BLOCK: usize,
    /// Number of blocks per IC
    const NUM_BLOCKS_PER_IC: usize,
    /// Number of ICs
    const NUM_IC: usize,
    /// Number of blocks per IC
    const LOGICAL_BLOCK_SIZE: usize,
    /// Number of logical addresses
    const NUM_LOGICAL_ADDR: usize,
    /// Number of blocks per IC
    const CACHE_BUFFFER_N: usize,
> {
    // pub map_page_address_table: [Option<NandAddr>; NUM_LOGICAL_ADDR/]
}

impl<
        LogicalAddr: Copy + Clone + Eq + PartialEq,
        NandAddr: Copy + Clone + Eq + PartialEq,
        const PAGE_TOTAL_SIZE: usize,
        const PAGE_META_SIZE: usize,
        const NUM_PAGES_PER_BLOCK: usize,
        const NUM_BLOCKS_PER_IC: usize,
        const NUM_IC: usize,
        const LOGICAL_BLOCK_SIZE: usize,
        const NUM_LOGICAL_ADDR: usize,
        const CACHE_BUFFFER_N: usize,
    >
    NandAddressMapper<
        LogicalAddr,
        NandAddr,
        PAGE_TOTAL_SIZE,
        PAGE_META_SIZE,
        NUM_PAGES_PER_BLOCK,
        NUM_BLOCKS_PER_IC,
        NUM_IC,
        LOGICAL_BLOCK_SIZE,
        NUM_LOGICAL_ADDR,
        CACHE_BUFFFER_N,
    >
{
    /// NAND Pageのうち、データ部分のサイズ
    pub const fn page_data_size(&self) -> usize {
        PAGE_TOTAL_SIZE - PAGE_META_SIZE
    }
    /// NAND PageあたりのNandAddrのエントリ数
    pub const fn map_entries_per_page(&self) -> usize {
        (PAGE_TOTAL_SIZE - PAGE_META_SIZE) / mem::size_of::<NandAddr>()
    }

    /// NAND PageあたりのNandAddrエントリが示せる容量
    pub const fn map_capacity_per_page(&self) -> usize {
        (PAGE_TOTAL_SIZE - PAGE_META_SIZE) / mem::size_of::<NandAddr>()
            * ((PAGE_TOTAL_SIZE - PAGE_META_SIZE) / LOGICAL_BLOCK_SIZE) // TODO: 関数分割したいがエラーになる
    }
}

/// Flash Storage Controller for FTL
/// Buffer Size == Nand Page Size
/// Logical Block Size <= Nand Page Size
pub struct NandStorageHandler<
    LogicalAddr: Copy + Clone + Eq + PartialEq,
    NandAddr: Copy + Clone + Eq + PartialEq,
    const PAGE_TOTAL_SIZE: usize,
    const PAGE_META_SIZE: usize,
    const NUM_PAGES_PER_BLOCK: usize,
    const NUM_BLOCKS_PER_IC: usize,
    const NUM_IC: usize,
    const LOGICAL_BLOCK_SIZE: usize,
    const NUM_LOGICAL_ADDR: usize,
    const CACHE_BUFFFER_N: usize,
    const READ_BUFFER_N: usize,
    const WRITE_BUFFER_N: usize,
> {
    // TODO: Add NAND Controller
    // TODO: Add NAND Map
    // TODO: Add NAND Block Assignment
    // TODO: Channel for NAND Controller, ...
}

impl<
        const LOGICAL_BLOCK_SIZE: usize,
        const NAND_PAGE_TOTAL_SIZE: usize,
        const NAND_PAGE_METADATA_SIZE: usize,
        const READ_BUFFER_N: usize,
        const WRITE_BUFFER_N: usize,
    > Default
    for NandStorageHandler<
        LOGICAL_BLOCK_SIZE,
        NAND_PAGE_TOTAL_SIZE,
        NAND_PAGE_METADATA_SIZE,
        READ_BUFFER_N,
        WRITE_BUFFER_N,
    >
{
    fn default() -> Self {
        Self::new()
    }
}

impl<
        const LOGICAL_BLOCK_SIZE: usize,
        const NAND_PAGE_TOTAL_SIZE: usize,
        const NAND_PAGE_METADATA_SIZE: usize,
        const READ_BUFFER_N: usize,
        const WRITE_BUFFER_N: usize,
    >
    NandStorageHandler<
        LOGICAL_BLOCK_SIZE,
        NAND_PAGE_TOTAL_SIZE,
        NAND_PAGE_METADATA_SIZE,
        READ_BUFFER_N,
        WRITE_BUFFER_N,
    >
{
    /// Create a new DataBuffer
    pub const fn new() -> Self {
        if (NAND_PAGE_TOTAL_SIZE - NAND_PAGE_METADATA_SIZE) < LOGICAL_BLOCK_SIZE {
            panic!("NAND_PAGE_SIZE must be larger than LOGICAL_BLOCK_SIZE");
        }
        Self {
            internal_read_status: CacheBuffer::new(),
            internal_write_status: CacheBuffer::new(),
            host_read_statuses: [CacheBuffer::new(); READ_BUFFER_N],
            host_write_statuses: [CacheBuffer::new(); WRITE_BUFFER_N],
            internal_read_buffer: [0; NAND_PAGE_TOTAL_SIZE],
            internal_write_buffer: [0; NAND_PAGE_TOTAL_SIZE],
            host_read_buffers: [[0; NAND_PAGE_TOTAL_SIZE]; READ_BUFFER_N],
            host_write_buffers: [[0; NAND_PAGE_TOTAL_SIZE]; WRITE_BUFFER_N],
        }
    }

    pub const fn usable_bytes_per_buffer(&self) -> usize {
        NAND_PAGE_TOTAL_SIZE - NAND_PAGE_METADATA_SIZE
    }
    /// Get the number of blocks per ReadBuffer
    pub const fn logical_blocks_per_write_buffer(&self) -> usize {
        self.usable_bytes_per_buffer() / LOGICAL_BLOCK_SIZE
    }

    /// Get the number of blocks per WriteBuffer
    pub const fn logical_blocks_per_read_buffer(&self) -> usize {
        self.usable_bytes_per_buffer() / LOGICAL_BLOCK_SIZE
    }
}

impl<
        ReqTag: Eq + PartialEq,
        const LOGICAL_BLOCK_SIZE: usize,
        const NAND_PAGE_SIZE: usize,
        const NAND_PAGE_METADATA_SIZE: usize,
        const READ_BUFFER_N: usize,
        const WRITE_BUFFER_N: usize,
    > StorageHandler<ReqTag, LOGICAL_BLOCK_SIZE>
    for NandStorageHandler<
        LOGICAL_BLOCK_SIZE,
        NAND_PAGE_SIZE,
        NAND_PAGE_METADATA_SIZE,
        READ_BUFFER_N,
        WRITE_BUFFER_N,
    >
{
    /// Request handler
    async fn request(
        &mut self,
        request: StorageRequest<ReqTag, LOGICAL_BLOCK_SIZE>,
    ) -> StorageResponse<ReqTag, LOGICAL_BLOCK_SIZE> {
        match request.message_id {
            StorageMsgId::Setup => {
                // TODO: NANDの初期化処理
                StorageResponse::report_setup_success(
                    request.req_tag,
                    ((1024 - 100) * 64 * 2048 / LOGICAL_BLOCK_SIZE), // TODO: 仮の値. NANDの容量とブロックサイズ、管理データ向けに割り当てた容量から計算する
                )
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
