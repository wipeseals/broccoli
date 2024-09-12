#![allow(unused, dead_code)]
#![cfg_attr(not(test), no_std)]

use core::future::Future;

use defmt::{trace, warn};

use broccoli_core::{
    nand::address::Address,
    nand::driver::{
        CommandId, Driver, Error, StatusOutput, ID_READ_CMD_BYTES, ID_READ_EXPECT_DATA,
    },
};

use crate::nand::pins::NandIoPins;

/// Delay for command latch
/// t_XXX worst (w/o t_RST) = 100ns
pub const DELAY_US_FOR_COMMAND_LATCH: u32 = 1;

/// Delay for reset
/// t_RST = ~500us
pub const DELAY_US_FOR_RESET: u32 = 500;

/// Delay for wait busy (read)
/// t_R=25us,, t_DCBSYR1=25us, t_DCBSYR2=30us,
pub const DELAY_US_FOR_WAIT_BUSY_READ: u32 = 30;

/// Delay for wait busy (write)
/// t_PROG = 700us, t_DCBSYW2 = 700us
pub const DELAY_US_FOR_WAIT_BUSY_WRITE: u32 = 700;

/// Delay for wait busy (erase)
/// t_BERASE = 5ms (5,000us)
pub const DELAY_US_FOR_WAIT_BUSY_ERASE: u32 = 5000;

/// Check RBB (Ready/Busy) status
pub const RESOLUTION_COUNT_FOR_WAIT_BUSY: u32 = 10;

/// Retry count for wait busy
pub const RETRY_LIMIT_COUNT_FOR_WAIT_BUSY: u32 = 10 * RESOLUTION_COUNT_FOR_WAIT_BUSY;

