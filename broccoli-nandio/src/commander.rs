#![allow(unused, dead_code)]
#![cfg_attr(not(test), no_std)]

use crate::{address::MAX_IC, driver::Driver};
use bitvec::prelude as bv;
use core::future::Future;

#[derive(Default)]
pub struct Commander {
    /// Number of NAND chip
    pub valid_cs: bv::bitarr![0; MAX_IC],
    /// Number of valid NAND chip
    pub num_cs: usize,
}

impl Commander {
    /// Communication Setup
    pub async fn setup_comm(&mut self, driver: &mut impl Driver) -> usize {
        driver.init_pins();
        self.valid_cs.set_all(false);
        self.num_cs = 0;

        for i in 0..MAX_IC {
            driver.reset(i);
            let (success, _) = driver.read_id_async(i).await;
            self.valid_cs[i].set(i, success);
            self.num_cs += 1;
        }
        self.num_cs
    }
}
