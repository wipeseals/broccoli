#![feature(generic_const_exprs)]

use core::{
    borrow::{Borrow, BorrowMut},
    cell::RefCell,
};

use embassy_sync::{
    blocking_mutex::{raw::CriticalSectionRawMutex, CriticalSectionMutex},
    mutex::{Mutex, MutexGuard},
};

/// General Data Buffer
#[derive(Copy, Clone, Eq, PartialEq, defmt::Format)]
pub struct BufferIdentify {
    pub buf_index: u32,
}

/// Buffer status
#[derive(Copy, Clone, Eq, PartialEq, defmt::Format)]
pub enum BufferStatus {
    /// not used
    Free,
    /// in use
    InUse { user_tag: u32 },
}

/// Shared fixed-size buffer manager
#[derive(defmt::Format)]
pub struct SharedBufferManager<const BUFFER_SIZE: usize, const BUFFER_N: usize> {
    /// Shared buffer
    pub buffers: [Mutex<CriticalSectionRawMutex, [u8; BUFFER_SIZE]>; BUFFER_N],
    /// Buffer status
    pub statuses: [Mutex<CriticalSectionRawMutex, [BufferStatus; BUFFER_SIZE]>; BUFFER_N],
}

impl BufferIdentify {
    pub fn new(buf_index: u32) -> Self {
        Self { buf_index }
    }
}

impl BufferStatus {
    pub fn free() -> Self {
        Self::Free
    }
    pub fn in_use(user_tag: u32) -> Self {
        Self::InUse { user_tag }
    }
}

impl<const BUFFER_SIZE: usize, const BUFFER_N: usize> SharedBufferManager<BUFFER_SIZE, BUFFER_N> {
    pub fn new() -> Self {
        Self {
            buffers: [const { Mutex::new([0; BUFFER_SIZE]) }; BUFFER_N],
            statuses: [const { Mutex::new([BufferStatus::Free; BUFFER_SIZE]) }; BUFFER_N],
        }
    }

    /// Allocate buffer
    pub async fn allocate(&mut self, user_tag: u32) -> Option<BufferIdentify> {
        for i in 0..BUFFER_N {
            // lock status
            let mut status = self.statuses[i].lock().await;
            // check if free
            if status.borrow()[0] != BufferStatus::Free {
                continue;
            }
            // set in use
            status.borrow_mut()[0] = BufferStatus::InUse { user_tag };
            return Some(BufferIdentify::new(i as u32));
        }
        crate::warn!("allocate failed");
        None
    }

    /// Free buffer
    pub async fn free(&mut self, id: BufferIdentify) {
        crate::assert!(id.buf_index < BUFFER_N as u32);

        // lock status
        let mut status = self.statuses[id.buf_index as usize].lock().await;
        // check if in use
        if !matches!(status.borrow()[0], BufferStatus::InUse { .. }) {
            crate::warn!("free failed");
        }
        // set free
        status.borrow_mut()[0] = BufferStatus::Free;
    }

    /// Get buffer body (mutable)
    pub async fn get_buf(
        &mut self,
        id: BufferIdentify,
    ) -> MutexGuard<'_, CriticalSectionRawMutex, [u8; BUFFER_SIZE]> {
        crate::assert!(id.buf_index < BUFFER_N as u32);
        // SAFETY: id is valid
        let mut status = self.statuses[id.buf_index as usize].lock().await;
        if !matches!(status.borrow()[0], BufferStatus::InUse { .. }) {
            crate::unreachable!("free failed");
        }

        self.buffers[id.buf_index as usize].lock().await
    }
}
