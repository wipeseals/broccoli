/// Data Transfer Request ID
#[derive(Eq, PartialEq, defmt::Format)]
pub enum DataRequest<'buffer, ReqTag: Eq + PartialEq, const DATA_SIZE: usize> {
    /// Setup Request
    /// 初期化時に使用する。NAND IOやDMACなどの初期化を行う。これの応答まではUSB Endpointを有効にしない
    Setup { req_tag: ReqTag },
    /// Echo Request
    /// 何もせずに応答する
    Echo { req_tag: ReqTag },

    /// Read Request
    /// 指定されたLBAのデータを返す。1要求に対し1回の応答が返る
    Read {
        req_tag: ReqTag,
        lba: usize,
        read_buf: &'buffer mut [u8; DATA_SIZE],
    },

    /// Write Request
    /// 指定されたLBAに指定されたBufferのデータを書き込む。1要求に対し1回の応答が返る
    Write {
        req_tag: ReqTag,
        lba: usize,
        write_buf: &'buffer [u8; DATA_SIZE],
    },

    /// Flush Request
    /// Write Requestで要求された書き込みで、未完了のものがあれば完了させる
    Flush { req_tag: ReqTag },
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
pub enum DataResponse<'buffer, ReqTag: Copy + Clone + Eq + PartialEq, const DATA_SIZE: usize> {
    /// Setup Response
    Setup {
        req_tag: ReqTag,
        error: Option<DataRequestError>,
    },
    /// Echo Response
    Echo {
        req_tag: ReqTag,
        error: Option<DataRequestError>,
    },

    /// Read Response
    Read {
        req_tag: ReqTag,
        error: Option<DataRequestError>,
        read_buf: &'buffer [u8; DATA_SIZE],
    },

    /// Write Response
    Write {
        req_tag: ReqTag,
        error: Option<DataRequestError>,
        write_buf: &'buffer [u8; DATA_SIZE],
    },

    /// Flush Response
    Flush {
        req_tag: ReqTag,
        error: Option<DataRequestError>,
    },
}

impl<'a, ReqTag: Copy + Clone + Eq + PartialEq, const DATA_SIZE: usize>
    DataRequest<'a, ReqTag, DATA_SIZE>
{
    pub fn echo(req_tag: ReqTag) -> Self {
        Self::Echo { req_tag }
    }

    pub fn read(req_tag: ReqTag, lba: usize, read_buf: &'a mut [u8; DATA_SIZE]) -> Self {
        Self::Read {
            req_tag,
            lba,
            read_buf,
        }
    }

    pub fn write(req_tag: ReqTag, lba: usize, write_buf: &'a [u8; DATA_SIZE]) -> Self {
        Self::Write {
            req_tag,
            lba,
            write_buf,
        }
    }

    pub fn flush(req_tag: ReqTag) -> Self {
        Self::Flush { req_tag }
    }
}

impl<'buffer, ReqTag: Copy + Clone + Eq + PartialEq, const DATA_SIZE: usize>
    DataResponse<'buffer, ReqTag, DATA_SIZE>
{
    pub fn echo(req_tag: ReqTag, error: Option<DataRequestError>) -> Self {
        Self::Echo { req_tag, error }
    }

    pub fn read(
        req_tag: ReqTag,
        error: Option<DataRequestError>,
        read_buf: &'buffer [u8; DATA_SIZE],
    ) -> Self {
        Self::Read {
            req_tag,
            error,
            read_buf,
        }
    }

    pub fn write(
        req_tag: ReqTag,
        error: Option<DataRequestError>,
        write_buf: &'buffer [u8; DATA_SIZE],
    ) -> Self {
        Self::Write {
            req_tag,
            error,
            write_buf,
        }
    }

    pub fn flush(req_tag: ReqTag, error: Option<DataRequestError>) -> Self {
        Self::Flush { req_tag, error }
    }
}
