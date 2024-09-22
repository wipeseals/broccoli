#![cfg_attr(not(test), no_std)]

use crate::common::{io_address::IoAddress, io_driver::*};
use core::{future::Future, marker::PhantomData};

#[cfg(test)]
use async_mock::async_mock;
use async_trait::async_trait;

pub struct NandCommander<
    'd,
    Addr: IoAddress,
    Status: NandStatusReadResult,
    Driver: NandIoDriver<Addr, Status>,
    const MAX_IC_NUM: usize,
> {
    /// IO Driver
    driver: &'d mut Driver,

    /// Number of valid NAND chip
    /// CS1だけに有効なNANDチップがある場合は想定していない
    num_cs: usize,

    /// PhantomData to hold the Addr type parameter
    _phantom0: PhantomData<Addr>,

    /// PhantomData to hold the Status type parameter
    _phantom1: PhantomData<Status>,
}

impl<
        'd,
        Addr: IoAddress,
        Status: NandStatusReadResult,
        Driver: NandIoDriver<Addr, Status>,
        const MAX_IC_NUM: usize,
    > NandCommander<'d, Addr, Status, Driver, MAX_IC_NUM>
{
    pub fn new(driver: &'d mut Driver) -> Self {
        Self {
            driver,
            num_cs: 0,
            _phantom0: PhantomData,
            _phantom1: PhantomData,
        }
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
