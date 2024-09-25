use num_enum::{FromPrimitive, IntoPrimitive};

use crate::commander::NandCommander;
use crate::common::io_address::IoAddress;
use crate::common::io_driver::{NandIoDriver, NandStatusReadResult};

use crate::common::storage_req::{
    StorageHandler, StorageMsgId, StorageRequest, StorageResponse, StorageResponseReport,
};
use crate::nand_block::{NandBlockAllocator, NandBlockInfo, NandBlockState, NandBlockStats};

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
    block_allocator: NandBlockAllocator<MAX_CHIP_NUM, NAND_BLOCKS_PER_CHIP>,
    // TODO: Add NAND Map
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
            block_allocator: NandBlockAllocator::new(),
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
                            self.block_allocator.change_state(
                                chip,
                                block,
                                NandBlockState::InitialBad,
                                true,
                            );
                        } else {
                            self.block_allocator.change_state(
                                chip,
                                block,
                                NandBlockState::Free,
                                true,
                            );
                        }
                    }
                    Err(_) => {
                        // エラーが発生した場合は、一応BadBlockに割り当てておく
                        self.block_allocator.change_state(
                            chip,
                            block,
                            NandBlockState::InitialBadByOtherError,
                            true,
                        );
                    }
                }
            }
        }
        // CS1が見つからない場合、NotMountedで埋めておく
        for chip in num_cs..MAX_CHIP_NUM {
            for block in 0..NAND_BLOCKS_PER_CHIP {
                self.block_allocator
                    .change_state(chip, block, NandBlockState::NotMounted, true);
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
        ReqTag: Eq + PartialEq,
        const MAX_CHIP_NUM: usize,
        const LOGICAL_BLOCK_SIZE: usize,
        const NAND_BLOCKS_PER_CHIP: usize,
    > StorageHandler<ReqTag, LOGICAL_BLOCK_SIZE>
    for NandStorageHandler<'d, Addr, Status, Driver, MAX_CHIP_NUM, NAND_BLOCKS_PER_CHIP>
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
