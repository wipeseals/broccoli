use core::future::Future;

use defmt::{trace, warn};

use broccoli_core::nand::{
    address::NandAddress,
    commander::NandCommander,
    io_driver::{NandCommandId, NandIoDriver, NandIoError, NandStatusOutput},
};
use embassy_time::Timer;

use crate::{
    nand::nand_pins::NandIoPins,
    shared::constant::{
        DELAY_US_FOR_COMMAND_LATCH, DELAY_US_FOR_RESET, DELAY_US_FOR_WAIT_BUSY_READ,
        ID_READ_CMD_BYTES, ID_READ_EXPECT_DATA, NAND_MAX_IC_NUM, TIMEOUT_LIMIT_US_FOR_WAIT_BUSY,
    },
};

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

impl<'d> NandIoDriver for NandIoFwDriver<'d> {
    async fn setup(&mut self) {
        self.pins.setup().await;
    }

    async fn set_write_protect(&mut self, enable: bool) {
        self.pins.set_write_protect(enable).await;
    }

    async fn reset(&mut self, cs_index: usize) {
        self.pins.assert_cs(cs_index).await;
        self.pins
            .input_command(NandCommandId::Reset as u8, DELAY_US_FOR_COMMAND_LATCH)
            .await;
        self.pins.deassert_cs().await;
        Timer::after_micros(DELAY_US_FOR_RESET).await;
        defmt::trace!("Reset: cs={}", cs_index);
    }

    /// Read NAND IC ID
    async fn read_id(&mut self, cs_index: usize) -> bool {
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
    async fn read_status(&mut self, cs_index: usize) -> NandStatusOutput {
        let mut status = [0x00];

        self.pins.assert_cs(cs_index);
        self.pins
            .input_command(NandCommandId::StatusRead as u8, DELAY_US_FOR_COMMAND_LATCH)
            .await;
        self.pins
            .read_data(&mut status, 1, DELAY_US_FOR_COMMAND_LATCH)
            .await;
        self.pins.deassert_cs().await;

        defmt::trace!("Status Read: cs={}, status={:02x}", cs_index, status[0]);
        NandStatusOutput::from_bits_truncate(status[0])
    }

    /// Read NAND IC data
    async fn read_data<'data>(
        &mut self,
        cs_index: usize,
        address: NandAddress,
        read_data_ref: &'data mut [u8],
        read_bytes: usize,
    ) -> Result<(), NandIoError> {
        self.pins.assert_cs(cs_index).await;
        self.pins
            .input_command(NandCommandId::ReadFirst as u8, DELAY_US_FOR_COMMAND_LATCH)
            .await;
        self.pins
            .input_address(&address.to_full_slice(), DELAY_US_FOR_COMMAND_LATCH)
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
        cs_index: usize,
        address: NandAddress,
    ) -> Result<NandStatusOutput, NandIoError> {
        self.pins.assert_cs(cs_index).await;
        self.pins
            .input_command(
                NandCommandId::AutoBlockEraseFirst as u8,
                DELAY_US_FOR_COMMAND_LATCH,
            )
            .await;
        self.pins
            .input_address(&address.to_page_slice(), DELAY_US_FOR_COMMAND_LATCH)
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

                Ok(NandStatusOutput::from_bits_truncate(status[0]))
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
        cs_index: usize,
        address: NandAddress,
        write_data_ref: &[u8],
        write_bytes: usize,
    ) -> Result<NandStatusOutput, NandIoError> {
        self.pins.assert_cs(cs_index).await;
        self.pins
            .input_command(
                NandCommandId::AutoPageProgramFirst as u8,
                DELAY_US_FOR_COMMAND_LATCH,
            )
            .await;
        self.pins
            .input_address(&address.to_full_slice(), DELAY_US_FOR_COMMAND_LATCH)
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

                Ok(NandStatusOutput::from_bits_truncate(status[0]))
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
