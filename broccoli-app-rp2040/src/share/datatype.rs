use bitflags::bitflags;
use core::cmp::{Eq, PartialEq};

use embassy_sync::channel::{DynamicReceiver, DynamicSender};

use broccoli_core::common::{
    io_driver::NandStatusReadResult,
    storage_req::{StorageHandler, StorageRequest, StorageResponse},
};

/// NAND IC Status Output
///
/// | Bit | Description            | Value                      |
/// | --- | ---------------------- | -------------------------- |
/// | 0   | Chip Status0           | Pass:0 , Fail: 1           |
/// | 1   | Chip Status1           | Pass:0 , Fail: 1           |
/// | 2   | -                      | -                          |
/// | 3   | -                      | -                          |
/// | 4   | -                      | -                          |
/// | 5   | Page Buffer Ready/Busy | Ready: 1, Busy: 0          |
/// | 6   | Data Cache Ready/Busy  | Ready: 1, Busy: 0          |
/// | 7   | Write Protect          | Not Protect: 1, Protect: 0 |
bitflags! {
    #[derive(Default, Clone, Copy, PartialEq)]
    pub struct NandStatusReadBitFlags: u8 {
        const CHIP_STATUS0_FAIL = 0b0000_0001;
        const CHIP_STATUS1_FAIL = 0b0000_0010;
        const PAGE_BUFFER_READY = 0b0010_0000;
        const DATA_CACHE_READY = 0b0100_0000;
        const WRITE_PROTECT_DISABLE = 0b1000_0000;
    }
}

impl NandStatusReadBitFlags {
    /// Check if page buffer is ready
    fn is_page_buffer_ready(&self) -> bool {
        !(*self & NandStatusReadBitFlags::PAGE_BUFFER_READY).is_empty()
    }

    /// Check if data cache is ready
    pub fn is_data_cache_ready(&self) -> bool {
        !(*self & NandStatusReadBitFlags::DATA_CACHE_READY).is_empty()
    }
}

impl NandStatusReadResult for NandStatusReadBitFlags {
    fn is_failed(&self) -> bool {
        (!(*self & NandStatusReadBitFlags::CHIP_STATUS0_FAIL).is_empty())
            || (!(*self & NandStatusReadBitFlags::CHIP_STATUS1_FAIL).is_empty())
    }

    fn is_write_protect(&self) -> bool {
        !(*self & NandStatusReadBitFlags::WRITE_PROTECT_DISABLE).is_empty()
    }
}

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
