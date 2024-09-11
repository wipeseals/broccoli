use core::cmp::{Eq, PartialEq};

/// USB MSC <--> Internal Request Data Transfer Channel
#[derive(Copy, Clone, Eq, PartialEq, defmt::Format)]
pub struct MscDataTransferTag {
    /// CBW dCBWTag
    cbw_tag: u32,
    /// sequence number
    seq_num: usize,
}

impl MscDataTransferTag {
    pub fn new(cbw_tag: u32, seq_num: usize) -> Self {
        Self { cbw_tag, seq_num }
    }
}
/// LED Illumination State
pub enum LedState {
    On,
    Off,
    Toggle,
}
