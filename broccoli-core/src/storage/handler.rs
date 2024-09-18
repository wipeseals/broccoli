use super::protocol::{StorageHandler, StorageMsgId, StorageRequest, StorageResponse};

/// Buffer Assigning Type for NandStorageHandler
#[derive(Copy, Clone, Eq, PartialEq)]
#[cfg_attr(test, derive(Debug))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum BufferAssign<const LOGICAL_BLOCKS_PER_BUFFER: usize> {
    /// Empty Buffer
    Empty,
    /// Buffer is being used by Host Data
    /// lbas: Logical Block Addresses List
    HostData {
        lbas: [usize; LOGICAL_BLOCKS_PER_BUFFER],
    },
    /// Buffer is being used by Map Data
    MapData { map_index: usize },
}

/// Buffer Progress
/// This enum is used to check the progress of Buffer
/// - Initial -> Reading -> DecodingAfterRead -> ReadComplete -> Initial
/// - Initial -> EncodingBeforeWrite -> Writing -> WriteComplete -> Initial
#[derive(Copy, Clone, Eq, PartialEq)]
#[cfg_attr(test, derive(Debug))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum BufferProgress<Error: Eq + PartialEq> {
    /// Buffer is Empty (data invalid)
    Initial,
    /// Buffer is being Read (data invalid)
    Reading,
    /// Buffer Read is Complete (data valid, but not yet decoded)
    DecodingAfterRead,
    /// Buffer Read is Complete (data valid)
    ReadComplete { error: Error },
    /// Buffer is being Written (data valid, but not yet encoded)
    EncodingBeforeWrite,
    /// Buffer is being Written (data valid, but not yet written)
    Writing,
    /// Buffer Write is Complete (data valid, written)
    WriteComplete { error: Error },
}

/// Buffer Status
/// This struct is used to check the status of Buffer
#[derive(Copy, Clone, Eq, PartialEq)]
#[cfg_attr(test, derive(Debug))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct BufferStatus<
    const LOGICAL_BLOCKS_PER_BUFFER: usize,
    Error: Eq + PartialEq,
    NandAddr: Eq + PartialEq,
> {
    /// Physical Address
    pub addr: Option<NandAddr>,
    /// Buffer Assigning Type
    pub assign: BufferAssign<LOGICAL_BLOCKS_PER_BUFFER>,
    /// Buffer Progress
    pub progress: BufferProgress<Error>,
}

impl<const LOGICAL_BLOCKS_PER_BUFFER: usize, Error: Eq + PartialEq, NandAddr: Eq + PartialEq>
    BufferStatus<LOGICAL_BLOCKS_PER_BUFFER, Error, NandAddr>
{
    /// Create a new BufferStatus
    pub const fn new() -> Self {
        Self {
            addr: None,
            assign: BufferAssign::Empty,
            progress: BufferProgress::Initial,
        }
    }
}

/// Flash Storage Controller for FTL
/// Buffer Size == Nand Page Size
/// Logical Block Size <= Nand Page Size
pub struct NandStorageHandler<
    const LOGICAL_BLOCK_SIZE: usize,
    const NAND_PAGE_TOTAL_SIZE: usize,
    const NAND_PAGE_METADATA_SIZE: usize,
    const READ_BUFFER_N: usize,
    const WRITE_BUFFER_N: usize,
> {
    internal_read_status: BufferStatus<LOGICAL_BLOCK_SIZE, u8, usize>,
    internal_write_status: BufferStatus<LOGICAL_BLOCK_SIZE, u8, usize>,
    host_read_statuses: [BufferStatus<LOGICAL_BLOCK_SIZE, u8, usize>; READ_BUFFER_N],
    host_write_statuses: [BufferStatus<LOGICAL_BLOCK_SIZE, u8, usize>; WRITE_BUFFER_N],

    /// Internal Read Buffer (NAND_PAGE_SIZE)
    internal_read_buffer: [u8; NAND_PAGE_TOTAL_SIZE],
    /// Internal Write Buffer (NAND_PAGE_SIZE)
    internal_write_buffer: [u8; NAND_PAGE_TOTAL_SIZE],
    /// Read Buffer (NAND_PAGE_SIZE * READ_BUFFER_N)
    host_read_buffers: [[u8; NAND_PAGE_TOTAL_SIZE]; READ_BUFFER_N],
    /// Write Buffer (NAND_PAGE_SIZE * WRITE_BUFFER_N)
    host_write_buffers: [[u8; NAND_PAGE_TOTAL_SIZE]; WRITE_BUFFER_N],
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
            internal_read_status: BufferStatus::new(),
            internal_write_status: BufferStatus::new(),
            host_read_statuses: [BufferStatus::new(); READ_BUFFER_N],
            host_write_statuses: [BufferStatus::new(); WRITE_BUFFER_N],
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
