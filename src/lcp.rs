//! lcp\[i\] = max_pref(sa\[i\], sa\[i - 1\]) and lcp.len() == sa.len()

use core::slice::{Iter, SliceIndex};
use core::ops::Index;

#[derive(Debug, Clone)]
pub struct LCP(Vec<usize>);
impl LCP {

    pub(crate) fn new(lcp: Vec<usize>) -> Self {
        Self(lcp)
    }

    /// Return ref to inner slice
    /// ```
    /// use suff_collections::{array::*, lcp::*};
    ///
    /// let sa = SuffixArray::new("word");
    /// let lcp = sa.lcp();
    /// let inner_lcp: &[usize] = lcp.inner();
    /// ```
    #[inline]
    pub fn inner(&self) -> &[usize] {
        &self.0
    }

    /// Move inner vec
    /// ```
    /// use suff_collections::{array::*, lcp::*};
    ///
    /// let sa = SuffixArray::new("word");
    /// let lcp = sa.lcp();
    /// let inner_lcp: Vec<usize> = lcp.owned();
    /// // lcp not valid
    /// ```
    #[inline]
    pub fn owned(self) -> Vec<usize> {
        self.0
    }

    /// Return iterator to inner slice
    /// ```
    /// use suff_collections::{array::*, lcp::*};
    ///
    /// let sa = SuffixArray::new("word");
    /// let lcp = sa.lcp();
    /// let copy = lcp.iter().map(|&x| x).collect::<Vec<_>>();
    /// ```
    #[inline]
    pub fn iter(&self) -> Iter<'_, usize> {
        self.0.iter()
    }

    /// Return length lcp
    /// ```
    /// use suff_collections::{array::*, lcp::*};
    ///
    /// let sa = SuffixArray::new("word");
    /// let lcp_len: usize = sa.lcp().len();
    /// assert_eq!("word\0".len(), lcp_len);
    /// ```
    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Return value by index. In debug safe. In release disable bound checks
    /// ```
    /// use suff_collections::{array::*, lcp::*};
    ///
    /// let sa = SuffixArray::new("word");
    /// let lcp = sa.lcp();
    /// // safe because "word".len() >= 0
    /// let len = unsafe { lcp.idx(0) };
    /// ```
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