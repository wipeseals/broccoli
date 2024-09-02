use super::buffer::BufferIdentify;

/// Internal Transfer Request ID
#[derive(Copy, Clone, Eq, PartialEq, defmt::Format)]
pub enum FtlReqId {
    Echo,
    Read,
    Write,
    Flush,
}
/// Internal Transfer Error Code
#[derive(Copy, Clone, Eq, PartialEq, defmt::Format)]
pub enum FtlErrorCode {
    General,
    InvalidRequest,
    DataError,
    NoData,
    OutOfRange,
    NotImplemented,
}
#[derive(Copy, Clone, Eq, PartialEq, defmt::Format)]
pub enum FtlRespStatus {
    Success,
    Error { code: FtlErrorCode },
}

/// Internal Transfer Request
#[derive(Copy, Clone, Eq, PartialEq, defmt::Format)]
pub struct FtlReq {
    /// Request ID
    pub req_id: FtlReqId,
    /// Requester Tag (for response)
    pub requester_tag: u32,
    /// Data Buffer ID
    pub data_buf_id: Option<BufferIdentify>,
}

/// Internal Transfer Response
#[derive(Copy, Clone, Eq, PartialEq, defmt::Format)]
pub struct FtlResp {
    /// Request ID
    pub req_id: FtlReqId,
    /// Requester Tag (for response)
    pub requester_tag: u32,
    /// Data Buffer ID
    pub data_buf_id: Option<BufferIdentify>,
    /// Response Status
    pub resp_status: FtlRespStatus,
}

impl FtlReq {
    pub fn new(req_id: FtlReqId, requester_tag: u32, data_buf_id: Option<BufferIdentify>) -> Self {
        Self {
            req_id,
            requester_tag,
            data_buf_id,
        }
    }
}
impl FtlResp {
    pub fn new(
        req_id: FtlReqId,
        requester_tag: u32,
        data_buf_id: Option<BufferIdentify>,
        resp_status: FtlRespStatus,
    ) -> Self {
        Self {
            req_id,
            requester_tag,
            data_buf_id,
            resp_status,
        }
    }
}
