use core::cell::RefCell;
use core::sync::atomic::AtomicBool;

use crate::shared::constant::*;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_sync::mutex::Mutex;
use once_cell::sync::Lazy;

use super::datatype::{LedState, MscDataTransferTag};

pub static CHANNEL_USB_TO_LEDCTRL: Channel<CriticalSectionRawMutex, LedState, CHANNEL_LEDCTRL_N> =
    Channel::new();
