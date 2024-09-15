use core::cmp::{Eq, PartialEq};
use core::option::{
    Option,
    Option::{None, Some},
};

use trait_variant;

/// Data Transfer Request ID
#[derive(Clone, Copy, Eq, PartialEq, defmt::Format)]
pub enum StorageMsgId {
    Setup = 0,
    Echo = 1,
    Read = 2,
    Write = 3,
    Flush = 4,
}

/// Data Transfer Request
#[derive(Eq, PartialEq, defmt::Format)]
pub struct StorageRequest<ReqTag: Eq + PartialEq, const DATA_SIZE: usize> {
    /// Request ID
    pub message_id: StorageMsgId,
    /// Request Tag
    pub req_tag: ReqTag,
    /// Logical Block Address
    pub lba: usize,
    /// Data (for Write) Channelに使うためにはSized traitを満たす必要がありOption削除
    pub data: [u8; DATA_SIZE],
}

impl<ReqTag: Eq + PartialEq, const DATA_SIZE: usize> StorageRequest<ReqTag, DATA_SIZE> {
    /// Create a new DataRequest for Setup
    pub fn setup(req_tag: ReqTag) -> Self {
        Self {
            message_id: StorageMsgId::Setup,
            req_tag,
            lba: 0,
            data: [0; DATA_SIZE],
        }
    }

    /// Create a new DataRequest for Read
    pub fn read(req_tag: ReqTag, lba: usize) -> Self {
        Self {
            message_id: StorageMsgId::Read,
            req_tag,
            lba,
            data: [0; DATA_SIZE],
        }
    }

    /// Create a new DataRequest for Write
    pub fn write(req_tag: ReqTag, lba: usize, data: [u8; DATA_SIZE]) -> Self {
        Self {
            message_id: StorageMsgId::Write,
            req_tag,
            lba,
            data,
        }
    }

    /// Create a new DataRequest for Flush
    pub fn flush(req_tag: ReqTag) -> Self {
        Self {
            message_id: StorageMsgId::Flush,
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
    ReportSetupSuccess { num_blocks: usize },
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
pub struct StorageResponse<ReqTag: Eq + PartialEq, const DATA_SIZE: usize> {
    /// Request ID (copy from Request)
    pub message_id: StorageMsgId,
    /// Request Tag (copy from Request)
    pub req_tag: ReqTag,
    /// Error Code
    pub error: Option<DataRequestError>,
    /// Data (for Read): Channelに使うためにはSized traitを満たす必要がありOption削除
    pub data: [u8; DATA_SIZE],
}

impl<ReqTag: Eq + PartialEq, const DATA_SIZE: usize> StorageResponse<ReqTag, DATA_SIZE> {
    /// Create a new DataResponse for Setup
    pub fn setup(req_tag: ReqTag) -> Self {
        Self {
            message_id: StorageMsgId::Setup,
            req_tag,
            error: None,
            data: [0; DATA_SIZE],
        }
    }

    /// Create a new DataResponse for Setup Success
    pub fn report_setup_success(req_tag: ReqTag, num_blocks: usize) -> Self {
        Self {
            message_id: StorageMsgId::Setup,
            req_tag,
            error: Some(DataRequestError::ReportSetupSuccess { num_blocks }),
            data: [0; DATA_SIZE],
        }
    }

    /// Create a new DataResponse for Echo
    pub fn echo(req_tag: ReqTag) -> Self {
        Self {
            message_id: StorageMsgId::Echo,
            req_tag,
            error: None,
            data: [0; DATA_SIZE],
        }
    }

    /// Create a new DataResponse for Read
    pub fn read(req_tag: ReqTag, data: [u8; DATA_SIZE]) -> Self {
        Self {
            message_id: StorageMsgId::Read,
            req_tag,
            error: None,
            data,
        }
    }

    /// Create a new DataResponse for Write
    pub fn write(req_tag: ReqTag) -> Self {
        Self {
            message_id: StorageMsgId::Write,
            req_tag,
            error: None,
            data: [0; DATA_SIZE],
        }
    }

    /// Create a new DataResponse for Flush
    pub fn flush(req_tag: ReqTag) -> Self {
        Self {
            message_id: StorageMsgId::Flush,
            req_tag,
            error: None,
            data: [0; DATA_SIZE],
        }
    }
}
