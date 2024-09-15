use embassy_sync::channel::{DynamicReceiver, DynamicSender, Receiver};

use crate::{ftl::request::DataRequestError, shared::constant::LOGICAL_BLOCK_SIZE};

use super::request::{DataRequest, DataRequestId, DataResponse};

pub struct RamDisk<
    'ch,
    ReqTag: Eq + PartialEq,
    const LOGICAL_BLOCK_SIZE: usize,
    const TOTAL_DATA_SIZE: usize,
> {
    /// Storage on RAM
    data: [u8; TOTAL_DATA_SIZE],
    /// Request Channel Receiver
    receiver: DynamicReceiver<'ch, DataRequest<ReqTag, LOGICAL_BLOCK_SIZE>>,
    /// Response Channel Sender
    sender: DynamicSender<'ch, DataResponse<ReqTag, LOGICAL_BLOCK_SIZE>>,
}

impl<
        'ch,
        ReqTag: Eq + PartialEq,
        const TOTAL_DATA_SIZE: usize,
        const LOGICAL_BLOCK_SIZE: usize,
    > RamDisk<'ch, ReqTag, LOGICAL_BLOCK_SIZE, TOTAL_DATA_SIZE>
{
    /// Create a new RamDisk
    pub fn new(
        receiver: DynamicReceiver<'ch, DataRequest<ReqTag, LOGICAL_BLOCK_SIZE>>,
        sender: DynamicSender<'ch, DataResponse<ReqTag, LOGICAL_BLOCK_SIZE>>,
    ) -> Self {
        Self {
            data: [0; TOTAL_DATA_SIZE],
            receiver,
            sender,
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
            let request = self.receiver.receive().await;
            match request.req_id {
                DataRequestId::Setup => {
                    // Setupは何もしない
                    let response = DataResponse::setup(request.req_tag);
                    self.sender.send(response).await;
                }
                DataRequestId::Echo => {
                    // Echoは何もしない
                    let response = DataResponse::echo(request.req_tag);
                    self.sender.send(response).await;
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
                    self.sender.send(resp).await;
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
                    self.sender.send(resp).await;
                }
                DataRequestId::Flush => {
                    // Flushは何もしない
                    let response = DataResponse::flush(request.req_tag);
                    self.sender.send(response).await;
                }
            }
        }
    }
}
