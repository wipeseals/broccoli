use super::buffer::BufferIdentify;

/// Data Transfer Request ID
#[derive(Copy, Clone, Eq, PartialEq, defmt::Format)]
pub enum DataRequest<ReqTag, const DATA_SIZE: usize> {
    /// Echo Request
    /// 何もせずに応答する
    Echo { req_tag: ReqTag },

    /// Read Request
    /// 指定されたLBAから指定されたブロック数を読み出す。1要求に対しblock_count回数分の応答が返る
    /// DataBufferは連続読み出しを想定し、それぞれ確保して応答に含める。応答に乗せた時点でBufferの所有権は移動する
    Read {
        req_tag: ReqTag,
        lba: u32,
        block_count: u32,
    },

    /// Write Request
    /// 指定されたLBAに指定されたBufferのデータを書き込む。1要求に対し1回の応答が返る
    /// 内部のWriteBufferに乗せた時点で応答を返しても良い。ただし、あとからFlushを要求された場合は、その時点で書き込みを行う
    Write {
        req_tag: ReqTag,
        lba: u32,
        write_buf_id: BufferIdentify<DATA_SIZE>,
    },

    /// Flush Request
    /// Write Requestで要求された書き込みで、未完了のものがあれば完了させる
    Flush { req_tag: ReqTag },
}
/// Internal Transfer Error Code
#[derive(Copy, Clone, Eq, PartialEq, defmt::Format)]
pub enum DataRequestError {
    General,
    InvalidRequest,
    DataError,
    NoData,
    OutOfRange,
    NotImplemented,
}

/// Internal Transfer Response
#[derive(Copy, Clone, Eq, PartialEq, defmt::Format)]
pub enum DataResponse<ReqTag, const DATA_SIZE: usize> {
    /// Echo Response
    /// Echo Requestに対する応答
    Echo {
        req_tag: ReqTag,
        error: Option<DataRequestError>,
    },

    /// Read Response
    /// Read Requestに対する応答
    Read {
        req_tag: ReqTag,
        read_buf_id: BufferIdentify<DATA_SIZE>,
        data_count: u32,
        error: Option<DataRequestError>,
    },

    /// Write Response
    /// Write Requestに対する応答
    Write {
        req_tag: ReqTag,
        error: Option<DataRequestError>,
    },

    /// Flush Response
    /// Flush Requestに対する応答
    Flush {
        req_tag: ReqTag,
        error: Option<DataRequestError>,
    },
}

impl<ReqTag, const DATA_SIZE: usize> DataRequest<ReqTag, DATA_SIZE> {
    pub fn echo(req_tag: ReqTag) -> Self {
        Self::Echo { req_tag }
    }

    pub fn read(req_tag: ReqTag, lba: u32, block_count: u32) -> Self {
        Self::Read {
            req_tag,
            lba,
            block_count,
        }
    }

    pub fn write(req_tag: ReqTag, lba: u32, write_buf_id: BufferIdentify<DATA_SIZE>) -> Self {
        Self::Write {
            req_tag,
            lba,
            write_buf_id,
        }
    }

    pub fn flush(req_tag: ReqTag) -> Self {
        Self::Flush { req_tag }
    }
}

impl<ReqTag, const DATA_SIZE: usize> DataResponse<ReqTag, DATA_SIZE> {
    pub fn echo(req_tag: ReqTag, error: Option<DataRequestError>) -> Self {
        Self::Echo { req_tag, error }
    }

    pub fn read(
        req_tag: ReqTag,
        read_buf_id: BufferIdentify<DATA_SIZE>,
        data_count: u32,
        error: Option<DataRequestError>,
    ) -> Self {
        Self::Read {
            req_tag,
            read_buf_id,
            data_count,
            error,
        }
    }

    pub fn write(req_tag: ReqTag, error: Option<DataRequestError>) -> Self {
        Self::Write { req_tag, error }
    }

    pub fn flush(req_tag: ReqTag, error: Option<DataRequestError>) -> Self {
        Self::Flush { req_tag, error }
    }
}
