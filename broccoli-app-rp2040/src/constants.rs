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
pub const USB_MAX_PACKET_SIZE: u8 = 64;
/// USB device vendor ID as a byte array
pub const USB_VENDOR_ID: [u8; 8] = *b"broccoli";
/// USB device product ID as a byte array
pub const USB_PRODUCT_ID: [u8; 16] = *b"wipeseals devapp";
/// USB device version as a byte array
pub const USB_PRODUCT_DEVICE_VERSION: [u8; 4] = *b"0001";
/// USB device number of blocks
pub const USB_NUM_BLOCKS: usize = 1024;
/// USB device block size
pub const USB_BLOCK_SIZE: usize = 512;
/// USB device total size
pub const USB_TOTAL_SIZE: usize = USB_NUM_BLOCKS * USB_BLOCK_SIZE;
/// LEDCTRL channel channel size
pub const CHANNEL_USB_TO_LEDCTRL_N: usize = 1;
/// USB Control Transfer to Bulk Transfer channel size
pub const CHANNEL_CTRL_TO_BULK_N: usize = 2;
/// USB Bulk Transfer to Internal Request channel size
pub const CHANNEL_BULK_TO_INTERNAL_N: usize = 4;
/// USB Internal Request to Bulk Transfer channel size
pub const CHANNEL_INTERNAL_TO_BULK_N: usize = 4;
/// USB block size
pub const LOGICAL_BLOCK_SIZE: usize = USB_BLOCK_SIZE as usize;
/// USB block buffer count
pub const LOGICAL_BLOCK_BUFFER_N: usize = 8;
/// NAND page size usable
/// TODO: broccoli-coreの値を参照して決定する。これ以外のパラメータ含む
pub const NAND_PAGE_SIZE_USABLE: usize = 2048;
/// NAND page size metadata
pub const NAND_PAGE_SIZE_METADATA: usize = 128;
/// NAND page size total (usable + metadata)
pub const NAND_PAGE_SIZE_TOTAL: usize = NAND_PAGE_SIZE_USABLE + NAND_PAGE_SIZE_METADATA;
/// NAND page buffer size
pub const NAND_PAGE_BUFFER_SIZE: usize = NAND_PAGE_SIZE_TOTAL as usize;
/// NAND page buffer count
pub const NAND_PAGE_BUFFER_N: usize = 4;
