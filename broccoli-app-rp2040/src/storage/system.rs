use embassy_sync::channel::{DynamicReceiver, DynamicSender};

use super::protocol::{DataRequest, DataRequestId, DataResponse};

/// Flash Storage Controller for FTL
pub struct StorageSystem<
    'ch,
    ReqTag: Eq + PartialEq,
    const LOGICAL_BLOCK_SIZE: usize,
    const NAND_PAGE_SIZE: usize,
    const READ_BUFFER_N: usize,
    const WRITE_BUFFER_N: usize,
> {
    /// Read Buffer (NAND_PAGE_SIZE * READ_BUFFER_N)
    read_buffer: [[u8; NAND_PAGE_SIZE]; READ_BUFFER_N],
    /// Write Buffer (NAND_PAGE_SIZE * WRITE_BUFFER_N)
    write_buffer: [[u8; NAND_PAGE_SIZE]; WRITE_BUFFER_N],
    /// Request Channel Receiver
    req_receiver: DynamicReceiver<'ch, DataRequest<ReqTag, LOGICAL_BLOCK_SIZE>>,
    /// Response Channel Sender
    resp_sender: DynamicSender<'ch, DataResponse<ReqTag, LOGICAL_BLOCK_SIZE>>,
}

impl<
        'ch,
        ReqTag: Eq + PartialEq,
        const LOGICAL_BLOCK_SIZE: usize,
        const NAND_PAGE_SIZE: usize,
        const READ_BUFFER_N: usize,
        const WRITE_BUFFER_N: usize,
    >
    StorageSystem<'ch, ReqTag, LOGICAL_BLOCK_SIZE, NAND_PAGE_SIZE, READ_BUFFER_N, WRITE_BUFFER_N>
{
    /// Create a new DataBuffer
    pub fn new(
        req_receiver: DynamicReceiver<'ch, DataRequest<ReqTag, LOGICAL_BLOCK_SIZE>>,
        resp_sender: DynamicSender<'ch, DataResponse<ReqTag, LOGICAL_BLOCK_SIZE>>,
    ) -> Self {
        Self {
            read_buffer: [[0; NAND_PAGE_SIZE]; READ_BUFFER_N],
            write_buffer: [[0; NAND_PAGE_SIZE]; WRITE_BUFFER_N],
            req_receiver,
            resp_sender,
        }
    }

    pub async fn run(&mut self) -> ! {
        loop {
            let request = self.req_receiver.receive().await;
            match request.req_id {
                DataRequestId::Setup => {
                    // TODO: NAND IOの初期化処理
                    let response = DataResponse::report_setup_success(
                        request.req_tag,
                        ((1024 - 100) * 64 * 2048 / LOGICAL_BLOCK_SIZE), // TODO: 仮の値. NANDの容量とブロックサイズ、管理データ向けに割り当てた容量から計算する
                    );
                    self.resp_sender.send(response).await;
                }
                DataRequestId::Echo => {
                    // Echoは何もしない
                    let response = DataResponse::echo(request.req_tag);
                    self.resp_sender.send(response).await;
                }
                DataRequestId::Read => {
                    // Read
                    // TODO: NANDからデータを読み出す処理
                    let response = DataResponse::read(request.req_tag, [0; LOGICAL_BLOCK_SIZE]);
                    self.resp_sender.send(response).await;
                }
                DataRequestId::Write => {
                    // Write
                    // TODO: NANDにデータを書き込む処理
                    let response = DataResponse::write(request.req_tag);
                    self.resp_sender.send(response).await;
                }
                DataRequestId::Flush => {
                    // Flush
                    // TODO: WriteBufferの内容をNANDに書き込む処理
                    let response = DataResponse::flush(request.req_tag);
                    self.resp_sender.send(response).await;
                }
            }
        }
    }
}
