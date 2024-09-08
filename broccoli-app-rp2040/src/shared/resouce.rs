use core::cell::RefCell;
use core::sync::atomic::AtomicBool;

use crate::ftl::buffer::SharedBufferManager;
use crate::shared::constant::*;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_sync::mutex::Mutex;
use once_cell::sync::Lazy;

use super::datatype::{LedState, MscDataTransferTag};

/// Shared buffer manager for logical block buffer
pub static LOGICAL_BLOCK_SHARED_BUFFER_MANAGER: Lazy<
    Mutex<
        CriticalSectionRawMutex,
        SharedBufferManager<MscDataTransferTag, LOGICAL_BLOCK_SIZE, LOGICAL_BLOCK_BUFFER_N>,
    >,
> = Lazy::new(|| Mutex::new(SharedBufferManager::new()));

/// Shared buffer manager for NAND page buffer
pub static NAND_PAGE_SHARED_BUFFER_MANAGER: Lazy<
    Mutex<
        CriticalSectionRawMutex,
        SharedBufferManager<MscDataTransferTag, NAND_PAGE_BUFFER_SIZE, NAND_PAGE_BUFFER_N>,
    >,
> = Lazy::new(|| Mutex::new(SharedBufferManager::new()));

pub static CHANNEL_USB_TO_LEDCTRL: Channel<CriticalSectionRawMutex, LedState, CHANNEL_LEDCTRL_N> =
    Channel::new();
