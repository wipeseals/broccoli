use crate::common::storage_req::StorageResponseReport;

use crate::common::storage_req::{StorageHandler, StorageMsgId, StorageRequest, StorageResponse};

/// RAM Disk for FTL
pub struct RamDiskHandler<const LOGICAL_BLOCK_SIZE: usize, const TOTAL_DATA_SIZE: usize> {
    /// Storage on RAM
    data: [u8; TOTAL_DATA_SIZE],
}

impl<const LOGICAL_BLOCK_SIZE: usize, const TOTAL_DATA_SIZE: usize> Default
    for RamDiskHandler<LOGICAL_BLOCK_SIZE, TOTAL_DATA_SIZE>
{
    fn default() -> Self {
        Self::new()
    }
}

impl<const LOGICAL_BLOCK_SIZE: usize, const TOTAL_DATA_SIZE: usize>
    RamDiskHandler<LOGICAL_BLOCK_SIZE, TOTAL_DATA_SIZE>
{
    /// Create a new RamDisk
    pub fn new() -> Self {
        Self {
            data: [0; TOTAL_DATA_SIZE],
        }
    }

    /// Set data to RamDisk
    pub fn set_data<const N: usize>(&mut self, offset_bytes: usize, data: &[u8; N]) {
        self.data[offset_bytes..offset_bytes + N].copy_from_slice(data);
    }

    /// Get data from RamDisk
    pub fn get_data<const N: usize>(&self, offset_bytes: usize) -> &[u8] {
        &self.data[offset_bytes..offset_bytes + N]
    }

    /// Set FAT12 Data to RAM Disk
    /// refs. <https://github.com/hathach/tinyusb/blob/master/examples/device/cdc_msc/src/msc_disk.c#L52>
    #[cfg(feature = "ramdisk_sample_data")]
    #[rustfmt::skip]
    pub fn set_fat12_sample_data(&mut self) {
        let readme_contents = b"Hello, broccoli!\n";
        // LBA0: MBR
        self.set_data(
            0,
            &[
            /// |  0|    1|    2|    3|    4|    5|    6|    7|    8|    9|  0xa| 0xb|  0xc|  0xd|  0xe|  0xf|
                0xEB, 0x3C, 0x90, 0x4D, 0x53, 0x44, 0x4F, 0x53, 0x35, 0x2E, 0x30, 0x00, 0x02, 0x01, 0x01, 0x00, // 0x00
                0x01, 0x10, 0x00, 0x10, 0x00, 0xF8, 0x01, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, // 0x10
                0x00, 0x00, 0x00, 0x00, 0x80, 0x00, 0x29, 0x34, 0x12, 0x00, 0x00, b'B', b'r', b'o', b'c', b'c', // 0x20
                b'o', b'l', b'i', b'M', b'S', b'C', 0x46, 0x41, 0x54, 0x31, 0x32, 0x20, 0x20, 0x20, 0x00, 0x00, // 0x30
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 0x40
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 0x50
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 0x60
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 0x70
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 0x80
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 0x90
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 0xa0
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 0xb0
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 0xc0
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 0xd0
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 0xe0
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x55, 0xaa, // 0xf0
            ],
        );
        // LBA1: FAT12 Table
        self.set_data(512, &[0xF8, 0xFF, 0xFF, 0x00, 0x00]);
        // LBA2: Root Directory
        let flen = (readme_contents.len() - 1) as u8;
        self.set_data(
            1024,
            &[
            /// first entry is volume label
            /// |  0|    1|    2|    3|    4|    5|    6|    7|    8|    9|  0xa| 0xb|  0xc|  0xd|  0xe|  0xf|
                b'B', b'r', b'o', b'c', b'c', b'o', b'l', b'i', b'M', b'S', b'C', 0x08, 0x00, 0x00, 0x00, 0x00, // volume label
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x4F, 0x6D, 0x65, 0x43, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // readme file
                b'R', b'E', b'A', b'D', b'M', b'E', b' ', b' ', b'T', b'X', b'T', 0x20, 0x00, 0xC6, 0x52, 0x6D, // readme file
                b'e', b'C', b'e', b'C', 0x00, 0x00, 0x88, 0x6D, 0x65, 0x43, 0x02, 0x00, flen, 0x00, 0x00, 0x00, // readme file
            ],
        );
        // lba3 readme file
        self.set_data(1536, readme_contents);
    }
}

