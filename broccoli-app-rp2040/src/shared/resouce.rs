use core::cell::RefCell;
use core::sync::atomic::AtomicBool;

use crate::{
    shared::constant::*,
    storage::protocol::{StorageRequest, StorageResponse},
};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_sync::mutex::Mutex;
use once_cell::sync::Lazy;

use super::datatype::MscReqTag;

/// Bulk Transfer -> Internal Request Channel
pub static CHANNEL_USB_BULK_TO_STORAGE_REQUEST: Channel<
    CriticalSectionRawMutex,
    StorageRequest<MscReqTag, USB_MSC_LOGICAL_BLOCK_SIZE>,
    CHANNEL_USB_BULK_TO_STORAGE_REQUEST_N,
> = Channel::new();

/// Internal Request -> Bulk Transfer Channel
pub static CHANNEL_STORAGE_RESPONSE_TO_USB_BULK: Channel<
    CriticalSectionRawMutex,
    StorageResponse<MscReqTag, USB_MSC_LOGICAL_BLOCK_SIZE>,
    CHANNEL_STORAGE_RESPONSE_TO_BULK_N,
> = Channel::new();
