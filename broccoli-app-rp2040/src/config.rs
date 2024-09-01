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
pub const USB_DEVICE_VERSION: [u8; 4] = *b"0001";
/// USB device number of blocks
pub const USB_NUM_BLOCKS: u32 = 1024;
/// USB device block size
pub const USB_BLOCK_SIZE: u32 = 512;
/// USB device total size
pub const USB_TOTAL_SIZE: u32 = USB_NUM_BLOCKS * USB_BLOCK_SIZE;
