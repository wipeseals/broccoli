use core::cmp::{Eq, PartialEq};

use embassy_sync::channel::{DynamicReceiver, DynamicSender};

use broccoli_core::storage::protocol::{StorageHandler, StorageRequest, StorageResponse};

/// USB MSC <--> Storage Request Tag
#[derive(Copy, Clone, Eq, PartialEq, defmt::Format)]
pub struct MscReqTag {
    /// CBW dCBWTag
    cbw_tag: u32,
    /// sequence number
    seq_num: u32,
}

impl MscReqTag {
    pub fn new(cbw_tag: u32, seq_num: u32) -> Self {
        Self { cbw_tag, seq_num }
    }
}
/// Channel <-> StorageHandler Dispatcher
/// This struct is used to dispatch StorageHandler from Channel.
pub struct StorageHandleDispatcher<
    'ch,
    ReqTag: Eq + PartialEq,
    const LOGICAL_BLOCK_SIZE: usize,
    Handler: StorageHandler<ReqTag, LOGICAL_BLOCK_SIZE>,
> {
    /// Storage Handler
    handler: Handler,
    /// Request Channel Receiver
    req_receiver: DynamicReceiver<'ch, StorageRequest<ReqTag, LOGICAL_BLOCK_SIZE>>,
    /// Response Channel Sender
    resp_sender: DynamicSender<'ch, StorageResponse<ReqTag, LOGICAL_BLOCK_SIZE>>,
}

impl<
        'ch,
        ReqTag: Eq + PartialEq,
        const LOGICAL_BLOCK_SIZE: usize,
        Handler: StorageHandler<ReqTag, LOGICAL_BLOCK_SIZE>,
    > StorageHandleDispatcher<'ch, ReqTag, LOGICAL_BLOCK_SIZE, Handler>
{
    /// Create a new StorageHandleDispatch
    pub fn new(
        handler: Handler,
        req_receiver: DynamicReceiver<'ch, StorageRequest<ReqTag, LOGICAL_BLOCK_SIZE>>,
        resp_sender: DynamicSender<'ch, StorageResponse<ReqTag, LOGICAL_BLOCK_SIZE>>,
    ) -> Self {
        Self {
            handler,
            req_receiver,
            resp_sender,
        }
    }

    /// Dispatch Request
    pub async fn run(&mut self) -> ! {
        loop {
            let req = self.req_receiver.receive().await;
            let resp = self.handler.request(req).await;
            self.resp_sender.send(resp).await;
        }
    }
}
