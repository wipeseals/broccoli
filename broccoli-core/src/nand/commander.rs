#![cfg_attr(not(test), no_std)]

use crate::nand::address::*;
use crate::nand::io_driver::*;
use core::future::Future;

#[cfg(test)]
use async_mock::async_mock;
use async_trait::async_trait;

pub struct NandCommander<'d, Driver: NandIoDriver, const MAX_IC_NUM: usize> {
    /// IO Driver
    driver: &'d mut Driver,

    /// Number of valid NAND chip
    /// CS1だけに有効なNANDチップがある場合は想定していない
    num_cs: usize,
}

impl<'d, Driver: NandIoDriver, const MAX_IC_NUM: usize> NandCommander<'d, Driver, MAX_IC_NUM> {
    pub fn new(driver: &'d mut Driver) -> Self {
        Self { driver, num_cs: 0 }
    }

    /// Communication Setup
    /// Return number of valid NAND chip
    /// If no NAND chip is found, return error
    pub async fn setup(&mut self) -> Result<usize, NandIoError> {
        self.driver.setup().await;
        self.num_cs = 0;

        for i in 0..MAX_IC_NUM {
            self.driver.reset(i).await;
            if !self.driver.read_id(i).await {
                break;
            }
            self.num_cs += 1;
        }
        // 一つも確認できない場合はエラー
        if (self.num_cs == 0) {
            Err(NandIoError::IdReadFailed)
        } else {
            Ok(self.num_cs)
        }
    }
}
