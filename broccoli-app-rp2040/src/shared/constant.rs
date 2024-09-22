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

/// NAND IC count (min)
pub const NAND_MIX_IC_NUM: usize = 1;
/// NAND IC count
pub const NAND_MAX_IC_NUM: usize = 2;
/// NAND page size write requester visible
pub const NAND_PAGE_SIZE_USABLE: usize = 2048;
/// NAND page size metadata
pub const NAND_PAGE_SIZE_SPARE: usize = 128;
/// Total NAND Page Size (Data + Spare = 2176 bytes)
pub const NAND_PAGE_TOTAL_SIZE: usize = NAND_PAGE_SIZE_USABLE + NAND_PAGE_SIZE_SPARE;

/// Page/Block
pub const PAGES_PER_PHYSICAL_BLOCK: usize = 64;
/// Total Blocks per IC
pub const MAX_PHYSICAL_BLOCKS_PER_IC: usize = 1024;
/// Minimum Blocks per IC
pub const MIN_PHYSICAL_BLOCKS_PER_IC: usize = 1004;

/// Total Bytes per Block (2176 * 64 = 139264 bytes)
pub const NAND_BYTES_PER_PHYSICAL_BLOCK: usize = NAND_PAGE_TOTAL_SIZE * PAGES_PER_PHYSICAL_BLOCK;
/// Maximum Pages per IC (64 * 1024 = 65536 pages)
pub const NAND_MAX_PAGES_PER_IC: usize = MAX_PHYSICAL_BLOCKS_PER_IC * PAGES_PER_PHYSICAL_BLOCK;
/// Maximum Bytes per IC (139264 * 1024 = 142606336 bytes = 142.6MB)
pub const MAX_BYTES_PER_IC: usize = MAX_PHYSICAL_BLOCKS_PER_IC * NAND_BYTES_PER_PHYSICAL_BLOCK;
/// Minimum Pages per IC (64 * 1004 = 64256 pages)
pub const MIN_PAGS_PER_IC: usize = MIN_PHYSICAL_BLOCKS_PER_IC * PAGES_PER_PHYSICAL_BLOCK;
/// Minimum Bytes per IC (139264 * 1004 = 140000256 bytes = 140MB)
pub const MIN_BYTES_PER_IC: usize = MIN_PHYSICAL_BLOCKS_PER_IC * NAND_BYTES_PER_PHYSICAL_BLOCK;

/* NAND AC/Function Characteristic */

/// ID read bytes (for TC58NVG0S3HTA00)
pub const ID_READ_CMD_BYTES: usize = 5;
/// ID read expect data (for TC58NVG0S3HTA00)
///
/// | Description            | Hex Data |
/// | ---------------------- | -------- |
/// | Maker Code             | 0x98     |
/// | Device Code            | 0xF1     |
/// | Chip Number, Cell Type | 0x80     |
/// | Page Size, Block Size  | 0x15     |
/// | District Number        | 0x72     |
pub const ID_READ_EXPECT_DATA: [u8; ID_READ_CMD_BYTES] = [0x98, 0xF1, 0x80, 0x15, 0x72];

/// Column Addressing Transfer cycles
pub const NAND_COLUMN_TRANSFER_BYTES: usize = 2;
/// Page(PA0~PA15) Address Transfer cycles
pub const NAND_PAGE_TRANSFER_BYTES: usize = 2;
/// Total Address Transfer cycles
pub const NAND_TOTAL_ADDR_TRANSFER_BYTES: usize =
    NAND_COLUMN_TRANSFER_BYTES + NAND_PAGE_TRANSFER_BYTES;
/// Delay for command latch
/// t_XXX worst (w/o t_RST) = 100ns
pub const DELAY_US_FOR_COMMAND_LATCH: u64 = 1;
/// Delay for reset
/// t_RST = ~500us
pub const DELAY_US_FOR_RESET: u64 = 500;
/// Delay for wait busy (read)
/// t_R=25us,, t_DCBSYR1=25us, t_DCBSYR2=30us,
pub const DELAY_US_FOR_WAIT_BUSY_READ: u64 = 30;
/// Delay for wait busy (write)
/// t_PROG = 700us, t_DCBSYW2 = 700us
pub const DELAY_US_FOR_WAIT_BUSY_WRITE: u64 = 700;
/// Delay for wait busy (erase)
/// t_BERASE = 5ms (5,000us)
pub const DELAY_US_FOR_WAIT_BUSY_ERASE: u64 = 5000;
/// Timeout limit for wait busy
pub const TIMEOUT_LIMIT_US_FOR_WAIT_BUSY: u64 = 1_000_000;

/* Debug Setup */

/// Enable RAM Disk for debug
pub const DEBUG_ENABLE_RAM_DISK: bool = false;
/// USB device number of blocks (for debug)
pub const DEBUG_RAM_DISK_NUM_BLOCKS: usize = 16;
/// USB device total size (for debug)
pub const DEBUG_RAM_DISK_TOTAL_SIZE: usize = DEBUG_RAM_DISK_NUM_BLOCKS * USB_LOGICAL_BLOCK_SIZE;
