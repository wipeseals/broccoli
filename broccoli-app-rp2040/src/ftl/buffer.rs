#![feature(generic_const_exprs)]

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
#[derive(Copy, Clone, Eq, PartialEq, defmt::Format)]
pub struct SharedBufferManager<const TOTAL_BUFFER_SIZE: usize, const BUFFER_N: usize> {
    /// Shared buffer
    pub work_buf: [u8; TOTAL_BUFFER_SIZE],
    /// Buffer status
    pub buf_status: [BufferStatus; BUFFER_N],
    /// Buffer size
    pub buffer_size: usize,
    /// Free buffer count
    pub free_count: usize,
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

impl<const TOTAL_BUFFER_SIZE: usize, const BUFFER_N: usize>
    SharedBufferManager<TOTAL_BUFFER_SIZE, BUFFER_N>
{
    pub fn new() -> Self {
        Self {
            work_buf: [0; TOTAL_BUFFER_SIZE],
            buf_status: [BufferStatus::Free; BUFFER_N],
            buffer_size: TOTAL_BUFFER_SIZE / BUFFER_N,
            free_count: BUFFER_N,
        }
    }

    /// Get buffer status
    pub fn buf_status(&self, id: BufferIdentify) -> BufferStatus {
        crate::assert!(id.id < BUFFER_N as u32);
        self.buf_status[id.id as usize]
    }

    /// Get buffer size
    pub fn buf_size(&self) -> usize {
        self.buffer_size
    }

    /// Get free buffer count
    pub fn free_count(&self) -> usize {
        self.free_count
    }

    /// allocatable
    pub fn allocatable(&self) -> bool {
        self.free_count > 0
    }

    /// Allocate buffer
    pub fn allocate(&mut self, user_tag: u32) -> Option<BufferIdentify> {
        if !self.allocatable() {
            return None;
        }

        for i in 0..BUFFER_N {
            match self.buf_status[i] {
                BufferStatus::Free => {
                    self.buf_status[i] = BufferStatus::InUse { user_tag };
                    self.free_count -= 1;
                    return Some(BufferIdentify::new(i as u32));
                }
                _ => {}
            }
        }
        crate::unreachable!("allocate failed");
        None
    }

    /// Free buffer
    pub fn free(&mut self, id: BufferIdentify) {
        crate::assert!(id.id < BUFFER_N as u32);
        // SAFETY: id is valid
        if !matches!(self.buf_status[id.id as usize], BufferStatus::InUse { .. }) {
            crate::unreachable!("free failed");
            return;
        }

        self.buf_status[id.id as usize] = BufferStatus::Free;
        self.free_count += 1;
    }

    /// Get buffer body (mutable)
    pub fn buf_slice_mut(&mut self, id: BufferIdentify) -> Option<&mut [u8]> {
        crate::assert!(id.id < BUFFER_N as u32);
        // SAFETY: id is valid
        if !matches!(self.buf_status[id.id as usize], BufferStatus::InUse { .. }) {
            crate::unreachable!("buf_slice failed");
            return None;
        }

        let start_addr = id.id as usize * TOTAL_BUFFER_SIZE / BUFFER_N;
        let end_addr = (id.id as usize + 1) * TOTAL_BUFFER_SIZE / BUFFER_N;
        Some(&mut self.work_buf[start_addr..end_addr])
    }

    /// Get buffer body (immutable)
    pub fn buf_slice(&self, id: BufferIdentify) -> Option<&[u8]> {
        crate::assert!(id.id < BUFFER_N as u32);
        // SAFETY: id is valid
        if !matches!(self.buf_status[id.id as usize], BufferStatus::InUse { .. }) {
            crate::unreachable!("buf_slice failed");
            return None;
        }

        let start_addr = id.id as usize * TOTAL_BUFFER_SIZE / BUFFER_N;
        let end_addr = (id.id as usize + 1) * TOTAL_BUFFER_SIZE / BUFFER_N;
        Some(&self.work_buf[start_addr..end_addr])
    }

    /// Get buffer body (pointer, mutable)
    pub fn buf_ptr_mut(&mut self, id: BufferIdentify) -> Option<*mut u8> {
        crate::assert!(id.id < BUFFER_N as u32);
        // SAFETY: id is valid
        if !matches!(self.buf_status[id.id as usize], BufferStatus::InUse { .. }) {
            crate::unreachable!("buf_ptr failed");
            return None;
        }

        let start_addr = id.id as usize * TOTAL_BUFFER_SIZE / BUFFER_N;
        Some(&mut self.work_buf[start_addr] as *mut u8)
    }
}
