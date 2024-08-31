pub const U32_BIT_WIDTH: usize = 32;

/// A bit array with a fixed number of bits.
/// generic_const_exprs使えるならbitwidthからword数求めたかったが今は無理
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct BitArr<const U32DATA_N: usize> {
    pub data: [u32; U32DATA_N],
}

impl<const U32DATA_N: usize> Default for BitArr<U32DATA_N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const U32DATA_N: usize> BitArr<U32DATA_N> {
    pub fn new() -> Self {
        Self {
            data: [0; U32DATA_N],
        }
    }
    /// Returns the number of bits in the bit array.
    pub fn get_idx(&self, idx: usize) -> (usize, usize) {
        let data_idx = idx / U32_BIT_WIDTH;
        let bit_idx = idx % U32_BIT_WIDTH;
        assert!(data_idx < self.data.len());

        (data_idx, bit_idx)
    }
    /// Returns the number of bits in the bit array.
    pub fn get(&self, idx: usize) -> bool {
        let (data_idx, bit_idx) = self.get_idx(idx);
        (self.data[data_idx] & (1 << bit_idx)) != 0
    }

    /// Sets the bit at the given index to the given value.
    pub fn set(&mut self, idx: usize, value: bool) {
        let (data_idx, bit_idx) = self.get_idx(idx);
        if value {
            self.data[data_idx] |= 1 << bit_idx;
        } else {
            self.data[data_idx] &= !(1 << bit_idx);
        }
    }
    /// set all bits to value
    pub fn fill(&mut self, value: bool) {
        for i in 0..self.data.len() {
            self.data[i] = if value { !0 } else { 0 };
        }
    }
    /// Return the numer of words.
    pub fn data_len(&self) -> usize {
        self.data.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_bitarr() {
        const U32DATA_N: usize = 16;
        let mut bitarr = BitArr::<U32DATA_N>::new();
        for i in 0..U32DATA_N {
            assert!(!bitarr.get(i));
        }
        assert_eq!(bitarr.data_len(), U32DATA_N);
    }

    #[test]
    fn test_set_bitarr() {
        const U32DATA_N: usize = 16;
        let mut bitarr = BitArr::<U32DATA_N>::new();
        for i in 0..U32DATA_N {
            bitarr.set(i, true);
            assert!(bitarr.get(i));
            bitarr.set(i, false);
            assert!(!bitarr.get(i));
        }
    }

    #[test]
    fn test_fill_bitarr() {
        const U32DATA_N: usize = 16;
        let mut bitarr = BitArr::<U32DATA_N>::new();
        bitarr.fill(true);
        for i in 0..U32DATA_N {
            assert!(bitarr.get(i));
        }
        bitarr.fill(false);
        for i in 0..U32DATA_N {
            assert!(!bitarr.get(i));
        }
    }
}
