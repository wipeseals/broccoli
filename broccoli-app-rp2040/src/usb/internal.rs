/// General Data Buffer
#[derive(Copy, Clone, Eq, PartialEq, defmt::Format)]
pub struct DataBufferIdentify {
    pub tag: u32,
}
/// Internal Transfer Request ID
#[derive(Copy, Clone, Eq, PartialEq, defmt::Format)]
pub enum InternalTransferRequestId {
    Echo,
    Read,
    Write,
    Flush,
}
/// Internal Transfer Error Code
#[derive(Copy, Clone, Eq, PartialEq, defmt::Format)]
pub enum InternalTransferErrorCode {
    General,
    InvalidRequest,
    DataError,
    NoData,
    OutOfRange,
    NotImplemented,
}
#[derive(Copy, Clone, Eq, PartialEq, defmt::Format)]
pub enum InternalTransferResponseStatus {
    Success,
    Error { code: InternalTransferErrorCode },
}

/// Internal Transfer Request
#[derive(Copy, Clone, Eq, PartialEq, defmt::Format)]
pub struct InternalTransferRequest {
    /// Request ID
    pub req_id: InternalTransferRequestId,
    /// Requester Tag (for response)
    pub requester_tag: u32,
    /// Data Buffer ID
    pub data_buf_id: Option<DataBufferIdentify>,
}

/// Internal Transfer Response
#[derive(Copy, Clone, Eq, PartialEq, defmt::Format)]
pub struct InternalTransferResponse {
    /// Request ID
    pub req_id: InternalTransferRequestId,
    /// Requester Tag (for response)
    pub requester_tag: u32,
    /// Data Buffer ID
    pub data_buf_id: Option<DataBufferIdentify>,
    /// Response Status
    pub resp_status: InternalTransferResponseStatus,
}

impl InternalTransferRequest {
    pub fn new(
        req_id: InternalTransferRequestId,
        requester_tag: u32,
        data_buf_id: Option<DataBufferIdentify>,
    ) -> Self {
        Self {
            req_id,
            requester_tag,
            data_buf_id,
        }
    }
}
impl InternalTransferResponse {
    pub fn new(
        req_id: InternalTransferRequestId,
        requester_tag: u32,
        data_buf_id: Option<DataBufferIdentify>,
        resp_status: InternalTransferResponseStatus,
    ) -> Self {
        Self {
            req_id,
            requester_tag,
            data_buf_id,
            resp_status,
        }
    }
}
