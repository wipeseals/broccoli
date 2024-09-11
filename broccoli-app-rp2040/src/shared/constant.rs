/// USB device number of blocks
pub const DEBUG_ENABLE_RAM_DISK: bool = true;
/// USB device number of blocks
pub const DEBUG_RAM_DISK_NUM_BLOCKS: usize = 128;

/// Core1 task stack size
pub const CORE1_TASK_STACK_SIZE: usize = 4096;

/// LEDCTRL channel channel size
pub const CHANNEL_LEDCTRL_N: usize = 1;
/// USB Control Transfer to Bulk Transfer channel size
pub const CHANNEL_CTRL_TO_BULK_N: usize = 2;
/// USB Bulk Transfer to Internal Request channel size
pub const CHANNEL_BULK_TO_DATA_REQUEST_N: usize = 4;
/// USB Internal Request to Bulk Transfer channel size
pub const CHANNEL_DATA_RESPONSE_TO_BULK_N: usize = 4;

/// Buffer allocation fail retry duration in microseconds
pub const BUFFER_ALLOCATION_FAIL_RETRY_DURATION_US: u64 = 100;
/// Buffer allocation fail retry count max
pub const BUFFER_ALLOCATION_FAIL_RETRY_COUNT_MAX: u32 = 100;

/// USB device number of blocks
pub const USB_NUM_BLOCKS: usize = if DEBUG_ENABLE_RAM_DISK {
    DEBUG_RAM_DISK_NUM_BLOCKS
} else {
    1024
};
/// USB device block size
pub const USB_BLOCK_SIZE: usize = 512;
/// USB device total size
pub const USB_TOTAL_SIZE: usize = USB_NUM_BLOCKS * USB_BLOCK_SIZE;

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
pub const USB_PACKET_COUNT_PER_LOGICAL_BLOCK: usize = (USB_BLOCK_SIZE / USB_MAX_PACKET_SIZE);
/// USB device vendor ID as a byte array
pub const USB_VENDOR_ID: [u8; 8] = *b"broccoli";
/// USB device product ID as a byte array
pub const USB_PRODUCT_ID: [u8; 16] = *b"wipeseals devapp";
/// USB device version as a byte array
pub const USB_PRODUCT_DEVICE_VERSION: [u8; 4] = *b"0001";

/// USB block size
pub const LOGICAL_BLOCK_SIZE: usize = USB_BLOCK_SIZE;
/// USB block buffer count
/// Write/ReadのOutstanding数分と処理中+1は確保しておく。 USB MSC <-> DataRequest/Response Arbiter 間としては
/// Write/Readは同時には行わないので、Write/ReadのOutstanding数が最大の場合に全てのBufferが使われる
pub const LOGICAL_BLOCK_BUFFER_N: usize =
    if CHANNEL_BULK_TO_DATA_REQUEST_N > CHANNEL_DATA_RESPONSE_TO_BULK_N {
        CHANNEL_BULK_TO_DATA_REQUEST_N + 1
    } else {
        CHANNEL_DATA_RESPONSE_TO_BULK_N + 1
    };
/// NAND page size usable
/// TODO: broccoli-coreの値を参照して決定する。これ以外のパラメータ含む
pub const NAND_PAGE_SIZE_USABLE: usize = 2048;
/// NAND page size metadata
pub const NAND_PAGE_SIZE_METADATA: usize = 128;
/// NAND page size total (usable + metadata)
pub const NAND_PAGE_SIZE_TOTAL: usize = NAND_PAGE_SIZE_USABLE + NAND_PAGE_SIZE_METADATA;
/// NAND page buffer size
pub const NAND_PAGE_BUFFER_SIZE: usize = NAND_PAGE_SIZE_TOTAL;
/// NAND page buffer count
pub const NAND_PAGE_BUFFER_N: usize = 4;
