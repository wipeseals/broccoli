/* System Setup */

/// Core1 task stack size
pub const CORE1_TASK_STACK_SIZE: usize = 128 * 1024; // TODO: LBA -> Block Address変換とWrite/Read Bufferで調整

/// USB Control Transfer to Bulk Transfer channel size
pub const CHANNEL_CTRL_TO_BULK_N: usize = 2;
/// USB Bulk Transfer to Internal Request channel size
pub const CHANNEL_USB_BULK_TO_STORAGE_REQUEST_N: usize = 4;
/// USB Internal Request to Bulk Transfer channel size
pub const CHANNEL_STORAGE_RESPONSE_TO_BULK_N: usize = 4;

/* USB Setup */

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
/// USB device block size
pub const USB_LOGICAL_BLOCK_SIZE: usize = 512;
/// USB device packet count per logical block (512byte / 64byte = 8)
pub const USB_PACKET_COUNT_PER_LOGICAL_BLOCK: usize =
    (USB_LOGICAL_BLOCK_SIZE / USB_MAX_PACKET_SIZE);
/// USB device vendor ID as a byte array
pub const USB_VENDOR_ID: [u8; 8] = *b"broccoli";
/// USB device product ID as a byte array
pub const USB_PRODUCT_ID: [u8; 16] = *b"wipeseals devapp";
/// USB device version as a byte array
pub const USB_PRODUCT_DEVICE_VERSION: [u8; 4] = *b"0001";

/* NAND Setup */

/// NAND page size write requester visible
pub const NAND_PAGE_SIZE_USABLE: usize = 2048;
/// NAND page size metadata
pub const NAND_PAGE_SIZE_METADATA: usize = 128;
/// NAND page size total (usable + metadata)
pub const NAND_TOTAL_PAGE_SIZE: usize = NAND_PAGE_SIZE_USABLE + NAND_PAGE_SIZE_METADATA;
/// NAND page read buffer count (TOTAL_NAND_PAGE_SIZE)
pub const NAND_PAGE_READ_BUFFER_N: usize = 8;
/// NAND page write buffer count (TOTAL_NAND_PAGE_SIZE)
pub const NAND_PAGE_WRITE_BUFFER_N: usize = 8;

/* Debug Setup */

/// Enable RAM Disk for debug
pub const DEBUG_ENABLE_RAM_DISK: bool = true;
/// USB device number of blocks (for debug)
pub const DEBUG_RAM_DISK_NUM_BLOCKS: usize = 16;
/// USB device total size (for debug)
pub const DEBUG_RAM_DISK_TOTAL_SIZE: usize = DEBUG_RAM_DISK_NUM_BLOCKS * USB_LOGICAL_BLOCK_SIZE;
