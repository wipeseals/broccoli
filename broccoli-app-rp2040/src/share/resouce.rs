use core::cell::RefCell;
use core::sync::atomic::AtomicBool;

use crate::share::constant::*;
use crate::usb::msc::BulkTransferRequest;
use broccoli_core::common::storage_req::{StorageRequest, StorageResponse};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_sync::mutex::Mutex;
use once_cell::sync::Lazy;

use super::datatype::MscReqTag;

// Control Transfer -> Bulk Transfer Channel
pub static CHANNEL_USB_CTRL_TO_USB_BULK: Channel<
    CriticalSectionRawMutex,
    BulkTransferRequest,
    CHANNEL_CTRL_TO_BULK_N,
> = Channel::new();

/// Bulk Transfer -> Storage Request Channel
pub static CHANNEL_USB_BULK_TO_STORAGE_REQUEST: Channel<
    CriticalSectionRawMutex,
    StorageRequest<MscReqTag, USB_LOGICAL_BLOCK_SIZE>,
    CHANNEL_USB_BULK_TO_STORAGE_REQUEST_N,
> = Channel::new();

/// Storage Response -> Bulk Transfer Channel
pub static CHANNEL_STORAGE_RESPONSE_TO_USB_BULK: Channel<
    CriticalSectionRawMutex,
    StorageResponse<MscReqTag, USB_LOGICAL_BLOCK_SIZE>,
    CHANNEL_STORAGE_RESPONSE_TO_BULK_N,
> = Channel::new();
