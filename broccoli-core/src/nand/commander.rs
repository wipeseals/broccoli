#![cfg_attr(not(test), no_std)]

use crate::nand::address::*;
use crate::nand::driver::*;
use core::future::Future;

#[cfg(test)]
use async_mock::async_mock;
use async_trait::async_trait;

pub struct Commander {
    /// Number of NAND chip
    pub valid_cs_bitarr: IcBitmapArr,
    /// Number of valid NAND chip
    pub num_cs: usize,
    /// Bad block bit array
    pub bad_block_bitarr: AllIcNandBlockBitArr,
}

impl Default for Commander {
    fn default() -> Self {
        Self::new()
    }
}

impl Commander {
    pub fn new() -> Self {
        Self {
            valid_cs_bitarr: IcBitmapArr::new(),
            num_cs: 0,
            bad_block_bitarr: [NandBlockBitArr::new(); MAX_IC],
        }
    }
    /// Communication Setup
    pub async fn setup(&mut self, driver: &mut impl Driver) -> usize {
        driver.init_pins();
        self.valid_cs_bitarr.fill(false);
        self.num_cs = 0;

        for i in 0..MAX_IC {
            driver.reset(i);
            let (success, _) = driver.read_id_async(i).await;
            self.valid_cs_bitarr.set(i, success);
            self.num_cs += 1;
        }
        self.num_cs
    }

    /// Check Bad Blocks
    pub async fn create_badblock_bitarr(
        &mut self,
        driver: &mut impl Driver,
    ) -> &AllIcNandBlockBitArr {
        assert!(self.num_cs > 0, "No valid NAND chip found");

        for ic_idx in 0..MAX_IC {
            if self.valid_cs_bitarr.get(ic_idx) {
                for block_idx in 0..MAX_BLOCKS_PER_IC {
                    let addr = Address::from_block(block_idx as u16);
                    let mut read_data = [0u8; 1];
                    driver
                        .read_data_async(ic_idx, addr, &mut read_data, 1)
                        .await;
                    let success = read_data[0] == 0xFF;
                    self.bad_block_bitarr[ic_idx].set(block_idx, !success);
                }
            }
        }

        &self.bad_block_bitarr
    }

    /// Add Bad Blocks
    pub fn add_badblock(&mut self, ic_idx: usize, block_idx: usize) {
        self.bad_block_bitarr[ic_idx].set(block_idx, true);
    }

    /// Restore Bad Blocks
    pub fn restore_badblock_bitarr(&mut self, all_bitmap: &AllIcNandBlockBitArr) {
        self.bad_block_bitarr = *all_bitmap;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nand::driver::{MockDriver, ID_READ_EXPECT_DATA};

    #[tokio::test]
    async fn test_setup() {
        let mut commander = Commander {
            valid_cs_bitarr: IcBitmapArr::new(),
            num_cs: 2,
            bad_block_bitarr: [NandBlockBitArr::new(); MAX_IC],
        };
        let mut driver = MockDriver::new();
        driver.expect_init_pins().times(1).returning(|| {});
        driver.expect_reset().times(2).returning(|_| {});
        driver
            .expect_read_id_async()
            .times(2)
            .returning(|_| (true, ID_READ_EXPECT_DATA));

        let num_cs = commander.setup(&mut driver).await;
        assert_eq!(num_cs, 2);
    }
}
