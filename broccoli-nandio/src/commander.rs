#![allow(unused, dead_code)]
#![cfg_attr(not(test), no_std)]

use crate::{address::MAX_IC, driver::Driver};
use bitvec::prelude::*;
use core::future::Future;

type ValidCsBitArr = BitArr!(for MAX_IC, in u32, Lsb0);
#[derive(Default)]
pub struct Commander {
    /// Number of NAND chip
    pub valid_cs: ValidCsBitArr,
    /// Number of valid NAND chip
    pub num_cs: usize,
}

impl Commander {
    /// Communication Setup
    pub async fn setup(&mut self, driver: &mut impl Driver) -> usize {
        driver.init_pins();
        self.valid_cs.fill(false);
        self.num_cs = 0;

        for i in 0..MAX_IC {
            driver.reset(i);
            let (success, _) = driver.read_id_async(i).await;
            self.valid_cs.set(i, success);
            self.num_cs += 1;
        }
        self.num_cs
    }
}
