use core::slice::{Iter, SliceIndex};
use core::ops::Index;

#[derive(Debug, Clone)]
/// lcp\[i\] = max_pref(sa\[i\], sa\[i - 1\]) and lcp.len() == sa.len()
pub struct LCP(Vec<usize>);

impl LCP {

    pub(crate) fn new(lcp: Vec<usize>) -> Self {
        Self(lcp)
    }

    #[inline]
    pub fn inner(&self) -> &[usize] {
        &self.0
    }

    #[inline]
    pub fn owned(self) -> Vec<usize> {
        self.0
    }

    #[inline]
    pub fn iter(&self) -> Iter<'_, usize> {
        self.0.iter()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[cfg(debug_assertions)]
    #[inline]
    pub unsafe fn idx<I: SliceIndex<[usize]>>(&self, index: I) -> &I::Output {
        &self.0[index]
    }
    #[cfg(not(debug_assertions))]
    #[inline]
    pub unsafe fn idx<I: SliceIndex<[usize]>>(&self, index: I) -> &I::Output {
        self.0.get_unchecked(index)
    }

    #[cfg(debug_assertions)]
    #[inline]
    pub(crate) unsafe fn idx_mut<I: SliceIndex<[usize]>>(&mut self, index: I) -> &mut I::Output {
        &mut self.0[index]
    }
    #[cfg(not(debug_assertions))]
    #[inline]
    pub(crate) unsafe fn idx_mut<I: SliceIndex<[usize]>>(&mut self, index: I) -> &mut I::Output {
        self.0.get_unchecked_mut(index)
    }
}

impl<I: SliceIndex<[usize]>> Index<I> for LCP {
    type Output = I::Output;

    #[inline]
    fn index(&self, index: I) -> &Self::Output {
        &self.0.index(index)
    }
}