#![feature(generic_const_exprs)]

use core::{
    borrow::{Borrow, BorrowMut},
    cell::RefCell,
    future::Future,
};

use cortex_m::interrupt::free;
use embassy_sync::{
    blocking_mutex::{raw::CriticalSectionRawMutex, CriticalSectionMutex},
    mutex::{Mutex, MutexGuard},
};

/// General Data Buffer
#[derive(Copy, Clone, Eq, PartialEq, defmt::Format)]
pub struct BufferIdentify<AlocTag: Copy + Clone + Eq + PartialEq, const BUFFER_SIZE: usize> {
    pub buf_index: u32,
    pub alloc_tag: AlocTag,
}

/// Buffer status
#[derive(Copy, Clone, Eq, PartialEq, defmt::Format)]
pub enum BufferStatus<AllocTag: Copy + Clone + Eq + PartialEq + Eq + PartialEq> {
    /// not used
    Free,
    /// in use
    Busy { alloc_tag: AllocTag },
}

/// Shared fixed-size buffer manager
#[derive(defmt::Format)]
pub struct SharedBufferManager<
    AllocTag: Copy + Clone + Eq + PartialEq,
    const BUFFER_SIZE: usize,
    const BUFFER_N: usize,
> {
    /// Shared buffer
    /// TODO: SharedBufferManager自体がMutexを持っているので、buffersが個別にMutexを持っているが、親に束縛されている可能性がある。再考の余地あり
    pub buffers: [Mutex<CriticalSectionRawMutex, [u8; BUFFER_SIZE]>; BUFFER_N],
    /// Buffer status
    pub statuses: [Mutex<CriticalSectionRawMutex, [BufferStatus<AllocTag>; BUFFER_SIZE]>; BUFFER_N],
}

impl<Alloctag: Copy + Clone + Eq + PartialEq, const BUFFER_SIZE: usize>
    BufferIdentify<Alloctag, BUFFER_SIZE>
{
    pub fn new(buf_index: u32, alloc_tag: Alloctag) -> Self {
        Self {
            buf_index,
            alloc_tag,
        }
    }
}

impl<AllocTag: Copy + Clone + Eq + PartialEq> BufferStatus<AllocTag> {
    pub fn free() -> Self {
        Self::Free
    }
    pub fn in_use(alloc_tag: AllocTag) -> Self {
        Self::Busy { alloc_tag }
    }
}

impl<AllocTag: Copy + Clone + Eq + PartialEq, const BUFFER_SIZE: usize, const BUFFER_N: usize>
    SharedBufferManager<AllocTag, BUFFER_SIZE, BUFFER_N>
{
    pub fn new() -> Self {
        Self {
            buffers: [const { Mutex::new([0; BUFFER_SIZE]) }; BUFFER_N],
            statuses: core::array::from_fn(|_| {
                Mutex::new([BufferStatus::<AllocTag>::free(); BUFFER_SIZE])
            }),
        }
    }

    /// Allocate buffer
    pub async fn allocate(
        &mut self,
        user_tag: AllocTag,
    ) -> Option<BufferIdentify<AllocTag, BUFFER_SIZE>> {
        for i in 0..BUFFER_N {
            // lock status
            let mut status = self.statuses[i].lock().await;
            // check if free
            if status.borrow()[0] != BufferStatus::Free {
                continue;
            }
            // set in use
            status.borrow_mut()[0] = BufferStatus::Busy {
                alloc_tag: user_tag,
            };
            return Some(BufferIdentify::<AllocTag, BUFFER_SIZE>::new(
                i as u32, user_tag,
            ));
        }
        crate::warn!("allocate failed");
        None
    }

    /// Allocate buffer with retry
    pub async fn allocate_with_retry<DelayF, Fut>(
        &mut self,
        user_tag: AllocTag,
        delay_func: DelayF,
        retry_count_max: u32,
    ) -> Option<BufferIdentify<AllocTag, BUFFER_SIZE>>
    where
        DelayF: Fn() -> Fut,
        Fut: Future<Output = ()>,
    {
        for i in 0..retry_count_max {
            if let Some(buf) = self.allocate(user_tag).await {
                return Some(buf);
            }
            // 他Taskにリソースを奪われた場合は、再度リクエストを送る
            // ただし、即リクエストしてもリソース開放に至らないケースがあるので、時間Delayを入れる
            delay_func().await;
        }
        None
    }

    /// user_tag を取得
    pub async fn user_tag(
        &mut self,
        id: BufferIdentify<AllocTag, BUFFER_SIZE>,
    ) -> Option<AllocTag> {
        crate::assert!(id.buf_index < BUFFER_N as u32);

        // lock status
        let status = self.statuses[id.buf_index as usize].lock().await;
        match status.borrow()[0] {
            BufferStatus::Busy { alloc_tag } => Some(alloc_tag),
            _ => None,
        }
    }

    /// Free buffer
    pub async fn free(&mut self, id: BufferIdentify<AllocTag, BUFFER_SIZE>) {
        crate::assert!(id.buf_index < BUFFER_N as u32);

        // lock status
        let mut status = self.statuses[id.buf_index as usize].lock().await;
        // check if in use
        if !matches!(status.borrow()[0], BufferStatus::Busy { .. }) {
            crate::warn!("free failed");
        }
        // set free
        status.borrow_mut()[0] = BufferStatus::Free;
    }

    /// Get buffer body (mutable)
    pub async fn lock_buffer(
        &mut self,
        id: BufferIdentify<AllocTag, BUFFER_SIZE>,
    ) -> MutexGuard<'_, CriticalSectionRawMutex, [u8; BUFFER_SIZE]> {
        crate::assert!(id.buf_index < BUFFER_N as u32);
        // SAFETY: id is valid
        let mut status = self.statuses[id.buf_index as usize].lock().await;
        if !matches!(status.borrow()[0], BufferStatus::Busy { .. }) {
            crate::unreachable!("free failed");
        }

        self.buffers[id.buf_index as usize].lock().await
    }
}
