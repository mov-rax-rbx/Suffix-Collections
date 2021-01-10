use alloc::vec::Vec;

#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
#[repr(transparent)]
pub(crate) struct Byte(pub(crate) u8);

impl Byte {
    /// 2^3 = 8
    pub const LOG2: usize = 3;
    /// 8 = 00000111
    pub const MASK: usize = 0b111;

    #[inline]
    pub fn vec(n: usize) -> Vec<Byte> {
        vec![Byte(0); num_to_byte_pack(n)]
    }

    #[inline]
    pub unsafe fn get_unchecked(self, n: usize) -> u8 {
        debug_assert!(n < 8);
        (self.0 >> n) & 0b1
    }
    #[inline]
    pub unsafe fn set_unchecked(&mut self, n: usize) {
        debug_assert!(n < 8);
        self.0 |= 0b1 << n;
    }
}

impl Default for Byte {
    #[inline]
    fn default() -> Self {
        Byte(0)
    }
}

pub(crate) trait Bit {
    unsafe fn get_unchecked(&self, idx: usize) -> u8;
}

pub(crate) trait BitMut: Bit {
    unsafe fn set_unchecked(&mut self, idx: usize);
    // unsafe fn unset_unchecked(&mut self, idx: usize);
    unsafe fn range_to_mut(&mut self, to: usize) -> Self;
    fn clear(&mut self);
}

#[repr(transparent)]
pub(crate) struct BitArrMut<'b>(pub(crate) &'b mut [Byte]);
impl<'b> BitMut for BitArrMut<'b> {
    #[inline]
    unsafe fn range_to_mut(&mut self, end: usize) -> Self {
        debug_assert!(num_to_byte_pack(end) <= self.0.len());
        // safe cast because Rust can't deduce that we won't return multiple references to the same value
        Self(&mut *(self.0.get_unchecked_mut(..num_to_byte_pack(end)) as *mut _))
    }
    #[inline]
    unsafe fn set_unchecked(&mut self, n: usize) {
        debug_assert!(hi(n) < self.0.len());
        self.0.get_unchecked_mut(hi(n)).set_unchecked(lo(n));
    }
    #[inline]
    fn clear(&mut self) {
        self.0.iter_mut().for_each(|x| *x = Byte::default());
    }
}

impl<'b> Bit for BitArrMut<'b> {
    #[inline]
    unsafe fn get_unchecked(&self, n: usize) -> u8 {
        debug_assert!(hi(n) < self.0.len());
        self.0.get_unchecked(hi(n)).get_unchecked(lo(n))
    }
}

#[repr(transparent)]
pub(crate) struct BitArr<'b>(pub(crate) &'b [Byte]);
impl<'b> Bit for BitArr<'b> {
    #[inline]
    unsafe fn get_unchecked(&self, n: usize) -> u8 {
        debug_assert!(hi(n) < self.0.len());
        self.0.get_unchecked(hi(n)).get_unchecked(lo(n))
    }
}

#[inline]
fn hi(n: usize) -> usize {
    n >> Byte::LOG2
}

#[inline]
fn lo(n: usize) -> usize {
    n & Byte::MASK
}

#[inline]
fn num_to_byte_pack(n: usize) -> usize {
    hi(n) + if lo(n) == 0 { 0 } else { 1 }
}

mod test {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn t1() {
        let mut slice = Byte::vec(8);
        let mut bit_arr = BitArrMut(&mut slice);
        // safe because test run in debug
        unsafe {
            bit_arr.set_unchecked(0);
            bit_arr.set_unchecked(1);
            bit_arr.set_unchecked(2);
            bit_arr.set_unchecked(3);
            bit_arr.set_unchecked(4);
            bit_arr.set_unchecked(5);
            bit_arr.set_unchecked(6);
            bit_arr.set_unchecked(7);
        }
        assert_eq!(0b11111111, bit_arr.0[0].0);
    }

    #[test]
    fn t2() {
        let mut slice = Byte::vec(10);
        let mut bit_arr = BitArrMut(&mut slice);
        // safe because test run in debug
        unsafe {
            bit_arr.set_unchecked(0);
            bit_arr.set_unchecked(2);
            bit_arr.set_unchecked(4);
            bit_arr.set_unchecked(6);
        }
        assert_eq!(0b01010101, bit_arr.0[0].0);
    }

    #[test]
    fn t3() {
        let mut slice = Byte::vec(20);
        let mut bit_arr = BitArrMut(&mut slice);
        // safe because test run in debug
        unsafe {
            bit_arr.set_unchecked(1);
            bit_arr.set_unchecked(2);
            bit_arr.set_unchecked(3);
            bit_arr.set_unchecked(4);
            bit_arr.set_unchecked(5);
            bit_arr.set_unchecked(6);
            bit_arr.set_unchecked(7);

            bit_arr.set_unchecked(8);
            bit_arr.set_unchecked(9);
            bit_arr.set_unchecked(10);
            bit_arr.set_unchecked(11);
            bit_arr.set_unchecked(12);
            bit_arr.set_unchecked(13);
            bit_arr.set_unchecked(14);
            bit_arr.set_unchecked(15);

            bit_arr.set_unchecked(18);
            bit_arr.set_unchecked(19);
            bit_arr.set_unchecked(20);
            bit_arr.set_unchecked(21);
        }
        assert_eq!(0b11111110, bit_arr.0[0].0);
        assert_eq!(0b11111111, bit_arr.0[1].0);
        assert_eq!(0b00111100, bit_arr.0[2].0);
    }
}
