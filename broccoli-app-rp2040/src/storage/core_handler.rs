use embassy_sync::channel::{DynamicReceiver, DynamicSender};

use super::protocol::{StorageHandler, StorageMsgId, StorageRequest, StorageResponse};

/// Flash Storage Controller for FTL
pub struct StorageCoreHandler<
    const LOGICAL_BLOCK_SIZE: usize,
    const NAND_PAGE_SIZE: usize,
    const READ_BUFFER_N: usize,
    const WRITE_BUFFER_N: usize,
> {
    /// Read Buffer (NAND_PAGE_SIZE * READ_BUFFER_N)
    read_buffer: [[u8; NAND_PAGE_SIZE]; READ_BUFFER_N],
    /// Write Buffer (NAND_PAGE_SIZE * WRITE_BUFFER_N)
    write_buffer: [[u8; NAND_PAGE_SIZE]; WRITE_BUFFER_N],
}

impl<
        const LOGICAL_BLOCK_SIZE: usize,
        const NAND_PAGE_SIZE: usize,
        const READ_BUFFER_N: usize,
        const WRITE_BUFFER_N: usize,
    > StorageCoreHandler<LOGICAL_BLOCK_SIZE, NAND_PAGE_SIZE, READ_BUFFER_N, WRITE_BUFFER_N>
{
    /// Create a new DataBuffer
    pub fn new() -> Self {
        Self {
            read_buffer: [[0; NAND_PAGE_SIZE]; READ_BUFFER_N],
            write_buffer: [[0; NAND_PAGE_SIZE]; WRITE_BUFFER_N],
        }
    }

    /// Get the number of blocks per ReadBuffer
    pub const fn num_blocks_per_write_buffer(&self) -> usize {
        NAND_PAGE_SIZE / LOGICAL_BLOCK_SIZE
    }

    /// Get the number of blocks per WriteBuffer
    pub const fn num_blocks_per_read_buffer(&self) -> usize {
        NAND_PAGE_SIZE / LOGICAL_BLOCK_SIZE
    }

    /// Get the total number of blocks in ReadBuffer
    pub const fn total_blocks_in_write_buffer(&self) -> usize {
        WRITE_BUFFER_N * self.num_blocks_per_write_buffer()
    }

    /// Get the total number of blocks in WriteBuffer
    pub const fn total_blocks_in_read_buffer(&self) -> usize {
        READ_BUFFER_N * self.num_blocks_per_read_buffer()
    }
}

impl<
        ReqTag: Eq + PartialEq,
        const LOGICAL_BLOCK_SIZE: usize,
        const NAND_PAGE_SIZE: usize,
        const READ_BUFFER_N: usize,
        const WRITE_BUFFER_N: usize,
    > StorageHandler<ReqTag, LOGICAL_BLOCK_SIZE>
    for StorageCoreHandler<LOGICAL_BLOCK_SIZE, NAND_PAGE_SIZE, READ_BUFFER_N, WRITE_BUFFER_N>
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
