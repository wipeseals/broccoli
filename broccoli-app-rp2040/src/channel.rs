use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;

pub enum LedState {
    On,
    Off,
    Toggle,
}
pub static LEDCONTROLCHANNEL: Channel<CriticalSectionRawMutex, LedState, 4> = Channel::new();
