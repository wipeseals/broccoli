use embassy_sync::channel::{DynamicReceiver, DynamicSender, Receiver};

use crate::{shared::constant::LOGICAL_BLOCK_SIZE, storage::protocol::DataRequestError};

use super::protocol::{DataRequest, DataRequestId, DataResponse};

/// RAM Disk for FTL
pub struct RamDiskSystem<
    'ch,
    ReqTag: Eq + PartialEq,
    const LOGICAL_BLOCK_SIZE: usize,
    const TOTAL_DATA_SIZE: usize,
> {
    /// Storage on RAM
    data: [u8; TOTAL_DATA_SIZE],
    /// Request Channel Receiver
    req_receiver: DynamicReceiver<'ch, DataRequest<ReqTag, LOGICAL_BLOCK_SIZE>>,
    /// Response Channel Sender
    resp_sender: DynamicSender<'ch, DataResponse<ReqTag, LOGICAL_BLOCK_SIZE>>,
}

impl<
        'ch,
        ReqTag: Eq + PartialEq,
        const TOTAL_DATA_SIZE: usize,
        const LOGICAL_BLOCK_SIZE: usize,
    > RamDiskSystem<'ch, ReqTag, LOGICAL_BLOCK_SIZE, TOTAL_DATA_SIZE>
{
    /// Create a new RamDisk
    pub fn new(
        req_receiver: DynamicReceiver<'ch, DataRequest<ReqTag, LOGICAL_BLOCK_SIZE>>,
        resp_sender: DynamicSender<'ch, DataResponse<ReqTag, LOGICAL_BLOCK_SIZE>>,
    ) -> Self {
        Self {
            data: [0; TOTAL_DATA_SIZE],
            req_receiver,
            resp_sender,
        }
    }

    /// Set data to RamDisk
    pub fn set_data<const N: usize>(&mut self, offset_bytes: usize, data: &[u8; N]) {
        self.data[offset_bytes..offset_bytes + N].copy_from_slice(data);
    }

    /// Get data from RamDisk
    pub fn get_data<const N: usize>(&self, offset_bytes: usize) -> &[u8] {
        &self.data[offset_bytes..offset_bytes + N]
    }

    /// Run the RamDisk
    pub async fn run(&mut self) -> ! {
        loop {
            let request = self.req_receiver.receive().await;
            match request.req_id {
                DataRequestId::Setup => {
                    // Setupは何もしない
                    let response = DataResponse::report_setup_success(
                        request.req_tag,
                        TOTAL_DATA_SIZE / LOGICAL_BLOCK_SIZE,
                    );
                    self.resp_sender.send(response).await;
                }
                DataRequestId::Echo => {
                    // Echoは何もしない
                    let response = DataResponse::echo(request.req_tag);
                    self.resp_sender.send(response).await;
                }
                DataRequestId::Read => {
                    let mut resp = DataResponse::read(request.req_tag, [0; LOGICAL_BLOCK_SIZE]);

                    let ram_offset_start = request.lba * LOGICAL_BLOCK_SIZE;
                    let ram_offset_end = ram_offset_start + LOGICAL_BLOCK_SIZE;

                    if ram_offset_end > self.data.len() {
                        resp.error = Some(DataRequestError::OutOfRange { lba: request.lba });
                    } else {
                        // データをRAM Diskからコピー
                        resp.data
                            .as_mut()
                            .copy_from_slice(&self.data[ram_offset_start..ram_offset_end]);
                    }
                    // 応答
                    self.resp_sender.send(resp).await;
                }
                DataRequestId::Write => {
                    let mut resp = DataResponse::write(request.req_tag);

                    let ram_offset_start = request.lba * LOGICAL_BLOCK_SIZE;
                    let ram_offset_end = ram_offset_start + LOGICAL_BLOCK_SIZE;

                    // 範囲外応答
                    if ram_offset_end > self.data.len() {
                        resp.error = Some(DataRequestError::OutOfRange { lba: request.lba })
                    } else {
                        // データをRAM Diskにコピーしてから応答
                        self.data[ram_offset_start..ram_offset_end]
                            .copy_from_slice(request.data.as_ref());
                    }
                    // 応答
                    self.resp_sender.send(resp).await;
                }
                DataRequestId::Flush => {
                    // Flushは何もしない
                    let response = DataResponse::flush(request.req_tag);
                    self.resp_sender.send(response).await;
                }
            }
        }
    }
}