/// NAND IC Command Driver
/// async not supported (implemented `async { self.func() }`))
pub struct Rp2040FwDriver<'a> {
    pub nandio_pins: &'a mut NandIoPins<'a>,
    pub delay: &'a mut cortex_m::delay::Delay,
}
impl Driver for Rp2040FwDriver<'_> {
    /// Initialize all pins
    fn init_pins<'a>(&'a mut self) {
        self.nandio_pins.init_all_pin();
        defmt::trace!("Initialize all pins")
    }

    fn set_write_protect<'a>(&'a mut self, enable: bool) {
        self.nandio_pins.set_write_protect_enable(enable);
        defmt::trace!("Set Write Protect: enable={}", enable);
    }

    /// Reset NAND IC
    fn reset<'a>(&'a mut self, cs_index: usize) {
        self.nandio_pins.assert_cs(cs_index);
        self.nandio_pins.input_command(CommandId::Reset as u8, || {
            self.delay.delay_us(DELAY_US_FOR_COMMAND_LATCH)
        });
        self.nandio_pins.deassert_cs();
        self.delay.delay_us(DELAY_US_FOR_RESET);
        defmt::trace!("Reset: cs={}", cs_index);
    }

    /// Read NAND IC ID
    fn read_id<'a>(&'a mut self, cs_index: usize) -> (bool, [u8; ID_READ_CMD_BYTES]) {
        let mut id_read_results = [0x00, 0x00, 0x00, 0x00, 0x00];

        self.nandio_pins.assert_cs(cs_index);
        self.nandio_pins.input_command(CommandId::IdRead as u8, || {
            self.delay.delay_us(DELAY_US_FOR_COMMAND_LATCH)
        });
        self.nandio_pins
            .input_address(&[0x00], || self.delay.delay_us(DELAY_US_FOR_COMMAND_LATCH));
        self.nandio_pins
            .output_data(&mut id_read_results, ID_READ_CMD_BYTES, || {
                self.delay.delay_us(DELAY_US_FOR_COMMAND_LATCH)
            });
        self.nandio_pins.deassert_cs();

        defmt::trace!(
            "ID Read: [{:02x}, {:02x}, {:02x}, {:02x}, {:02x}]",
            id_read_results[0],
            id_read_results[1],
            id_read_results[2],
            id_read_results[3],
            id_read_results[4]
        );
        (id_read_results == ID_READ_EXPECT_DATA, id_read_results)
    }

    /// Read NAND IC status
    fn read_status<'a>(&'a mut self, cs_index: usize) -> StatusOutput {
        let mut status = [0x00];

        self.nandio_pins.assert_cs(cs_index);
        self.nandio_pins
            .input_command(CommandId::StatusRead as u8, || {
                self.delay.delay_us(DELAY_US_FOR_COMMAND_LATCH)
            });
        self.nandio_pins.output_data(&mut status, 1, || {
            self.delay.delay_us(DELAY_US_FOR_COMMAND_LATCH)
        });
        self.nandio_pins.deassert_cs();

        defmt::trace!("Status Read: cs={}, status={:02x}", cs_index, status[0]);
        StatusOutput::from_bits_truncate(status[0])
    }

    /// Read NAND IC data
    fn read_data<'a>(
        &'a mut self,
        cs_index: usize,
        address: Address,
        read_data_ref: &mut [u8],
        read_bytes: usize,
    ) -> Result<(), Error> {
        self.nandio_pins.assert_cs(cs_index);
        self.nandio_pins
            .input_command(CommandId::ReadFirst as u8, || {
                self.delay.delay_us(DELAY_US_FOR_COMMAND_LATCH)
            });
        self.nandio_pins
            .input_address(&address.to_full_slice(), || {
                self.delay.delay_us(DELAY_US_FOR_COMMAND_LATCH)
            });
        self.nandio_pins
            .input_command(CommandId::ReadSecond as u8, || {
                self.delay.delay_us(DELAY_US_FOR_COMMAND_LATCH)
            });

        match self.nandio_pins.wait_for_busy(
            || self.delay.delay_us(DELAY_US_FOR_WAIT_BUSY_READ),
            RETRY_LIMIT_COUNT_FOR_WAIT_BUSY,
        ) {
            Ok(_) => {
                self.nandio_pins.output_data(read_data_ref, read_bytes, || {
                    self.delay.delay_us(DELAY_US_FOR_COMMAND_LATCH)
                });
                self.nandio_pins.deassert_cs();

                defmt::trace!("Read OK: cs={} address={:08x}", cs_index, address.raw());
                Ok(())
            }
            Err(_) => {
                defmt::warn!(
                    "Read Timeout: cs={} address={:08x}",
                    cs_index,
                    address.raw()
                );
                self.nandio_pins.deassert_cs();
                Err(Error::Timeout)
            }
        }
    }

    fn read_id_async<'a>(
        &'a mut self,
        cs_index: usize,
    ) -> impl Future<Output = (bool, [u8; ID_READ_CMD_BYTES])> {
        async move { self.read_id(cs_index) }
    }

    fn read_data_async<'a>(
        &'a mut self,
        cs_index: usize,
        address: Address,
        read_data_ref: &mut [u8],
        read_bytes: usize,
    ) -> impl Future<Output = Result<(), Error>> {
        async move { self.read_data(cs_index, address, read_data_ref, read_bytes) }
    }

    fn erase_block<'a>(
        &'a mut self,
        cs_index: usize,
        address: Address,
    ) -> Result<StatusOutput, Error> {
        self.nandio_pins.assert_cs(cs_index);
        self.nandio_pins
            .input_command(CommandId::AutoBlockEraseFirst as u8, || {
                self.delay.delay_us(DELAY_US_FOR_COMMAND_LATCH)
            });
        self.nandio_pins
            .input_address(&address.to_page_slice(), || {
                self.delay.delay_us(DELAY_US_FOR_COMMAND_LATCH)
            });
        self.nandio_pins
            .input_command(CommandId::AutoBlockEraseSecond as u8, || {
                self.delay.delay_us(DELAY_US_FOR_COMMAND_LATCH)
            });

        match self.nandio_pins.wait_for_busy(
            || self.delay.delay_us(DELAY_US_FOR_WAIT_BUSY_READ),
            RETRY_LIMIT_COUNT_FOR_WAIT_BUSY,
        ) {
            Ok(_) => {
                let mut status = [0x00];
                self.nandio_pins
                    .input_command(CommandId::StatusRead as u8, || {
                        self.delay.delay_us(DELAY_US_FOR_COMMAND_LATCH)
                    });
                self.nandio_pins.output_data(&mut status, 1, || {
                    self.delay.delay_us(DELAY_US_FOR_COMMAND_LATCH)
                });
                self.nandio_pins.deassert_cs();
                defmt::trace!(
                    "Erase: cs={} address={:08x} status={}",
                    cs_index,
                    address,
                    status[0]
                );

                Ok(StatusOutput::from_bits_truncate(status[0]))
            }
            Err(_) => {
                self.nandio_pins.deassert_cs();
                defmt::warn!(
                    "Erase Timeout: cs={} address={:08x}",
                    cs_index,
                    address.raw()
                );
                Err(Error::Timeout)
            }
        }
    }

    fn write_data<'a>(
        &'a mut self,
        cs_index: usize,
        address: Address,
        write_data_ref: &[u8],
        write_bytes: usize,
    ) -> Result<StatusOutput, Error> {
        self.nandio_pins.assert_cs(cs_index);
        self.nandio_pins
            .input_command(CommandId::AutoPageProgramFirst as u8, || {
                self.delay.delay_us(DELAY_US_FOR_COMMAND_LATCH)
            });
        self.nandio_pins
            .input_address(&address.to_full_slice(), || {
                self.delay.delay_us(DELAY_US_FOR_COMMAND_LATCH)
            });
        self.nandio_pins
            .input_data(&write_data_ref[..write_bytes], || {
                self.delay.delay_us(DELAY_US_FOR_COMMAND_LATCH)
            });
        self.nandio_pins
            .input_command(CommandId::AutoPageProgramSecond as u8, || {
                self.delay.delay_us(DELAY_US_FOR_COMMAND_LATCH)
            });

        match self.nandio_pins.wait_for_busy(
            || self.delay.delay_us(DELAY_US_FOR_WAIT_BUSY_READ),
            RETRY_LIMIT_COUNT_FOR_WAIT_BUSY,
        ) {
            Ok(_) => {
                let mut status = [0x00];
                self.nandio_pins
                    .input_command(CommandId::StatusRead as u8, || {
                        self.delay.delay_us(DELAY_US_FOR_COMMAND_LATCH)
                    });
                self.nandio_pins.output_data(&mut status, 1, || {
                    self.delay.delay_us(DELAY_US_FOR_COMMAND_LATCH)
                });
                self.nandio_pins.deassert_cs();
                defmt::trace!(
                    "Program: cs={} address={:08x} status={}",
                    cs_index,
                    address,
                    status[0]
                );

                Ok(StatusOutput::from_bits_truncate(status[0]))
            }
            Err(_) => {
                self.nandio_pins.deassert_cs();
                defmt::warn!(
                    "Program Timeout: cs={} address={:08x}",
                    cs_index,
                    address.raw()
                );
                Err(Error::Timeout)
            }
        }
    }

    async fn init_pins_async<'a>(&'a mut self) -> () {
        async { self.init_pins() }.await
    }

    async fn reset_async<'a>(&'a mut self, cs_index: usize) -> () {
        async move { self.reset(cs_index) }.await
    }

    async fn read_status_async<'a>(&'a mut self, cs_index: usize) -> StatusOutput {
        async move { self.read_status(cs_index) }.await
    }

    async fn set_write_protect_async<'a>(&'a mut self, enable: bool) -> () {
        async move {
            self.set_write_protect(enable);
        }
        .await
    }

    async fn erase_block_async<'a>(
        &'a mut self,
        cs_index: usize,
        address: Address,
    ) -> Result<StatusOutput, Error> {
        async move { self.erase_block(cs_index, address) }.await
    }

    async fn write_data_async(
        &mut self,
        cs_index: usize,
        address: Address,
        write_data_ref: &[u8],
        write_bytes: usize,
    ) -> Result<StatusOutput, Error> {
        async move { self.write_data(cs_index, address, write_data_ref, write_bytes) }.await
    }
}
