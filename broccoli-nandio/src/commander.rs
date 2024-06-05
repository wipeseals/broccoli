#![allow(unused, dead_code)]
#![cfg_attr(not(test), no_std)]

use bit_field::BitArray;

use crate::{
    address::{
        Address, AllIcNandBlockBitArr, IcBitmapArr, NandBlockBitArr, BLOCK_BITMAP_U32_SIZE,
        IC_BITMAP_U32_SIZE, MAX_BLOCKS_PER_IC, MAX_IC,
    },
    driver::Driver,
};
use broccoli_util::bitarr::BitArr;
use core::future::Future;

pub struct Commander {
    /// Number of NAND chip
    pub valid_cs_bitarr: IcBitmapArr,
    /// Number of valid NAND chip
    pub num_cs: usize,
    /// Bad block bit array
    pub bad_block_bitarr: AllIcNandBlockBitArr,
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
        self.bad_block_bitarr = all_bitmap.clone();
    }
}