impl<ReqTag: Eq + PartialEq, const LOGICAL_BLOCK_SIZE: usize, const TOTAL_DATA_SIZE: usize>
    StorageHandler<ReqTag, LOGICAL_BLOCK_SIZE>
    for RamDiskHandler<LOGICAL_BLOCK_SIZE, TOTAL_DATA_SIZE>
{
    /// Request handler
    async fn request(
        &mut self,
        request: StorageRequest<ReqTag, LOGICAL_BLOCK_SIZE>,
    ) -> StorageResponse<ReqTag, LOGICAL_BLOCK_SIZE> {
        match request.message_id {
            StorageMsgId::Setup => {
                // Setupは何もしない
                StorageResponse::report_setup_success(
                    request.req_tag,
                    TOTAL_DATA_SIZE / LOGICAL_BLOCK_SIZE,
                )
            }
            StorageMsgId::Echo => {
                // Echoは何もしない
                StorageResponse::echo(request.req_tag)
            }
            StorageMsgId::Read => {
                let mut resp = StorageResponse::read(request.req_tag, [0; LOGICAL_BLOCK_SIZE]);

                let ram_offset_start = request.lba * LOGICAL_BLOCK_SIZE;
                let ram_offset_end = ram_offset_start + LOGICAL_BLOCK_SIZE;

                if ram_offset_end > self.data.len() {
                    resp.meta_data = Some(StorageResponseReport::OutOfRange { lba: request.lba });
                } else {
                    // データをRAM Diskからコピー
                    resp.data
                        .as_mut()
                        .copy_from_slice(&self.data[ram_offset_start..ram_offset_end]);
                }
                resp
            }
            StorageMsgId::Write => {
                let mut resp = StorageResponse::write(request.req_tag);

                let ram_offset_start = request.lba * LOGICAL_BLOCK_SIZE;
                let ram_offset_end = ram_offset_start + LOGICAL_BLOCK_SIZE;

                // 範囲外応答
                if ram_offset_end > self.data.len() {
                    resp.meta_data = Some(StorageResponseReport::OutOfRange { lba: request.lba })
                } else {
                    // データをRAM Diskにコピーしてから応答
                    self.data[ram_offset_start..ram_offset_end]
                        .copy_from_slice(request.data.as_ref());
                }
                // 応答
                resp
            }
            StorageMsgId::Flush => {
                // Flushは何もしない
                StorageResponse::flush(request.req_tag)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    type StorageRequestTag = u32;

    const LOGICAL_BLOCK_SIZE: usize = 512;
    const TOTAL_DATA_SIZE: usize = 1024;

    #[rstest]
    #[tokio::test]
    #[case(
        StorageRequest::setup(0x00),
        StorageResponse::report_setup_success(0x00, TOTAL_DATA_SIZE / LOGICAL_BLOCK_SIZE)
    )]
    #[case(
        StorageRequest::read(0x01, 0),
        StorageResponse::read(0x01, [0; 512])
    )]
    #[case(
        StorageRequest::write(0x02, 0, [0; 512]),
        StorageResponse::write(0x02)
    )]
    #[case(
        StorageRequest::read(0x03, 0),
        StorageResponse::read(0x03, [0; 512])
    )]
    #[case(StorageRequest::flush(0x04), StorageResponse::flush(0x04))]
    async fn test_check_id_tag(
        #[case] req: StorageRequest<StorageRequestTag, LOGICAL_BLOCK_SIZE>,
        #[case] expected_resp: StorageResponse<StorageRequestTag, LOGICAL_BLOCK_SIZE>,
    ) {
        let mut handler = RamDiskHandler::<LOGICAL_BLOCK_SIZE, TOTAL_DATA_SIZE>::new();

        let resp = handler.request(req).await;
        assert_eq!(resp, expected_resp);
    }

    #[rstest]
    #[tokio::test]
    async fn test_write_read() {
        let mut handler = RamDiskHandler::<LOGICAL_BLOCK_SIZE, TOTAL_DATA_SIZE>::new();
        let mut write_data = [0u8; LOGICAL_BLOCK_SIZE];
        write_data.copy_from_slice(
            &(0..LOGICAL_BLOCK_SIZE)
                .map(|i| (i & 0xff) as u8)
                .collect::<Vec<u8>>(),
        );

        let write_req = StorageRequest::write(0x01, 1, write_data);
        let write_resp = handler.request(write_req).await;
        assert_eq!(write_resp, StorageResponse::write(0x01));

        let read_req = StorageRequest::read(0x02, 1);
        let read_resp = handler.request(read_req).await;
        assert_eq!(read_resp, StorageResponse::read(0x02, write_data));
    }
}
