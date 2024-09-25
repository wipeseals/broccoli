use crate::nand::nand_address::NandAddress;
use crate::shared::constant::{NAND_PAGE_TRANSFER_BYTES, NAND_TOTAL_ADDR_TRANSFER_BYTES};
use crate::{
    nand::nand_pins::NandIoPins,
    shared::constant::{
        DELAY_US_FOR_COMMAND_LATCH, DELAY_US_FOR_RESET, DELAY_US_FOR_WAIT_BUSY_READ,
        ID_READ_CMD_BYTES, ID_READ_EXPECT_DATA, NAND_MAX_CHIP_NUM, TIMEOUT_LIMIT_US_FOR_WAIT_BUSY,
    },
};
use bit_field::BitField;
use bitflags::bitflags;
use broccoli_core::commander::NandCommander;
use broccoli_core::common::io_address::IoAddress;
use broccoli_core::common::io_driver::{
    NandCommandId, NandIoDriver, NandIoError, NandStatusReadResult,
};
use core::future::Future;
use defmt::{trace, warn};
use embassy_time::Timer;

/// NAND IC Status Output
///
/// | Bit | Description            | Value                      |
/// | --- | ---------------------- | -------------------------- |
/// | 0   | Chip Status0           | Pass:0 , Fail: 1           |
/// | 1   | Chip Status1           | Pass:0 , Fail: 1           |
/// | 2   | -                      | -                          |
/// | 3   | -                      | -                          |
/// | 4   | -                      | -                          |
/// | 5   | Page Buffer Ready/Busy | Ready: 1, Busy: 0          |
/// | 6   | Data Cache Ready/Busy  | Ready: 1, Busy: 0          |
/// | 7   | Write Protect          | Not Protect: 1, Protect: 0 |

bitflags! {
    #[derive(Default, Clone, Copy, PartialEq)]
    pub struct NandStatusReadBitFlags: u8 {
        const CHIP_STATUS0_FAIL = 0b0000_0001;
        const CHIP_STATUS1_FAIL = 0b0000_0010;
        const PAGE_BUFFER_READY = 0b0010_0000;
        const DATA_CACHE_READY = 0b0100_0000;
        const WRITE_PROTECT_DISABLE = 0b1000_0000;
    }
}

impl NandStatusReadBitFlags {
    /// Check if page buffer is ready
    fn is_page_buffer_ready(&self) -> bool {
        !(*self & NandStatusReadBitFlags::PAGE_BUFFER_READY).is_empty()
    }

    /// Check if data cache is ready
    pub fn is_data_cache_ready(&self) -> bool {
        !(*self & NandStatusReadBitFlags::DATA_CACHE_READY).is_empty()
    }
}

impl NandStatusReadResult for NandStatusReadBitFlags {
    fn is_failed(&self) -> bool {
        (!(*self & NandStatusReadBitFlags::CHIP_STATUS0_FAIL).is_empty())
            || (!(*self & NandStatusReadBitFlags::CHIP_STATUS1_FAIL).is_empty())
    }

    fn is_write_protect(&self) -> bool {
        !(*self & NandStatusReadBitFlags::WRITE_PROTECT_DISABLE).is_empty()
    }
}

/// NAND IC Command Driver for TC58NVG0S3HTA00 (JISC-SSD)
pub struct NandIoFwDriver<'d> {
    pins: NandIoPins<'d>,
}

impl<'d> NandIoFwDriver<'d> {
    /// Create a new NandIoFwDriver
    pub fn new(pins: NandIoPins<'d>) -> Self {
        Self { pins }
    }
}

impl<'d> NandIoDriver<NandAddress, NandStatusReadBitFlags> for NandIoFwDriver<'d> {
    async fn setup(&mut self) {
        self.pins.setup().await;
    }

    async fn set_write_protect(&mut self, enable: bool) {
        self.pins.set_write_protect(enable).await;
    }

