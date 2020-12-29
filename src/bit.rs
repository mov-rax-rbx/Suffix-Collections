use alloc::vec::Vec;
use core::fmt;

#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
#[repr(transparent)]
pub(crate) struct Byte(u8);

impl Byte {
    /// 2^3 = 8
    const LOG2: usize = 3;
    /// 8 = 00000111
    const MASK: usize = 0b111;

    #[inline]
    pub fn unwrap(self) -> u8 {
        self.0
    }

    pub fn vec(n: usize) -> Vec<Byte> {
        vec![Byte(0); hi(n) + if lo(n) == 0 { 0 } else { 1 }]
    }
}

impl Default for Byte {
    fn default() -> Self {
        Byte(0)
    }
}

impl fmt::Debug for Byte {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("dec {:?}, bin: {:#b}", self.0, self.0))?;
        Ok(())
    }
}

pub(crate) trait Bit {
    unsafe fn set_unchecked(&mut self, idx: usize);
    // unsafe fn unset_unchecked(&mut self, idx: usize);
    unsafe fn get_unchecked(&self, idx: usize) -> bool;
    unsafe fn range_to_mut(&mut self, to: usize) -> Self;
    fn clear(&mut self);
}

#[repr(transparent)]
pub(crate) struct BitArrMut<'b>(pub(crate) &'b mut [Byte]);
impl<'b> Bit for BitArrMut<'b> {
    #[cfg(debug_assertions)]
    #[inline]
    unsafe fn range_to_mut(&mut self, end: usize) -> Self {
        // safe cast because Rust can't deduce that we won't return multiple references to the same value
        Self(&mut *(&mut self.0[..hi(end) + if lo(end) == 0 { 0 } else { 1 }] as *mut _))
    }
    #[cfg(not(debug_assertions))]
    #[inline]
    unsafe fn range_to_mut(&mut self, end: usize) -> Self {
        Self(&mut *(self.0.get_unchecked_mut(..hi(end) + if lo(end) == 0 { 0 } else { 1 }) as *mut _))
    }

    #[cfg(debug_assertions)]
    #[inline]
    unsafe fn get_unchecked(&self, n: usize) -> bool {
        ((self.0[hi(n)].unwrap() >> lo(n)) & 0b1) == 1
    }
    #[cfg(not(debug_assertions))]
    #[inline]
    unsafe fn get_unchecked(&self, n: usize) -> bool {
        ((self.0.get_unchecked(hi(n)).unwrap() >> lo(n)) & 0b1) == 1
    }

    #[cfg(debug_assertions)]
    #[inline]
    unsafe fn set_unchecked(&mut self, n: usize) {
        self.0[hi(n)] = Byte(
            self.0[hi(n)].unwrap() | (0b1 << lo(n))
        );
    }
    #[cfg(not(debug_assertions))]
    #[inline]
    unsafe fn set_unchecked(&mut self, n: usize) {
        *self.0.get_unchecked_mut(hi(n)) = Byte(
            self.0.get_unchecked_mut(hi(n)).unwrap() | (0b1 << lo(n))
        );
    }

    // #[allow(dead_code)]
    // #[cfg(debug_assertions)]
    // #[inline]
    // unsafe fn unset_unchecked(&mut self, n: usize) {
    //     self.0[hi(n)] = Byte(
    //         self.0[hi(n)].unwrap() ^ (0b1 << lo(n))
    //     );
    // }
    // #[cfg(not(debug_assertions))]
    // #[inline]
    // unsafe fn unset_unchecked(&mut self, n: usize) {
    //     *self.0.get_unchecked_mut(hi(n)) = Byte(
    //         self.0.get_unchecked_mut(hi(n)).unwrap() ^ (0b1 << lo(n))
    //     );
    // }

    #[inline]
    fn clear(&mut self) {
        self.0.iter_mut().for_each(|x| *x = Byte::default());
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
        assert_eq!(0b11111111, bit_arr.0[0].unwrap());
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
        assert_eq!(0b01010101, bit_arr.0[0].unwrap());
    }

    // #[test]
    // fn t3() {
    //     let mut slice = Byte::vec(10);
    //     let mut bit_arr = BitArrMut(&mut slice);
    //     // safe because test run in debug
    //     unsafe {
    //         bit_arr.set_unchecked(0);
    //         bit_arr.set_unchecked(1);
    //         bit_arr.set_unchecked(2);
    //         bit_arr.set_unchecked(3);
    //         bit_arr.set_unchecked(4);
    //         bit_arr.set_unchecked(5);
    //         bit_arr.set_unchecked(6);
    //         bit_arr.set_unchecked(7);
    //         bit_arr.unset_unchecked(1);
    //         bit_arr.unset_unchecked(3);
    //         bit_arr.unset_unchecked(5);
    //         bit_arr.unset_unchecked(7);
    //     }
    //     assert_eq!(0b01010101, bit_arr.0[0].unwrap());
    // }

    #[test]
    fn t4() {
        let mut slice = Byte::vec(20);
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
        assert_eq!(0b11111111, bit_arr.0[0].unwrap());
        assert_eq!(0b11111111, bit_arr.0[1].unwrap());
        assert_eq!(0b00111100, bit_arr.0[2].unwrap());
    }

    // #[test]
    // fn t5() {
    //     let mut slice = Byte::vec(20);
    //     let mut bit_arr = BitArrMut(&mut slice);
    //     // safe because test run in debug
    //     unsafe {
    //         bit_arr.set_unchecked(0);
    //         bit_arr.set_unchecked(1);
    //         bit_arr.set_unchecked(2);
    //         bit_arr.set_unchecked(3);
    //         bit_arr.set_unchecked(4);
    //         bit_arr.set_unchecked(5);
    //         bit_arr.set_unchecked(6);
    //         bit_arr.set_unchecked(7);

    //         bit_arr.set_unchecked(8);
    //         bit_arr.set_unchecked(9);
    //         bit_arr.set_unchecked(10);
    //         bit_arr.set_unchecked(11);
    //         bit_arr.set_unchecked(12);
    //         bit_arr.set_unchecked(13);
    //         bit_arr.set_unchecked(14);
    //         bit_arr.set_unchecked(15);

    //         bit_arr.set_unchecked(18);
    //         bit_arr.set_unchecked(19);
    //         bit_arr.set_unchecked(20);
    //         bit_arr.set_unchecked(21);

    //         bit_arr.unset_unchecked(9);
    //         bit_arr.unset_unchecked(10);
    //         bit_arr.unset_unchecked(11);
    //         bit_arr.unset_unchecked(12);
    //     }
    //     assert_eq!(0b11111111, bit_arr.0[0].unwrap());
    //     assert_eq!(0b11100001, bit_arr.0[1].unwrap());
    //     assert_eq!(0b00111100, bit_arr.0[2].unwrap());
    // }
}