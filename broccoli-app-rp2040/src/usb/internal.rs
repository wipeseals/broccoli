/// General Data Buffer
pub struct DataBufferIdentify {
    pub tag: u32,
}
/// Internal Transfer Request ID
pub enum InternalTransferRequestId {
    Echo,
    Read,
    Write,
    Flush,
}
/// Internal Transfer Error Code
pub enum InternalTransferErrorCode {
    General,
    InvalidRequest,
    DataError,
    NoData,
    OutOfRange,
}
pub enum InternalTransferResponseStatus {
    Success,
    Error { code: InternalTransferErrorCode },
}

/// Internal Transfer Request
pub struct InternalTransferRequest {
    /// Request ID
    pub req_id: InternalTransferRequestId,
    /// Requester Tag (for response)
    pub requester_tag: u32,
    /// Data Buffer ID
    pub data_buf_id: Option<DataBufferIdentify>,
}

/// Internal Transfer Response
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
