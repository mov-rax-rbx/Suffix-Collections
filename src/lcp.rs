//! lcp\[i\] = max_pref(sa\[i\], sa\[i - 1\]) and lcp.len() == sa.len()

use crate::array::build_suffix_array::SuffixIndices;
use alloc::vec::Vec;
use core::ops::Index;
use core::slice::{Iter, SliceIndex};

#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct LCP<T: SuffixIndices<T>>(Vec<T>);
impl<T: SuffixIndices<T>> LCP<T> {
    pub(crate) fn new(lcp: Vec<T>) -> Self {
        Self(lcp)
    }

    /// Return ref to inner slice
    /// ```
    /// use suff_collections::{array::*, lcp::*};
    ///
    /// let sa = SuffixArray::<usize>::new("word");
    /// let lcp = sa.lcp();
    /// let inner_lcp: &[usize] = lcp.inner();
    /// ```
    #[inline]
    pub fn inner(&self) -> &[T] {
        &self.0
    }

    /// Move inner vec
    /// ```
    /// use suff_collections::{array::*, lcp::*};
    ///
    /// let sa = SuffixArray::<usize>::new("word");
    /// let lcp = sa.lcp();
    /// let inner_lcp: Vec<usize> = lcp.owned();
    /// // lcp not valid
    /// ```
    #[inline]
    pub fn owned(self) -> Vec<T> {
        self.0
    }

    /// Return iterator to inner slice
    /// ```
    /// use suff_collections::{array::*, lcp::*};
    ///
    /// let sa = SuffixArray::<usize>::new("word");
    /// let lcp = sa.lcp();
    /// let copy = lcp.iter().map(|&x| x).collect::<Vec<_>>();
    /// ```
    #[inline]
    pub fn iter(&self) -> Iter<'_, T> {
        self.0.iter()
    }

    /// Return length lcp
    /// ```
    /// use suff_collections::{array::*, lcp::*};
    ///
    /// let sa = SuffixArray::<usize>::new("word");
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
    /// let sa = SuffixArray::<usize>::new("word");
    /// let lcp = sa.lcp();
    /// // safe because "word".len() >= 0
    /// let len = unsafe { lcp.idx(0) };
    /// ```
    #[cfg(debug_assertions)]
    #[inline]
    pub unsafe fn idx<I: SliceIndex<[T]>>(&self, index: I) -> &I::Output {
        &self.0[index]
    }
    #[cfg(not(debug_assertions))]
    #[inline]
    pub unsafe fn idx<I: SliceIndex<[T]>>(&self, index: I) -> &I::Output {
        self.0.get_unchecked(index)
    }

    #[cfg(debug_assertions)]
    #[inline]
    pub(crate) unsafe fn idx_mut<I: SliceIndex<[T]>>(&mut self, index: I) -> &mut I::Output {
        &mut self.0[index]
    }
    #[cfg(not(debug_assertions))]
    #[inline]
    pub(crate) unsafe fn idx_mut<I: SliceIndex<[T]>>(&mut self, index: I) -> &mut I::Output {
        self.0.get_unchecked_mut(index)
    }
}

impl<T: SuffixIndices<T>, I: SliceIndex<[T]>> Index<I> for LCP<T> {
    type Output = I::Output;

    #[inline]
    fn index(&self, index: I) -> &Self::Output {
        &self.0.index(index)
    }
}