    async fn reset(&mut self, address: NandAddress) {
        let cs_index = address.chip();
        self.pins.assert_cs(cs_index).await;
        self.pins
            .input_command(NandCommandId::Reset as u8, DELAY_US_FOR_COMMAND_LATCH)
            .await;
        self.pins.deassert_cs().await;
        Timer::after_micros(DELAY_US_FOR_RESET).await;
        defmt::trace!("Reset: cs={}", cs_index);
    }

    /// Read NAND IC ID
    async fn read_id(&mut self, address: NandAddress) -> bool {
        let cs_index = address.chip();
        let mut id_read_results = [0x00u8; 5];

        self.pins.assert_cs(cs_index).await;
        self.pins
            .input_command(NandCommandId::IdRead as u8, DELAY_US_FOR_COMMAND_LATCH)
            .await;
        self.pins
            .input_address(&[0x00], DELAY_US_FOR_COMMAND_LATCH)
            .await;
        self.pins
            .read_data(
                &mut id_read_results,
                ID_READ_CMD_BYTES,
                DELAY_US_FOR_COMMAND_LATCH,
            )
            .await;
        self.pins.deassert_cs().await;

        defmt::trace!(
            "ID Read: [{:02x}, {:02x}, {:02x}, {:02x}, {:02x}]",
            id_read_results[0],
            id_read_results[1],
            id_read_results[2],
            id_read_results[3],
            id_read_results[4]
        );

        // ID Read results should be equal to expected data
        id_read_results == ID_READ_EXPECT_DATA
    }

    /// Read NAND IC status
    async fn read_status(&mut self, address: NandAddress) -> NandStatusReadBitFlags {
        let cs_index = address.chip();
        let mut status = [0x00];

        self.pins.assert_cs(cs_index).await;
        self.pins
            .input_command(NandCommandId::StatusRead as u8, DELAY_US_FOR_COMMAND_LATCH)
            .await;
        self.pins
            .read_data(&mut status, 1, DELAY_US_FOR_COMMAND_LATCH)
            .await;
        self.pins.deassert_cs().await;

        defmt::trace!("Status Read: cs={}, status={:02x}", cs_index, status[0]);
        NandStatusReadBitFlags::from_bits_truncate(status[0])
    }

