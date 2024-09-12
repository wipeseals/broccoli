use core::cmp::{Eq, PartialEq};
use core::option::{
    Option,
    Option::{None, Some},
};

/// Data Transfer Request ID
#[derive(Clone, Copy, Eq, PartialEq, defmt::Format)]
pub enum DataRequestId {
    Setup = 0,
    Echo = 1,
    Read = 2,
    Write = 3,
    Flush = 4,
}

/// Data Transfer Request
#[derive(Eq, PartialEq, defmt::Format)]
pub struct DataRequest<ReqTag: Eq + PartialEq, const DATA_SIZE: usize> {
    /// Request ID
    pub req_id: DataRequestId,
    /// Request Tag
    pub req_tag: ReqTag,
    /// Logical Block Address
    pub lba: usize,
    /// Data (for Write) Channelに使うためにはSized traitを満たす必要がありOption削除
    pub data: [u8; DATA_SIZE],
}

impl<ReqTag: Eq + PartialEq, const DATA_SIZE: usize> DataRequest<ReqTag, DATA_SIZE> {
    /// Create a new DataRequest for Setup
    pub fn setup(req_tag: ReqTag) -> Self {
        Self {
            req_id: DataRequestId::Setup,
            req_tag,
            lba: 0,
            data: [0; DATA_SIZE],
        }
    }

    /// Create a new DataRequest for Read
    pub fn read(req_tag: ReqTag, lba: usize) -> Self {
        Self {
            req_id: DataRequestId::Read,
            req_tag,
            lba,
            data: [0; DATA_SIZE],
        }
    }

    /// Create a new DataRequest for Write
    pub fn write(req_tag: ReqTag, lba: usize, data: [u8; DATA_SIZE]) -> Self {
        Self {
            req_id: DataRequestId::Write,
            req_tag,
            lba,
            data,
        }
    }

    /// Create a new DataRequest for Flush
    pub fn flush(req_tag: ReqTag) -> Self {
        Self {
            req_id: DataRequestId::Flush,
            req_tag,
            lba: 0,
            data: [0; DATA_SIZE],
        }
    }
}

/// Internal Transfer Error Code
#[derive(Copy, Clone, Eq, PartialEq, defmt::Format)]
pub enum DataRequestError {
    NoError,
    General,
    BufferAllocationFail,
    NandError,
    InvalidRequest,
    DataError,
    NoData,
    OutOfRange { lba: usize },
    NotImplemented,
}

/// Internal Transfer Response
#[derive(Copy, Clone, Eq, PartialEq, defmt::Format)]
pub struct DataResponse<ReqTag: Eq + PartialEq, const DATA_SIZE: usize> {
    /// Request ID
    pub req_id: DataRequestId,
    /// Request Tag
    pub req_tag: ReqTag,
    /// Error Code
    pub error: Option<DataRequestError>,
    /// Data (for Read): Channelに使うためにはSized traitを満たす必要がありOption削除
    pub data: [u8; DATA_SIZE],
}

impl<ReqTag: Eq + PartialEq, const DATA_SIZE: usize> DataResponse<ReqTag, DATA_SIZE> {
    /// Create a new DataResponse for Setup
    pub fn setup(req_tag: ReqTag) -> Self {
        Self {
            req_id: DataRequestId::Setup,
            req_tag,
            error: None,
            data: [0; DATA_SIZE],
        }
    }

    /// Create a new DataResponse for Echo
    pub fn echo(req_tag: ReqTag) -> Self {
        Self {
            req_id: DataRequestId::Echo,
            req_tag,
            error: None,
            data: [0; DATA_SIZE],
        }
    }

    /// Create a new DataResponse for Read
    pub fn read(req_tag: ReqTag, data: [u8; DATA_SIZE]) -> Self {
        Self {
            req_id: DataRequestId::Read,
            req_tag,
            error: None,
            data,
        }
    }

    /// Create a new DataResponse for Write
    pub fn write(req_tag: ReqTag) -> Self {
        Self {
            req_id: DataRequestId::Write,
            req_tag,
            error: None,
            data: [0; DATA_SIZE],
        }
    }

    /// Create a new DataResponse for Flush
    pub fn flush(req_tag: ReqTag) -> Self {
        Self {
            req_id: DataRequestId::Flush,
            req_tag,
            error: None,
            data: [0; DATA_SIZE],
        }
    }
}
