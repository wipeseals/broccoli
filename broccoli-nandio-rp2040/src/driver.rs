#![allow(unused, dead_code)]
#![cfg_attr(not(test), no_std)]

use core::future::Future;

use defmt::{trace, warn};

extern crate broccoli_nandio;
use broccoli_nandio::{
    address::Address,
    driver::{CommandId, Driver, Error, StatusOutput, ID_READ_CMD_BYTES, ID_READ_EXPECT_DATA},
};

use crate::pins::NandIoPins;

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
pub struct Rp2040FwDriver<'a> {
    pub nandio_pins: &'a mut NandIoPins<'a>,
    pub delay: &'a mut cortex_m::delay::Delay,
}
impl Driver for Rp2040FwDriver<'_> {
    /// Initialize all pins
    fn init_pins(&mut self) {
        self.nandio_pins.init_all_pin();
        trace!("Initialize all pins")
    }

    /// Reset NAND IC
    fn reset(&mut self, cs_index: u32) {
        self.nandio_pins.assert_cs(cs_index);
        self.nandio_pins.input_command(CommandId::Reset as u8, || {
            self.delay.delay_us(DELAY_US_FOR_COMMAND_LATCH)
        });
        self.nandio_pins.deassert_cs();
        self.delay.delay_us(DELAY_US_FOR_RESET);
        trace!("Reset NAND IC")
    }

    /// Read NAND IC ID
    fn read_id(&mut self, cs_index: u32) -> (bool, [u8; ID_READ_CMD_BYTES]) {
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

        trace!(
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
    fn read_status(&mut self, cs_index: u32) -> StatusOutput {
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

        trace!("Status Read: {:02x}", status[0]);
        StatusOutput::from_bits_truncate(status[0])
    }

    /// Read NAND IC data
    fn read_data(
        &mut self,
        cs_index: u32,
        address: Address,
        read_data_ref: &mut [u8],
        read_bytes: u32,
    ) -> Result<(), Error> {
        self.nandio_pins.assert_cs(cs_index);
        self.nandio_pins
            .input_command(CommandId::ReadFirst as u8, || {
                self.delay.delay_us(DELAY_US_FOR_COMMAND_LATCH)
            });
        self.nandio_pins.input_address(&address.pack_slice(), || {
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
                self.nandio_pins
                    .output_data(read_data_ref, read_bytes as usize, || {
                        self.delay.delay_us(DELAY_US_FOR_COMMAND_LATCH)
                    });
                self.nandio_pins.deassert_cs();

                trace!("Read OK");
                Ok(())
            }
            Err(_) => {
                warn!("Timeout for read data");
                self.nandio_pins.deassert_cs();
                Err(Error::Timeout)
            }
        }
    }

    fn init_pins_async(&mut self) -> impl Future<Output = ()> {
        async { self.init_pins() }
    }

    fn reset_async(&mut self, cs_index: u32) -> impl Future<Output = ()> {
        async move { self.reset(cs_index) }
    }

    fn read_id_async(
        &mut self,
        cs_index: u32,
    ) -> impl Future<Output = (bool, [u8; ID_READ_CMD_BYTES])> {
        async move { self.read_id(cs_index) }
    }

    fn read_status_async(&mut self, cs_index: u32) -> impl Future<Output = StatusOutput> {
        async move { self.read_status(cs_index) }
    }

    fn read_data_async(
        &mut self,
        cs_index: u32,
        address: Address,
        read_data_ref: &mut [u8],
        read_bytes: u32,
    ) -> impl Future<Output = Result<(), Error>> {
        async move { self.read_data(cs_index, address, read_data_ref, read_bytes) }
    }
}