    /// Read NAND IC data
    async fn read_data<'data>(
        &mut self,
        address: NandAddress,
        read_data_ref: &'data mut [u8],
        read_bytes: usize,
    ) -> Result<(), NandIoError> {
        let cs_index = address.chip();
        let mut address_data = [0x00u8; NAND_TOTAL_ADDR_TRANSFER_BYTES];
        address.to_slice(&mut address_data);

        self.pins.assert_cs(cs_index).await;
        self.pins
            .input_command(NandCommandId::ReadFirst as u8, DELAY_US_FOR_COMMAND_LATCH)
            .await;
        self.pins
            .input_address(&address_data, DELAY_US_FOR_COMMAND_LATCH)
            .await;
        self.pins
            .input_command(NandCommandId::ReadSecond as u8, DELAY_US_FOR_COMMAND_LATCH)
            .await;
        match self
            .pins
            .wait_for_busy(DELAY_US_FOR_WAIT_BUSY_READ, TIMEOUT_LIMIT_US_FOR_WAIT_BUSY)
            .await
        {
            Ok(_) => {
                self.pins
                    .read_data(read_data_ref, read_bytes, DELAY_US_FOR_COMMAND_LATCH)
                    .await;
                self.pins.deassert_cs().await;

                defmt::trace!("Read OK: cs={} address={:08x}", cs_index, address.raw());
                Ok(())
            }
            Err(_) => {
                defmt::warn!(
                    "Read Timeout: cs={} address={:08x}",
                    cs_index,
                    address.raw()
                );
                self.pins.deassert_cs().await;
                Err(NandIoError::Timeout)
            }
        }
    }

    async fn erase_block(
        &mut self,
        address: NandAddress,
    ) -> Result<NandStatusReadBitFlags, NandIoError> {
        let cs_index = address.chip();
        let mut block_address_data = [0x00u8; NAND_PAGE_TRANSFER_BYTES];
        address.to_block_slice(&mut block_address_data);

        self.pins.assert_cs(cs_index).await;
        self.pins
            .input_command(
                NandCommandId::AutoBlockEraseFirst as u8,
                DELAY_US_FOR_COMMAND_LATCH,
            )
            .await;
        self.pins
            .input_address(&block_address_data, DELAY_US_FOR_COMMAND_LATCH)
            .await;
        self.pins
            .input_command(
                NandCommandId::AutoBlockEraseSecond as u8,
                DELAY_US_FOR_COMMAND_LATCH,
            )
            .await;

        match self
            .pins
            .wait_for_busy(DELAY_US_FOR_WAIT_BUSY_READ, TIMEOUT_LIMIT_US_FOR_WAIT_BUSY)
            .await
        {
            Ok(_) => {
                let mut status = [0x00];
                self.pins
                    .input_command(NandCommandId::StatusRead as u8, DELAY_US_FOR_COMMAND_LATCH)
                    .await;
                self.pins
                    .read_data(&mut status, 1, DELAY_US_FOR_COMMAND_LATCH)
                    .await;
                self.pins.deassert_cs().await;

                defmt::trace!(
                    "Erase: cs={} address={:08x} status={}",
                    cs_index,
                    address,
                    status[0]
                );

                Ok(NandStatusReadBitFlags::from_bits_truncate(status[0]))
            }
            Err(_) => {
                self.pins.deassert_cs().await;
                defmt::warn!(
                    "Erase Timeout: cs={} address={:08x}",
                    cs_index,
                    address.raw()
                );
                Err(NandIoError::Timeout)
            }
        }
    }

    async fn write_data(
        &mut self,
        address: NandAddress,
        write_data_ref: &[u8],
        write_bytes: usize,
    ) -> Result<NandStatusReadBitFlags, NandIoError> {
        let cs_index = address.chip();
        let mut address_data = [0x00u8; NAND_TOTAL_ADDR_TRANSFER_BYTES];
        address.to_slice(&mut address_data);

        self.pins.assert_cs(cs_index).await;
        self.pins
            .input_command(
                NandCommandId::AutoPageProgramFirst as u8,
                DELAY_US_FOR_COMMAND_LATCH,
            )
            .await;
        self.pins
            .input_address(&address_data, DELAY_US_FOR_COMMAND_LATCH)
            .await;
        self.pins
            .write_data(&write_data_ref[..write_bytes], DELAY_US_FOR_COMMAND_LATCH)
            .await;
        self.pins
            .input_command(
                NandCommandId::AutoPageProgramSecond as u8,
                DELAY_US_FOR_COMMAND_LATCH,
            )
            .await;

        match self
            .pins
            .wait_for_busy(DELAY_US_FOR_WAIT_BUSY_READ, TIMEOUT_LIMIT_US_FOR_WAIT_BUSY)
            .await
        {
            Ok(_) => {
                let mut status = [0x00];
                self.pins
                    .input_command(NandCommandId::StatusRead as u8, DELAY_US_FOR_COMMAND_LATCH)
                    .await;
                self.pins
                    .read_data(&mut status, 1, DELAY_US_FOR_COMMAND_LATCH)
                    .await;
                self.pins.deassert_cs().await;

                defmt::trace!(
                    "Program: cs={} address={:08x} status={}",
                    cs_index,
                    address,
                    status[0]
                );

                Ok(NandStatusReadBitFlags::from_bits_truncate(status[0]))
            }
            Err(_) => {
                self.pins.deassert_cs().await;
                defmt::warn!(
                    "Program Timeout: cs={} address={:08x}",
                    cs_index,
                    address.raw()
                );
                Err(NandIoError::Timeout)
            }
        }
    }
}
