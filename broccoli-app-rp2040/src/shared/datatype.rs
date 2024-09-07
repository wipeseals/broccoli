/// USB MSC <--> Internal Request Data Transfer Channel
#[derive(Copy, Clone, Eq, PartialEq, defmt::Format)]
pub struct MscDataTransferTag {
    /// CBW dCBWTag
    cbw_tag: u32,
    /// sequence number
    seq_num: u32,
    /// num of request/response
    msg_num: u32,
}

impl MscDataTransferTag {
    pub fn new(cbw_tag: u32, seq_num: u32, msg_num: u32) -> Self {
        Self {
            cbw_tag,
            seq_num,
            msg_num,
        }
    }
}
/// LED Illumination State
pub enum LedState {
    On,
    Off,
    Toggle,
}
