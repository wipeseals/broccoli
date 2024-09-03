use core::cell::RefCell;
use core::sync::atomic::AtomicBool;

use crate::config::*;
use crate::ftl::buffer::SharedBufferManager;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_sync::mutex::Mutex;
use once_cell::sync::Lazy;

/// Shared buffer manager for logical block buffer
pub static LOGICAL_BLOCK_SHARED_BUFFERS: Lazy<
    SharedBufferManager<LOGICAL_BLOCK_SIZE, LOGICAL_BLOCK_BUFFER_N>,
> = Lazy::new(SharedBufferManager::new);

/// Shared buffer manager for NAND page buffer
pub static NAND_PAGE_SHARED_BUFFERS: Lazy<
    SharedBufferManager<NAND_PAGE_BUFFER_SIZE, NAND_PAGE_BUFFER_N>,
> = Lazy::new(SharedBufferManager::new);

pub enum LedState {
    On,
    Off,
    Toggle,
}
pub static CHANNEL_USB_TO_LEDCTRL: Channel<
    CriticalSectionRawMutex,
    LedState,
    CHANNEL_USB_TO_LEDCTRL_N,
> = Channel::new();
