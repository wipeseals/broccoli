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
    pub id: u32,
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
    pub buf_status: [Mutex<CriticalSectionRawMutex, [BufferStatus; BUFFER_SIZE]>; BUFFER_N],
}

impl BufferIdentify {
    pub fn new(id: u32) -> Self {
        Self { id }
    }
    pub fn id(&self) -> u32 {
        self.id
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
            buf_status: [const { Mutex::new([BufferStatus::Free; BUFFER_SIZE]) }; BUFFER_N],
        }
    }

    /// Get buffer size
    pub fn buf_size(&self) -> usize {
        BUFFER_SIZE
    }

    /// Allocate buffer
    pub async fn allocate(&mut self, user_tag: u32) -> Option<BufferIdentify> {
        for i in 0..BUFFER_N {
            // lock status
            let mut status = self.buf_status[i].lock().await;
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
        crate::assert!(id.id < BUFFER_N as u32);

        // lock status
        let mut status = self.buf_status[id.id as usize].lock().await;
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
        crate::assert!(id.id < BUFFER_N as u32);
        // SAFETY: id is valid
        let mut status = self.buf_status[id.id as usize].lock().await;
        if !matches!(status.borrow()[0], BufferStatus::InUse { .. }) {
            crate::unreachable!("free failed");
        }

        self.buffers[id.id as usize].lock().await
    }
}
