use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;

use crate::config::CHANNEL_USB_TO_LEDCTRL_N;

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
