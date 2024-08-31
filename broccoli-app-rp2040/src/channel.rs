use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;

pub enum LedState {
    On,
    Off,
    Toggle,
}
pub static CHANNEL_USB_TO_LEDCTRL: Channel<CriticalSectionRawMutex, LedState, 4> = Channel::new();
