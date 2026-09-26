/// The positions of the set bits of a bitmask, lowest first.
///
/// Each step is a `trailing_zeros` and clearing the lowest bit, so it costs
/// one step per set bit whatever the width, and the length is known from
/// `count_ones` up front.
#[derive(Debug, Clone, Copy)]
pub struct Bits<T>(pub T);

macro_rules! impl_bits {
    ($($t:ty),*) => {$(
        impl Iterator for Bits<$t> {
            type Item = u8;

            #[inline]
            fn next(&mut self) -> Option<u8> {
                if self.0 == 0 {
                    return None;
                }
                let b = self.0.trailing_zeros() as u8;
                self.0 &= self.0 - 1;
                Some(b)
            }

            #[inline]
            fn size_hint(&self) -> (usize, Option<usize>) {
                let n = self.0.count_ones() as usize;
                (n, Some(n))
            }
        }

        impl ExactSizeIterator for Bits<$t> {}
    )*};
}

impl_bits!(u8, u16, u32, u64, u128);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bits() {
        assert_eq!(Bits(0u8).collect::<Vec<_>>(), []);
        assert_eq!(Bits(0b10110u8).collect::<Vec<_>>(), [1, 2, 4]);
        assert_eq!(Bits(0b10110u8).len(), 3);
        assert_eq!(Bits(1u16 << 15 | 1).collect::<Vec<_>>(), [0, 15]);
        assert_eq!(Bits(1u128 << 127 | 1 << 64).collect::<Vec<_>>(), [64, 127]);
    }
}
