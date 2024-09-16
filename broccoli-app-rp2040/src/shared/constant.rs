/// USB device number of blocks
pub const DEBUG_ENABLE_RAM_DISK: bool = true;
/// USB device number of blocks (for debug)
pub const DEBUG_RAM_DISK_NUM_BLOCKS: usize = 16;

/// Core1 task stack size
pub const CORE1_TASK_STACK_SIZE: usize = 128 * 1024; // TODO: LBA -> Block Address変換とWrite/Read Bufferで調整

/// USB Control Transfer to Bulk Transfer channel size
pub const CHANNEL_CTRL_TO_BULK_N: usize = 2;
/// USB Bulk Transfer to Internal Request channel size
pub const CHANNEL_USB_BULK_TO_STORAGE_REQUEST_N: usize = 4;
/// USB Internal Request to Bulk Transfer channel size
pub const CHANNEL_STORAGE_RESPONSE_TO_BULK_N: usize = 4;

/// USB device number of blocks
pub const USB_NUM_BLOCKS: usize = if DEBUG_ENABLE_RAM_DISK {
    DEBUG_RAM_DISK_NUM_BLOCKS
} else {
    1024
};
/// USB device block size
pub const USB_MSC_LOGICAL_BLOCK_SIZE: usize = 512;
/// USB device total size
pub const USB_MSC_TOTAL_CAPACITY_BYTES: usize = USB_NUM_BLOCKS * USB_MSC_LOGICAL_BLOCK_SIZE;

/// USB device vendor ID
pub const USB_VID: u16 = 0xc0de;
/// USB device product ID
pub const USB_PID: u16 = 0xcafe;
/// USB device manufacturer string
pub const USB_MANUFACTURER: &str = "wipeseals";
/// USB device product string
pub const USB_PRODUCT: &str = "broccoli";
/// USB device serial number string
pub const USB_SERIAL_NUMBER: &str = "snbroccoli";
/// USB device maximum power consumption in mA
pub const USB_MAX_POWER: u16 = 100;
/// USB device maximum packet size
pub const USB_MAX_PACKET_SIZE: usize = 64;
/// USB device packet count per logical block (512byte / 64byte = 8)
pub const USB_PACKET_COUNT_PER_LOGICAL_BLOCK: usize =
    (USB_MSC_LOGICAL_BLOCK_SIZE / USB_MAX_PACKET_SIZE);
/// USB device vendor ID as a byte array
pub const USB_VENDOR_ID: [u8; 8] = *b"broccoli";
/// USB device product ID as a byte array
pub const USB_PRODUCT_ID: [u8; 16] = *b"wipeseals devapp";
/// USB device version as a byte array
pub const USB_PRODUCT_DEVICE_VERSION: [u8; 4] = *b"0001";

/// USB block size
pub const LOGICAL_BLOCK_SIZE: usize = USB_MSC_LOGICAL_BLOCK_SIZE;
/// USB block buffer count
/// Write/ReadのOutstanding数分と処理中+1は確保しておく。 USB MSC <-> DataRequest/Response Arbiter 間としては
/// Write/Readは同時には行わないので、Write/ReadのOutstanding数が最大の場合に全てのBufferが使われる
pub const LOGICAL_BLOCK_BUFFER_N: usize =
    if CHANNEL_USB_BULK_TO_STORAGE_REQUEST_N > CHANNEL_STORAGE_RESPONSE_TO_BULK_N {
        CHANNEL_USB_BULK_TO_STORAGE_REQUEST_N + 1
    } else {
        CHANNEL_STORAGE_RESPONSE_TO_BULK_N + 1
    };
/// NAND page size usable
/// TODO: broccoli-coreの値を参照して決定する。これ以外のパラメータ含む
pub const NAND_PAGE_SIZE_USABLE: usize = 2048;
/// NAND page size metadata
pub const NAND_PAGE_SIZE_METADATA: usize = 128;
/// NAND page size total (usable + metadata)
pub const TOTAL_NAND_PAGE_SIZE: usize = NAND_PAGE_SIZE_USABLE + NAND_PAGE_SIZE_METADATA;
/// NAND page read buffer count (TOTAL_NAND_PAGE_SIZE)
pub const NAND_PAGE_READ_BUFFER_N: usize = 8;
/// NAND page write buffer count (TOTAL_NAND_PAGE_SIZE)
pub const NAND_PAGE_WRITE_BUFFER_N: usize = 8;
