//! Implementation of the [suffix array](https://www.researchgate.net/profile/Daricks_Wai_Hong_Chan/publication/221577802_Linear_Suffix_Array_Construction_by_Almost_Pure_Induced-Sorting/links/00b495318a21ba484f000000/Linear-Suffix-Array-Construction-by-Almost-Pure-Induced-Sorting.pdf?origin=publication_detail)
//! construction of which is performed in linear time

//! # Examples
//!
//! ```
//! use suff_collections::{array::*, tree::*, lcp::*};
//!
//! // let word = "Some word";
//! let word: &str = "Some word\0";
//! let find: &str = "word";
//!
//! // construct suffix array
//! // let sa = SuffixArray::<usize>::new_stack(word);
//! // let sa = SuffixArray::<u8>::new(word);
//! // let sa = SuffixArray::<u16>::new(word);
//! // let sa = SuffixArray::<u32>::new(word);
//! let sa = SuffixArray::<usize>::new(word);
//!
//! // construct lcp
//! // lcp[i] = max_pref(sa[i], sa[i - 1]) && lcp.len() == sa.len()
//! let lcp: LCP<usize> = sa.lcp();
//!
//! // finds the entry position of the line 'find' in 'word'
//! // O(|find| * log(|word|))
//! let res: Option<usize> = sa.find(find);
//!
//! // finds all the entry position of the line 'find' in 'word'
//! // O(|find| * log(|word|))
//! let res_all: &[usize] = sa.find_all(find);
//!
//! // finds the entry position of the line 'find' in 'word'
//! // O(|word|)
//! let res: Option<usize> = sa.find_big(&sa.lcp(), find);
//!
//! // finds all the entry position of the line 'find' in 'word'
//! // O(|word|)
//! let res_all: &[usize] = sa.find_all_big(&sa.lcp(), find);
//!
//! // convert suffix array to suffix tree
//! let st = SuffixTree::from(sa);
//! ```

use alloc::borrow::{Cow, ToOwned};
use alloc::vec::{IntoIter, Vec};
use core::{
    cmp::{max, Eq},
    option::Option,
    slice::Iter,
    str,
};

use crate::{bit::*, canonic_word, lcp::*, tree::*};
use build_suffix_array::{Max, SuffixIndices};

#[repr(transparent)]
struct ByteSliceMut<'t>(&'t mut [Byte]);

impl<'t> BitMut for ByteSliceMut<'t> {
    #[inline]
    unsafe fn set_unchecked(&mut self, idx: usize) {
        *self.0.get_unchecked_mut(idx) = Byte(1);
    }
    #[inline]
    unsafe fn range_to_mut(&mut self, to: usize) -> Self {
        // safe cast because Rust can't deduce that we won't
        // return multiple references to the same value
        Self(&mut *(self.0.get_unchecked_mut(..to) as *mut _))
    }
    #[inline]
    fn clear(&mut self) {
        self.0.iter_mut().for_each(|x| *x = Default::default());
    }
}

impl<'t> Bit for ByteSliceMut<'t> {
    #[inline]
    unsafe fn raw(&self) -> &[Byte] {
        &*self.0
    }
    #[inline]
    unsafe fn get_unchecked(&self, idx: usize) -> u8 {
        self.0.get_unchecked(idx).0
    }
}

#[repr(transparent)]
struct ByteSlice<'t>(&'t [Byte]);
impl<'t> Bit for ByteSlice<'t> {
    #[inline]
    unsafe fn raw(&self) -> &[Byte] {
        &*self.0
    }
    #[inline]
    unsafe fn get_unchecked(&self, idx: usize) -> u8 {
        self.0.get_unchecked(idx).0
    }
}

#[derive(Debug, Clone)]
pub struct SuffixArray<'sa, T: SuffixIndices<T>> {
    word: Cow<'sa, str>,
    sa: Vec<T>,
}

impl<'sa, T: SuffixIndices<T>> IntoIterator for SuffixArray<'sa, T> {
    type Item = T;
    type IntoIter = IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.sa.into_iter()
    }
}

impl<'sa, T: SuffixIndices<T>> IntoIterator for &'sa SuffixArray<'sa, T> {
    type Item = &'sa T;
    type IntoIter = Iter<'sa, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.sa.iter()
    }
}

impl<'sa, T: SuffixIndices<T>> SuffixArray<'sa, T> {
    const DICT_SIZE: usize = 256;

    /// Construct suffix array recursive. Complexity O(n).
    /// Uses less memory to build than `new` by using bitpcking.
    /// ```
    /// use suff_collections::array::*;
    ///
    /// // let sa = SuffixArray::<u8>::new_compress("word");
    /// // let sa = SuffixArray::<u16>::new_compress("word");
    /// // let sa = SuffixArray::<u32>::new_compress("word");
    /// let sa = SuffixArray::<usize>::new_compress("word");
    /// let sa = SuffixArray::<usize>::new_compress("word\0");
    /// ```
    /// At the end of the line should hit '\0'.
    /// If there is no '\0' at the end then the line will be copied and added '\0' to the end.
    /// Otherwise, the value will be taken by reference.
    ///
    /// # Panics
    ///
    /// This function will panic if word.len() > T::MAX.
    pub fn new_compress(word: &'sa str) -> Self {
        assert!(word.len() < <T as Max>::max().to_usize());
        if word.is_empty() {
            return Self {
                word: Cow::from(""),
                sa: vec![],
            };
        }
        let word = canonic_word(word);
        let mut offset_dict =
            vec![(T::zero(), T::zero()); max(word.len(), SuffixArray::<T>::DICT_SIZE)];
        let mut tmp_end_s = vec![T::zero(); offset_dict.len()];
        let mut sa = vec![T::zero(); word.len()];
        let mut sa_init = Byte::vec(word.len());
        // safe because
        //      offset_dict.len() > max(s_idx) && offset_dict.len() >= s_idx.len()
        //      tmp_end_s.len() == offset_dict.len()
        //      sa.len() == s_idx.len()
        //      sa_init.len() == sa.len()
        //      s_idx.len() == word.len()
        //      word.last() == '\0'
        debug_assert!(
            offset_dict.len() > *word.as_bytes().iter().max().unwrap() as usize
                && offset_dict.len() >= word.as_bytes().len()
                && tmp_end_s.len() == offset_dict.len()
                && sa.len() == word.as_bytes().len()
                && sa_init.len() * 8 >= sa.len()
                && *word.as_bytes().last().unwrap() == 0
        );
        unsafe {
            build_suffix_array::suffix_array(
                word.as_bytes(),
                &mut offset_dict,
                &mut tmp_end_s,
                &mut sa,
                &mut BitArrMut(&mut sa_init),
            );
        }

        Self { word, sa }
    }

    /// Construct suffix array not recursive. Complexity O(n).
    /// Uses less memory to build than `new_stack` by using bitpcking.
    /// ```
    /// use suff_collections::array::*;
    ///
    /// // let sa = SuffixArray::<u8>::new_stack_compress("word");
    /// // let sa = SuffixArray::<u16>::new_stack_compress("word");
    /// // let sa = SuffixArray::<u32>::new_stack_compress("word");
    /// let sa = SuffixArray::<usize>::new_stack_compress("word");
    /// let sa = SuffixArray::<usize>::new_stack_compress("word\0");
    /// ```
    /// At the end of the line should hit '\0'.
    /// If there is no '\0' at the end then the line will be copied and added '\0' to the end.
    /// Otherwise, the value will be taken by reference.
    ///
    /// # Panics
    ///
    /// This function will panic if word.len() > T::MAX.
    pub fn new_stack_compress(word: &'sa str) -> Self {
        assert!(word.len() < <T as Max>::max().to_usize());
        if word.is_empty() {
            return Self {
                word: Cow::from(""),
                sa: vec![],
            };
        }
        let word = canonic_word(word);
        let mut offset_dict =
            vec![(T::zero(), T::zero()); max(word.len(), SuffixArray::<T>::DICT_SIZE)];
        let mut tmp_end_s = vec![T::zero(); offset_dict.len()];
        let mut sa = vec![T::zero(); word.len()];
        let mut sa_init = Byte::vec(word.len());
        // safe because
        //      offset_dict.len() > max(s_idx) && offset_dict.len() >= s_idx.len()
        //      tmp_end_s.len() == offset_dict.len()
        //      sa.len() == s_idx.len()
        //      s_idx.len() == word.len()
        //      word.last() == '\0'
        debug_assert!(
            offset_dict.len() > *word.as_bytes().iter().max().unwrap() as usize
                && offset_dict.len() >= word.as_bytes().len()
                && tmp_end_s.len() == offset_dict.len()
                && sa.len() == word.as_bytes().len()
                && sa_init.len() * 8 >= sa.len()
                && *word.as_bytes().last().unwrap() == 0
        );
        unsafe {
            build_suffix_array::suffix_array_stack(
                word.as_bytes(),
                &mut offset_dict,
                &mut tmp_end_s,
                &mut sa,
                &mut BitArrMut(&mut sa_init),
            );
        }

        Self { word, sa }
    }

    /// Construct suffix array recursive. Complexity O(n)
    /// ```
    /// use suff_collections::array::*;
    ///
    /// // let sa = SuffixArray::<u8>::new("word");
    /// // let sa = SuffixArray::<u16>::new("word");
    /// // let sa = SuffixArray::<u32>::new("word");
    /// let sa = SuffixArray::<usize>::new("word");
    /// let sa = SuffixArray::<usize>::new("word\0");
    /// ```
    /// At the end of the line should hit '\0'.
    /// If there is no '\0' at the end then the line will be copied and added '\0' to the end.
    /// Otherwise, the value will be taken by reference.
    ///
    /// # Panics
    ///
    /// This function will panic if word.len() > T::MAX.
    pub fn new(word: &'sa str) -> Self {
        assert!(word.len() < <T as Max>::max().to_usize());
        if word.is_empty() {
            return Self {
                word: Cow::from(""),
                sa: vec![],
            };
        }
        let word = canonic_word(word);
        let mut offset_dict =
            vec![(T::zero(), T::zero()); max(word.len(), SuffixArray::<T>::DICT_SIZE)];
        let mut tmp_end_s = vec![T::zero(); offset_dict.len()];
        let mut sa = vec![T::zero(); word.len()];
        let mut sa_init = vec![Byte::default(); word.len()];
        // safe because
        //      offset_dict.len() > max(s_idx) && offset_dict.len() >= s_idx.len()
        //      tmp_end_s.len() == offset_dict.len()
        //      sa.len() == s_idx.len()
        //      sa_init.len() == sa.len()
        //      s_idx.len() == word.len()
        //      word.last() == '\0'
        debug_assert!(
            offset_dict.len() > *word.as_bytes().iter().max().unwrap() as usize
                && offset_dict.len() >= word.as_bytes().len()
                && tmp_end_s.len() == offset_dict.len()
                && sa.len() == word.as_bytes().len()
                && sa_init.len() == sa.len()
                && *word.as_bytes().last().unwrap() == 0
        );
        unsafe {
            build_suffix_array::suffix_array(
                word.as_bytes(),
                &mut offset_dict,
                &mut tmp_end_s,
                &mut sa,
                &mut ByteSliceMut(&mut sa_init),
            );
        }

        Self { word, sa }
    }

    /// Construct suffix array not recursive. Complexity O(n)
    /// ```
    /// use suff_collections::array::*;
    ///
    /// // let sa = SuffixArray::<u8>::new_stack("word");
    /// // let sa = SuffixArray::<u16>::new_stack("word");
    /// // let sa = SuffixArray::<u32>::new_stack("word");
    /// let sa = SuffixArray::<usize>::new_stack("word");
    /// let sa = SuffixArray::<usize>::new_stack("word\0");
    /// ```
    /// At the end of the line should hit '\0'.
    /// If there is no '\0' at the end then the line will be copied and added '\0' to the end.
    /// Otherwise, the value will be taken by reference.
    ///
    /// # Panics
    ///
    /// This function will panic if word.len() > T::MAX.
    pub fn new_stack(word: &'sa str) -> Self {
        assert!(word.len() < <T as Max>::max().to_usize());
        if word.is_empty() {
            return Self {
                word: Cow::from(""),
                sa: vec![],
            };
        }
        let word = canonic_word(word);
        let mut offset_dict =
            vec![(T::zero(), T::zero()); max(word.len(), SuffixArray::<T>::DICT_SIZE)];
        let mut tmp_end_s = vec![T::zero(); offset_dict.len()];
        let mut sa = vec![T::zero(); word.len()];
        let mut sa_init = vec![Byte::default(); word.len()];
        // safe because
        //      offset_dict.len() > max(s_idx) && offset_dict.len() >= s_idx.len()
        //      tmp_end_s.len() == offset_dict.len()
        //      sa.len() == s_idx.len()
        //      sa_init.len() == sa.len()
        //      s_idx.len() == word.len()
        //      word.last() == '\0'
        debug_assert!(
            offset_dict.len() > *word.as_bytes().iter().max().unwrap() as usize
                && offset_dict.len() >= word.as_bytes().len()
                && tmp_end_s.len() == offset_dict.len()
                && sa.len() == word.as_bytes().len()
                && sa_init.len() == sa.len()
                && *word.as_bytes().last().unwrap() == 0
        );
        unsafe {
            build_suffix_array::suffix_array_stack(
                word.as_bytes(),
                &mut offset_dict,
                &mut tmp_end_s,
                &mut sa,
                &mut ByteSliceMut(&mut sa_init),
            );
        }

        Self { word, sa }
    }

    /// Return iterator on suffix array
    /// ```
    /// use suff_collections::array::*;
    ///
    /// SuffixArray::<usize>::new("word\0").iter().for_each(|&idx| println!("idx: {}", idx));
    /// ```
    #[inline]
    pub fn iter(&self) -> Iter<'_, T> {
        self.sa.iter()
    }

    /// Split suffix array
    /// ```
    /// use suff_collections::array::*;
    ///
    /// let (word, sa) = SuffixArray::<usize>::new("word").split_owned();
    /// ```
    #[inline]
    pub fn split_owned(self) -> (Cow<'sa, str>, Vec<T>) {
        (self.word, self.sa)
    }

    /// Return ref on suffix array
    /// ```
    /// use suff_collections::array::*;
    ///
    /// let sa = SuffixArray::<usize>::new("word");
    /// let sa: &[usize] = sa.suffix_array();
    /// ```
    #[inline]
    pub fn suffix_array(&self) -> &Vec<T> {
        &self.sa
    }

    /// Return ref on word
    /// ```
    /// use suff_collections::array::*;
    ///
    /// let sa = SuffixArray::<usize>::new("word");
    /// let word: &str = sa.word();
    /// assert_eq!("word\0", word);
    /// ```
    #[inline]
    pub fn word(&self) -> &str {
        &self.word
    }

    /// lcp\[i\] = max_pref(sa\[i\], sa\[i - 1\]) && lcp.len() == sa.len()
    /// Construct LCP. Complexity O(n)
    /// ```
    /// use suff_collections::array::*;
    ///
    /// let lcp = SuffixArray::<usize>::new("word").lcp();
    /// ```
    pub fn lcp(&self) -> LCP<T> {
        let mut lcp = LCP::<T>::new(vec![T::zero(); self.sa.len()]);
        let mut sa_idx = vec![T::zero(); self.sa.len()];

        // safe max(sa) < sa_idx.len()
        self.sa.iter().enumerate().for_each(|(i, &x)| unsafe {
            *sa_idx.get_unchecked_mut(x.to_usize()) = T::try_from(i + 1).ok().unwrap()
        });

        let mut pref_len = T::zero();
        let word = self.word.as_bytes();
        for x in sa_idx {
            if x.to_usize() == self.sa.len() {
                pref_len = T::zero();
                continue;
            }

            // safe max(sa_idx) < sa.len() && x < sa.len() by previous check
            // safe l < word.len() && r < word.len()
            let l = unsafe { *self.sa.get_unchecked(x.to_usize() - 1) };
            let r = unsafe { *self.sa.get_unchecked(x.to_usize()) };
            pref_len = unsafe {
                count_eq(
                    word.get_unchecked(l.to_usize()..),
                    word.get_unchecked(r.to_usize()..),
                    pref_len,
                )
            };

            // safe x < sa.len() by previous check && lcp.len() == sa.len()
            unsafe {
                *lcp.idx_mut(x.to_usize()) = pref_len;
            }
            if pref_len > T::zero() {
                pref_len -= T::one();
            }
        }
        lcp
    }

    /// Find substr. Complexity O(|find| * log(|word|))
    /// ```
    /// use suff_collections::array::*;
    ///
    /// // let find: Option<u8> = SuffixArray::<u8>::new("word").find("or");
    /// // let find: Option<u16> = SuffixArray::<u16>::new("word").find("or");
    /// // let find: Option<u32> = SuffixArray::<u32>::new("word").find("or");
    /// let find: Option<usize> = SuffixArray::<usize>::new("word").find("or");
    /// assert_eq!(find, Some(1));
    /// ```
    #[inline]
    pub fn find(&self, find: &str) -> Option<T> {
        let (start, end) = self.find_pos(find);
        if start >= end {
            return None;
        }
        Some(self.sa[start])
    }

    /// Find all substr. Complexity O(|find| * log(|word|))
    /// ```
    /// use suff_collections::array::*;
    ///
    /// let sa = SuffixArray::<usize>::new("word");
    /// let find: &[usize] = sa.find_all("or");
    /// assert_eq!(find, &[1]);
    /// ```
    #[inline]
    pub fn find_all(&self, find: &str) -> &[T] {
        let (start, end) = self.find_pos(find);
        &self.sa[start..end]
    }

    /// Find substr. Complexity O(|word|)
    /// ```
    /// use suff_collections::array::*;
    ///
    /// let sa = SuffixArray::<usize>::new("word");
    /// let find: Option<usize> = sa.find_big(&sa.lcp(), "or");
    /// assert_eq!(find, Some(1));
    /// ```
    #[inline]
    pub fn find_big(&self, lcp: &LCP<T>, find: &str) -> Option<T> {
        let idx = self.find_pos_big(lcp, find)?;
        Some(self.sa[idx])
    }

    /// Find all substr. Complexity O(|word|)
    /// ```
    /// use suff_collections::array::*;
    ///
    /// let sa = SuffixArray::<usize>::new("word");
    /// let find: &[usize] = sa.find_all_big(&sa.lcp(), "or");
    /// assert_eq!(find, &[1]);
    /// ```
    #[inline]
    pub fn find_all_big(&self, lcp: &LCP<T>, find: &str) -> &[T] {
        match self.find_pos_big(lcp, find) {
            None => &[],
            Some(start) => {
                let end = start
                    + lcp
                        .iter()
                        .skip(start + 1)
                        .take_while(|&&lcp| find.len() <= lcp.to_usize())
                        .count();

                &self.sa[start..=end]
            }
        }
    }

    // O(|find| * log(|word|))
    fn find_pos(&self, find: &str) -> (usize, usize) {
        if find.is_empty() {
            return (0, 0);
        }
        let (word, find) = (self.word.as_bytes(), find.as_bytes());
        let start = binary_search(&self.sa, |&idx| &word[idx.to_usize()..] < find);

        // skip all matches
        let end = start
            + binary_search(&self.sa[start..], |&idx| {
                idx.to_usize() + find.len() < word.len()
                    && &word[idx.to_usize()..idx.to_usize() + find.len()] == find
            });

        (start, end)
    }
    // O(|word|)
    fn find_pos_big(&self, lcp: &LCP<T>, find: &str) -> Option<usize> {
        if find.is_empty() {
            return None;
        }
        let (word, find) = (self.word.as_bytes(), find.as_bytes());
        // entry of the first character (byte) is searched for by means of binary search
        let start = binary_search(&self.sa, |&idx| word[idx.to_usize()] < find[0]);

        let mut total_eq = 0;
        for (&idx, i) in self.sa.iter().skip(start).zip((start + 1..).into_iter()) {
            total_eq = count_eq(&word[idx.to_usize()..], &find, total_eq);

            if total_eq == find.len() {
                return Some(i - 1);
            }

            if i < lcp.len() && total_eq > lcp[i].to_usize() {
                return None;
            }
        }
        None
    }
}

impl<T: SuffixIndices<T>> From<SuffixTree<'_>> for SuffixArray<'_, T> {
    /// Construct suffix array from suffix tree not recursive. Complexity O(n)
    /// ```
    /// use suff_collections::{array::*, tree::*};
    ///
    /// let st = SuffixTree::new("word\0");
    /// // let sa = SuffixArray::<u8>::from(st);
    /// // let sa = SuffixArray::<u16>::from(st);
    /// // let sa = SuffixArray::<u32>::from(st);
    /// let sa = SuffixArray::<usize>::from(st);
    /// ```
    fn from(tree: SuffixTree) -> Self {
        let word = if tree.word().as_bytes().last() == Some(&0) {
            Cow::from(tree.word().to_owned())
        } else {
            Cow::from(
                str::from_utf8(
                    &tree
                        .word()
                        .as_bytes()
                        .iter()
                        .chain(&[0])
                        .copied()
                        .collect::<Vec<_>>(),
                )
                .unwrap()
                .to_owned(),
            )
        };

        let mut sa = Vec::with_capacity(word.len());
        let mut stack = Vec::with_capacity(word.len());

        stack.push(ChildrenIterator {
            it: tree.root_node().children().iter(),
            len: 0,
        });

        while let Some(x) = stack.last_mut() {
            match x.it.next() {
                None => {
                    stack.pop();
                }
                Some((_, &i)) => {
                    let node = tree.node(i);
                    if node.children().is_empty() {
                        sa.push(T::try_from(node.pos() - x.len).ok().unwrap());
                    } else {
                        let len = x.len + node.len();
                        stack.push(ChildrenIterator {
                            it: node.children().iter(),
                            len: len,
                        });
                    }
                }
            }
        }

        return Self { word: word, sa: sa };

        struct ChildrenIterator<I>
        where
            I: Iterator,
        {
            it: I,
            len: usize,
        }
    }
}

fn binary_search<T>(x: &[T], cmp: impl Fn(&T) -> bool) -> usize {
    let mut start = 0;
    let mut cnt = x.len();
    while cnt > 0 {
        let mid = start + cnt / 2;
        let this = &x[mid];
        if cmp(this) {
            start = mid + 1;
            cnt -= cnt / 2 + 1;
        } else {
            cnt /= 2;
        }
    }
    start
}

fn count_eq<T: Eq, P: SuffixIndices<P>>(cmp1: &[T], cmp2: &[T], mut acc: P) -> P {
    while acc.to_usize() < cmp1.len()
        && acc.to_usize() < cmp2.len()
        && cmp1[acc.to_usize()] == cmp2[acc.to_usize()]
    {
        acc += P::one();
    }
    acc
}

// The algorithm of building a suffix array. It uses unsafe blocks to disable
// border check when accessing an array. But if the input data conditions are met,
// the construction is completely safe.
// safe if:
//      offset_dict.len() > max(s_idx) && offset_dict.len() >= s_idx.len()
//      tmp_end_s.len() >= offset_dict.len()
//      sa.len() >= s_idx.len()
//      sa.len() >= s_init.len()
//      s_idx.last() == 0
pub(crate) mod build_suffix_array {
    #[repr(u8)]
    #[derive(Debug, Copy, Clone, PartialOrd, PartialEq)]
    pub(crate) enum TSuff {
        S = 0,
        L = 1,
    }

    pub trait ToUsize {
        fn to_usize(self) -> usize;
    }
    macro_rules! impl_ToUsize {
        ($($tp:ident),* $(,)?) => {
            $(
                impl ToUsize for $tp {
                    #[inline]
                    fn to_usize(self) -> usize {
                        self as usize
                    }
                }
            )*
        };
    }
    pub trait One {
        fn one() -> Self;
    }
    macro_rules! impl_One {
        ($($tp:ident),* $(,)?) => {
            $(
                impl One for $tp {
                    #[inline]
                    fn one() -> $tp {
                        1 as $tp
                    }
                }
            )*
        };
    }
    pub trait Zero {
        fn zero() -> Self;
    }
    macro_rules! impl_Zero {
        ($($tp:ident),* $(,)?) => {
            $(
                impl Zero for $tp {
                    #[inline]
                    fn zero() -> $tp {
                        0 as $tp
                    }
                }
            )*
        };
    }
    pub trait Max {
        fn max() -> Self;
    }
    macro_rules! impl_Max {
        ($($tp:ident),* $(,)?) => {
            $(
                impl Max for $tp {
                    #[inline]
                    fn max() -> $tp {
                        $tp::MAX
                    }
                }
            )*
        };
    }
    use super::*;
    use core::convert::TryFrom;
    use core::{
        debug_assert,
        ops::{Add, AddAssign, Sub, SubAssign},
    };

    pub trait SuffixIndices<T>:
        AddAssign
        + Add<Output = T>
        + SubAssign
        + Sub<Output = T>
        + ToUsize
        + Zero
        + One
        + Max
        + TryFrom<usize>
        + Ord
        + Eq
        + Clone
        + Copy
        + Default
    {
    }

    macro_rules! impl_SuffixIndices {
        ($($tp:ident),* $(,)?) => {
            $(
                impl SuffixIndices<$tp> for $tp {}
            )*
        };
    }
    impl_ToUsize!(u8, u16, u32, u64, usize);
    impl_One!(u8, u16, u32, u64, usize);
    impl_Zero!(u8, u16, u32, u64, usize);
    impl_Max!(u8, u16, u32, u64, usize);
    impl_SuffixIndices!(u8, u16, u32, u64, usize);

    pub(crate) trait Layout: BitMut {
        type NoMut: Bit;

        fn ret_bytes(len: usize) -> Vec<Byte>;
        fn to_bits(bytes: &mut [Byte]) -> Self;
        fn to_bits_no_mut(bytes: &[Byte]) -> Self::NoMut;
        fn calc_lms<Scalar: SuffixIndices<Scalar>>(t: &impl Bit, len: usize) -> Vec<Scalar>;
    }

    impl<'t> Layout for ByteSliceMut<'t> {
        type NoMut = ByteSlice<'t>;
        #[inline]
        fn ret_bytes(len: usize) -> Vec<Byte> {
            vec![Byte::default(); len]
        }
        #[inline]
        fn to_bits(bytes: &mut [Byte]) -> Self {
            // safe cast because Rust can't deduce that we won't return multiple references to the same value
            ByteSliceMut(unsafe { &mut *(bytes as *mut _) })
        }
        #[inline]
        fn to_bits_no_mut(bytes: &[Byte]) -> Self::NoMut {
            // safe cast because Rust can't deduce that we won't return multiple references to the same value
            ByteSlice(unsafe { &*(bytes as *const _) })
        }
        #[inline]
        fn calc_lms<Scalar: SuffixIndices<Scalar>>(t: &impl Bit, _: usize) -> Vec<Scalar> {
            // safe because in the case of ByteSlice Byte stores the type TSuff completely
            unsafe { t.raw() }
                .windows(2)
                .enumerate()
                .filter(|&(_, x)| x[0].0 == TSuff::L as u8 && x[1].0 == TSuff::S as u8)
                .map(|(i, _)| Scalar::try_from(i + 1).ok().unwrap())
                .collect::<Vec<_>>()
        }
    }

    impl<'t> Layout for BitArrMut<'t> {
        type NoMut = BitArr<'t>;
        #[inline]
        fn ret_bytes(len: usize) -> Vec<Byte> {
            Byte::vec(len)
        }
        #[inline]
        fn to_bits(bytes: &mut [Byte]) -> Self {
            // safe cast because Rust can't deduce that we won't return multiple references to the same value
            BitArrMut(unsafe { &mut *(bytes as *mut _) })
        }
        #[inline]
        fn to_bits_no_mut(bytes: &[Byte]) -> Self::NoMut {
            // safe cast because Rust can't deduce that we won't return multiple references to the same value
            BitArr(unsafe { &*(bytes as *const _) })
        }
        #[inline]
        fn calc_lms<Scalar: SuffixIndices<Scalar>>(t: &impl Bit, len: usize) -> Vec<Scalar> {
            // like this but bit unfolding
            // (0..len - 1)
            //     .into_iter()
            //     .filter(|&x| unsafe {
            //         t.get_unchecked(x) == TSuff::L as u8 && t.get_unchecked(x + 1) == TSuff::S as u8
            //     })
            //     .map(|x| Scalar::try_from(x + 1).ok().unwrap())
            //     .collect::<Vec<_>>()

            let mut res = Vec::new();
            let mut i = 0;
            // safe because the next bits are unfolding
            for t in unsafe { t.raw() }.windows(2) {
                for bit_idx in 0..7 {
                    if unsafe {
                        t[0].get_unchecked(bit_idx) == TSuff::L as u8
                            && t[0].get_unchecked(bit_idx + 1) == TSuff::S as u8
                    } {
                        res.push(Scalar::try_from(i + 1).ok().unwrap());
                    }
                    i += 1;
                }
                if unsafe {
                    t[0].get_unchecked(7) == TSuff::L as u8
                        && t[1].get_unchecked(0) == TSuff::S as u8
                } {
                    res.push(Scalar::try_from(i + 1).ok().unwrap());
                }
                i += 1;
            }
            (i..len - 1).into_iter().for_each(|x| {
                if unsafe {
                    t.get_unchecked(x) == TSuff::L as u8 && t.get_unchecked(x + 1) == TSuff::S as u8
                } {
                    res.push(Scalar::try_from(x + 1).ok().unwrap());
                }
            });
            res
        }
    }

    const LEN_NAIVE_SORT: usize = 50;
    #[inline]
    unsafe fn naive_sort<T: ToUsize + Ord + Copy, P: Ord>(sort: &mut [T], s_idx: &[P]) {
        // safe if max(sort) < s_idx.len()
        debug_assert!(sort.iter().max().unwrap().to_usize() < s_idx.len());
        sort.sort_by(|&a, &b| {
            s_idx
                .get_unchecked(a.to_usize()..)
                .cmp(&s_idx.get_unchecked(b.to_usize()..))
        });
    }
    #[inline]
    unsafe fn create_bucket_dict<T: ToUsize + Copy, Scalar: SuffixIndices<Scalar>>(
        s_idx: &[T],
        offset_dict: &mut [(Scalar, Scalar)],
    ) {
        // safe because max(s_idx) < offset_dict.len()
        s_idx
            .iter()
            .for_each(|&x| offset_dict.get_unchecked_mut(x.to_usize()).0 += Scalar::one());
        offset_dict.iter_mut().fold(Scalar::zero(), |acc, offs| {
            let cnt = offs.0;
            *offs = (acc, acc + cnt);
            acc + cnt
        });
    }
    #[inline]
    unsafe fn add_lms_to_end<T: ToUsize + Copy, Scalar: SuffixIndices<Scalar>>(
        s_idx: &[T],
        sa_lms: &[Scalar],
        offset_dict: &mut [(Scalar, Scalar)],
        tmp_end_s: &mut [Scalar],
        sa: &mut [Scalar],
        sa_init: &mut impl BitMut,
    ) {
        offset_dict
            .iter()
            .zip(tmp_end_s.iter_mut())
            .for_each(|(&(_, x), end_s)| *end_s = x);
        sa_lms.iter().rev().for_each(|&x| {
            // safe x == sa_lms && max(sa_lms) < s_idx_len() && max(s_idx) < tmp_end_s.len() &&
            // max(tmp_end_s) < sa.len() && sa_init.len() == sa.len()
            let ptr_end_s =
                tmp_end_s.get_unchecked_mut(s_idx.get_unchecked(x.to_usize()).to_usize());
            *ptr_end_s -= Scalar::one();
            *sa.get_unchecked_mut(ptr_end_s.to_usize()) = x;
            sa_init.set_unchecked(ptr_end_s.to_usize());
        });
    }
    #[allow(non_snake_case)]
    #[inline]
    unsafe fn add_L_to_start<T: ToUsize + Copy, Scalar: SuffixIndices<Scalar>>(
        s_idx: &[T],
        t: &impl Bit,
        offset_dict: &mut [(Scalar, Scalar)],
        sa: &mut [Scalar],
        sa_init: &mut impl BitMut,
    ) {
        for x in 0..sa.len() {
            // safe x < sa.len() && sa_init.len() == sa.len()
            let mut idx = *sa.get_unchecked(x);
            if idx > Scalar::zero() && sa_init.get_unchecked(x) == 1 {
                idx -= Scalar::one();
                // safe idx == x && 0 < x < sa.len() && t.len() == s_idx.len() &&
                // max(s_idx) < offset_dict.len() && max(offset_dict) < sa.len()
                if t.get_unchecked(idx.to_usize()) == TSuff::L as u8 {
                    let ptr_offset = offset_dict
                        .get_unchecked_mut(s_idx.get_unchecked(idx.to_usize()).to_usize());
                    *sa.get_unchecked_mut(ptr_offset.0.to_usize()) = idx;
                    sa_init.set_unchecked(ptr_offset.0.to_usize());
                    ptr_offset.0 += Scalar::one();
                }
            }
        }
    }
    #[allow(non_snake_case)]
    #[inline]
    unsafe fn add_S_to_end<T: ToUsize + Copy, Scalar: SuffixIndices<Scalar>>(
        s_idx: &[T],
        t: &impl Bit,
        offset_dict: &mut [(Scalar, Scalar)],
        sa: &mut [Scalar],
        sa_init: &mut impl BitMut,
    ) {
        for x in (0..sa.len()).rev() {
            // safe x < sa.len() && sa_init.len() == sa.len()
            let mut idx = *sa.get_unchecked(x);
            if idx > Scalar::zero() && sa_init.get_unchecked(x) == 1 {
                idx -= Scalar::one();
                // safe idx == x && 0 < x < sa.len() && t.len() == s_idx.len() &&
                // max(s_idx) < offset_dict.len() && max(offset_dict) < sa.len()
                if t.get_unchecked(idx.to_usize()) == TSuff::S as u8 {
                    let ptr_offset = offset_dict
                        .get_unchecked_mut(s_idx.get_unchecked(idx.to_usize()).to_usize());
                    ptr_offset.1 -= Scalar::one();
                    *sa.get_unchecked_mut(ptr_offset.1.to_usize()) = idx;
                    sa_init.set_unchecked(ptr_offset.1.to_usize());
                }
            }
        }
    }
    #[inline]
    unsafe fn induced_sort<T: ToUsize + Copy, Scalar: SuffixIndices<Scalar>>(
        s_idx: &[T],
        sa_lms: &[Scalar],
        t: &impl Bit,
        offset_dict: &mut [(Scalar, Scalar)],
        tmp_end_s: &mut [Scalar],
        sa: &mut [Scalar],
        sa_init: &mut impl BitMut,
    ) {
        create_bucket_dict(s_idx, offset_dict);
        add_lms_to_end(s_idx, sa_lms, offset_dict, tmp_end_s, sa, sa_init);
        add_L_to_start(s_idx, t, offset_dict, sa, sa_init);
        add_S_to_end(s_idx, t, offset_dict, sa, sa_init);
    }
    #[inline]
    unsafe fn calc_type<T: Ord>(s_idx: &[T], t: &mut impl BitMut) {
        s_idx
            .windows(2)
            .rev()
            .zip((0..s_idx.len() - 1).into_iter().rev())
            .for_each(|(s_idx, i)| {
                if s_idx[0] > s_idx[1]
                    || (s_idx[0] == s_idx[1] && t.get_unchecked(i + 1) == TSuff::L as u8)
                {
                    t.set_unchecked(i);
                }
            });
    }
    #[inline]
    unsafe fn sublms_is_eq<T: Eq>(s_idx: &[T], t: &impl Bit, x: usize, prev: usize) -> bool {
        for i in 0.. {
            // safe because sublms is TSuff::S ... TSuff::S (<= ... > <=) &&
            // we have sentinel symbol in end => \0 - always TSuff::S (> <=) &&
            // 0 <= x < s_idx.len() && s_idx.len() == t.len() &&
            // compare only 2 sublms not all word
            if *s_idx.get_unchecked(x + i) != *s_idx.get_unchecked(prev + i)
                || t.get_unchecked(x + i) != t.get_unchecked(prev + i)
            {
                return false;
            }
            if i != 0
                && ((t.get_unchecked(x + i - 1) == TSuff::L as u8
                    && t.get_unchecked(x + i) == TSuff::S as u8)
                    || (t.get_unchecked(prev + i - 1) == TSuff::L as u8
                        && t.get_unchecked(prev + i) == TSuff::S as u8))
            {
                return true;
            }
        }
        false
    }
    #[inline]
    unsafe fn create_new_str<T: Ord, Scalar: SuffixIndices<Scalar>, BitLayout: Layout>(
        s_idx: &[T],
        alphabet: &mut [Scalar],
        sort_sublms: &[Scalar],
        t: &BitLayout,
        idx_lms: &[Scalar],
    ) -> Vec<Scalar> {
        let mut prev = Scalar::try_from(s_idx.len()).ok().unwrap() - Scalar::one();
        // safe prev < alphabet.len() && s_idx.len() <= alphabet.len()
        *alphabet.get_unchecked_mut(prev.to_usize()) = Scalar::zero();
        let sorted_lms = sort_sublms.iter().skip(1).filter(|&&x|
            // safe 0 < x < sort_sublms.len() && max(sa) < x.len() (sa contains sort sumlms => max(sa) == max(idx_lms))
            x > Scalar::zero()
            && t.get_unchecked(x.to_usize()) == TSuff::S as u8
            && t.get_unchecked((x - Scalar::one()).to_usize()) == TSuff::L as u8);
        for &x in sorted_lms {
            // safe x < alphabet.len() && sorted_lms == sa &&
            // sort_sublms.len() == s_idx.len() && s_idx.len() <= alphabet.len()
            // range(prev) == range(x)
            if sublms_is_eq(&s_idx, t, x.to_usize(), prev.to_usize()) {
                *alphabet.get_unchecked_mut(x.to_usize()) = *alphabet.get_unchecked(prev.to_usize())
            } else {
                *alphabet.get_unchecked_mut(x.to_usize()) =
                    *alphabet.get_unchecked(prev.to_usize()) + Scalar::one()
            }
            prev = x;
        }
        // safe max(idx_lms) < s_idx.len() && s_idx.len() <= alphabet.len()
        idx_lms
            .iter()
            .map(|&x| *alphabet.get_unchecked(x.to_usize()))
            .collect()
    }
    #[inline]
    unsafe fn pack_lms<T: ToUsize + Copy, Scalar: SuffixIndices<Scalar>>(
        idx_lms: &[Scalar],
        s_idx: &[T],
        offset_dict: &mut [(Scalar, Scalar)],
    ) {
        idx_lms.iter().enumerate().for_each(|(i, &x)| {
            // safe max(idx_lms) < s_idx.len() &&
            // max(s_idx) < offset_dict.len()
            let index = s_idx.get_unchecked(x.to_usize()).to_usize();
            offset_dict.get_unchecked_mut(index).0 += Scalar::one();
            offset_dict.get_unchecked_mut(index).1 = Scalar::try_from(i).ok().unwrap();
        });
    }
    #[inline]
    fn lms_is_unique<Scalar: Ord + One + Copy>(offset_dict: &[(Scalar, Scalar)]) -> bool {
        !offset_dict.iter().any(|&(x, _)| x > Scalar::one())
    }
    #[inline]
    unsafe fn unpack_lms<Scalar: Zero + Eq + ToUsize + Copy>(
        idx_lms: &[Scalar],
        offset_dict: &[(Scalar, Scalar)],
    ) -> Vec<Scalar> {
        // safe max(offset_dict) < idx_lms.len()
        offset_dict
            .iter()
            .filter(|&&(x, _)| x != Scalar::zero())
            .map(|&(_, y)| *idx_lms.get_unchecked(y.to_usize()))
            .collect()
    }
    #[inline]
    unsafe fn sort_lms_in_new_str<
        T: ToUsize + Ord + Copy,
        Scalar: SuffixIndices<Scalar>,
        BitLayout: Layout,
    >(
        new_s_idx: &[T],
        offset_dict: &mut [(Scalar, Scalar)],
        tmp_end_s: &mut [Scalar],
        sa: &mut [Scalar],
        sa_init: &mut BitLayout,
        idx_lms: &[Scalar],
    ) -> Vec<Scalar> {
        let new_size = idx_lms.len();
        // safe new_size == idx_lms.len() && max(idx_lms) < s_idx.len() &&
        // s_idx.len() <= offset_dict.len() && s_idx.len() == tmp_end_s.len() &&
        // s_idx.len() == sa.len() && sa.len() == sa_init.len()
        suffix_array(
            &new_s_idx,
            offset_dict.get_unchecked_mut(..new_size),
            tmp_end_s.get_unchecked_mut(..new_size),
            sa.get_unchecked_mut(..new_size),
            &mut sa_init.range_to_mut(new_size),
        );
        // safe new_size <= sa.len() by previous explanation && max(sa) < idx_lms.len()
        sa.get_unchecked(..new_size)
            .iter()
            .map(|&x| *idx_lms.get_unchecked(x.to_usize()))
            .collect()
    }
    #[inline]
    fn clear<T: Default>(src: &mut [T]) {
        src.iter_mut().for_each(|x| *x = T::default());
    }
    #[inline]
    unsafe fn sort_lms<
        T: ToUsize + Ord + Copy,
        Scalar: SuffixIndices<Scalar>,
        BitLayout: Layout,
    >(
        s_idx: &[T],
        offset_dict: &mut [(Scalar, Scalar)],
        tmp_end_s: &mut [Scalar],
        sa: &mut [Scalar],
        sa_init: &mut BitLayout,
        t: &mut BitLayout,
        idx_lms: &[Scalar],
    ) -> Vec<Scalar> {
        pack_lms(idx_lms, s_idx, offset_dict);
        if idx_lms.len() == 1 {
            // safe we have sentinel '\0'
            vec![*idx_lms.get_unchecked(0)]
        } else if lms_is_unique(offset_dict) {
            unpack_lms(idx_lms, offset_dict)
        } else if idx_lms.len() <= LEN_NAIVE_SORT {
            let mut sort_lms = idx_lms.to_vec();
            // safe because max(idx_lms) < s_idx.len()
            naive_sort(&mut sort_lms, s_idx);
            sort_lms
        } else {
            clear(offset_dict);
            induced_sort(s_idx, idx_lms, t, offset_dict, tmp_end_s, sa, sa_init);
            let new_s_idx = create_new_str(s_idx, tmp_end_s, sa, t, idx_lms);

            sa_init.clear();
            clear(offset_dict);

            sort_lms_in_new_str(&new_s_idx, offset_dict, tmp_end_s, sa, sa_init, idx_lms)
        }
    }
    /// https://www.researchgate.net/profile/Daricks_Wai_Hong_Chan/publication/221577802_Linear_Suffix_Array_Construction_by_Almost_Pure_Induced-Sorting/links/00b495318a21ba484f000000/Linear-Suffix-Array-Construction-by-Almost-Pure-Induced-Sorting.pdf?origin=publication_detail
    // safe if
    //      offset_dict.len() > max(s_idx) && offset_dict.len() >= s_idx.len()
    //      tmp_end_s.len() >= offset_dict.len()
    //      sa.len() >= s_idx.len()
    //      sa.len() >= s_init.len()
    //      s_idx.last() == 0
    pub(crate) unsafe fn suffix_array<
        T: ToUsize + Ord + Copy,
        Scalar: SuffixIndices<Scalar>,
        BitLayout: Layout,
    >(
        s_idx: &[T],
        offset_dict: &mut [(Scalar, Scalar)],
        tmp_end_s: &mut [Scalar],
        sa: &mut [Scalar],
        sa_init: &mut BitLayout,
    ) {
        let mut bytes = BitLayout::ret_bytes(s_idx.len());
        let t = &mut BitLayout::to_bits(&mut bytes);
        calc_type(&s_idx, t);
        // lms => ... L S ... (... > <= ...)
        let idx_lms = BitLayout::calc_lms(t, s_idx.len());
        let sa_lms = sort_lms(s_idx, offset_dict, tmp_end_s, sa, sa_init, t, &idx_lms);

        sa_init.clear();
        clear(offset_dict);

        induced_sort(s_idx, &sa_lms, t, offset_dict, tmp_end_s, sa, sa_init);
    }

    enum TState<Scalar: SuffixIndices<Scalar>> {
        Rec(Vec<Scalar>),
        RecEnd(Vec<Scalar>, Vec<Scalar>, Vec<Byte>),
        End(Vec<Scalar>, Vec<Byte>),
    }

    #[repr(u8)]
    enum NTState {
        RecEnd,
        End,
    }
    // safe if
    //      offset_dict.len() > max(s_idx) && offset_dict.len() >= s_idx.len()
    //      tmp_end_s.len() >= offset_dict.len()
    //      sa.len() >= s_idx.len()
    //      sa.len() >= s_init.len()
    //      s_idx.last() == 0
    pub(crate) unsafe fn suffix_array_stack<
        T: ToUsize + Ord + Copy,
        Scalar: SuffixIndices<Scalar>,
        BitLayout: Layout,
    >(
        s_idx: &[T],
        offset_dict: &mut [(Scalar, Scalar)],
        tmp_end_s: &mut [Scalar],
        sa: &mut [Scalar],
        sa_init: &mut BitLayout,
    ) {
        let mut state_stack = Vec::default();

        let mut slice = BitLayout::ret_bytes(s_idx.len());
        let mut t = BitLayout::to_bits(&mut slice);
        calc_type(&s_idx, &mut t);

        let idx_lms = BitLayout::calc_lms(&t, s_idx.len());
        pack_lms(&idx_lms, &s_idx, offset_dict);

        let (res_lms, end_state) = if idx_lms.len() == 1 {
            // safe we have sentinel => \0
            (vec![*idx_lms.get_unchecked(0)], NTState::End)
        } else if lms_is_unique(offset_dict) {
            (unpack_lms(&idx_lms, offset_dict), NTState::End)
        } else {
            clear(offset_dict);
            induced_sort(&s_idx, &idx_lms, &t, offset_dict, tmp_end_s, sa, sa_init);
            let new_s_idx = create_new_str(&s_idx, tmp_end_s, sa, &t, &idx_lms);

            sa_init.clear();
            clear(offset_dict);

            debug_assert!(idx_lms.len() == new_s_idx.len());
            state_stack.push(TState::Rec(new_s_idx));
            (
                suffix_array_stack_inner(offset_dict, tmp_end_s, sa, sa_init, state_stack),
                NTState::RecEnd,
            )
        };

        match end_state {
            NTState::End => {
                sa_init.clear();
                clear(offset_dict);
                induced_sort(&s_idx, &res_lms, &t, offset_dict, tmp_end_s, sa, sa_init);
            }
            NTState::RecEnd => {
                sa_init.clear();
                clear(offset_dict);
                // safe size < offset_dict.len() && tmp_end_s.len() <= offset_dict.len() && sa.len() == s_idx.len()
                let sa_lms = res_lms
                    .iter()
                    .map(|&x| *idx_lms.get_unchecked(x.to_usize()))
                    .collect::<Vec<_>>();
                induced_sort(&s_idx, &sa_lms, &t, offset_dict, tmp_end_s, sa, sa_init);
            }
        }
    }

    // safe if
    //      offset_dict.len() > max(s_idx) && offset_dict.len() >= s_idx.len()
    //      tmp_end_s.len() >= offset_dict.len()
    //      sa.len() >= s_idx.len()
    //      sa.len() >= s_init.len()
    //      s_idx.last() == 0
    unsafe fn suffix_array_stack_inner<Scalar: SuffixIndices<Scalar>, BitLayout: Layout>(
        offset_dict: &mut [(Scalar, Scalar)],
        tmp_end_s: &mut [Scalar],
        sa: &mut [Scalar],
        sa_init: &mut BitLayout,
        mut state_stack: Vec<TState<Scalar>>,
    ) -> Vec<Scalar> {
        let mut res_lms = Vec::default();
        loop {
            match state_stack.pop() {
                None => return res_lms,
                Some(state) => {
                    match state {
                        TState::Rec(s_idx) => {
                            // safe size < offset_dict.len()
                            let offset_dict = offset_dict.get_unchecked_mut(..s_idx.len());
                            // safe s_idx.len() <= sa.len()
                            let sa = sa.get_unchecked_mut(..s_idx.len());
                            // safe s_idx.len() <= sa_init.len()
                            let sa_init = &mut sa_init.range_to_mut(s_idx.len());

                            let mut slice = BitLayout::ret_bytes(s_idx.len());
                            let mut t = BitLayout::to_bits(&mut slice);
                            calc_type(&s_idx, &mut t);

                            let idx_lms = BitLayout::calc_lms(&t, s_idx.len());
                            pack_lms(&idx_lms, &s_idx, offset_dict);

                            if idx_lms.len() == 1 {
                                // safe we have sentinel => \0
                                res_lms = vec![*idx_lms.get_unchecked(0)];

                                state_stack.push(TState::End(s_idx, slice));
                            } else if lms_is_unique(offset_dict) {
                                res_lms = unpack_lms(&idx_lms, offset_dict);

                                state_stack.push(TState::End(s_idx, slice));
                            } else if idx_lms.len() <= LEN_NAIVE_SORT {
                                let mut sort_lms = idx_lms.clone().to_vec();
                                // safe because max(idx_lms) < s_idx.len()
                                naive_sort(&mut sort_lms, &s_idx);
                                res_lms = sort_lms;
                                state_stack.push(TState::End(s_idx, slice));
                            } else {
                                clear(offset_dict);
                                // safe size < offset_dict.len() && tmp_end_s.len() <= offset_dict.len() && sa.len() == s_idx.len()
                                induced_sort(
                                    &s_idx,
                                    &idx_lms,
                                    &t,
                                    offset_dict,
                                    tmp_end_s.get_unchecked_mut(..s_idx.len()),
                                    sa,
                                    sa_init,
                                );
                                let new_s_idx = create_new_str(&s_idx, tmp_end_s, sa, &t, &idx_lms);

                                sa_init.clear();
                                clear(offset_dict);

                                debug_assert!(idx_lms.len() == new_s_idx.len());
                                state_stack.push(TState::RecEnd(s_idx, idx_lms, slice));
                                state_stack.push(TState::Rec(new_s_idx));
                            }
                        }
                        TState::End(s_idx, t) => {
                            // safe size == idx_lms.len()
                            let offset_dict = offset_dict.get_unchecked_mut(..s_idx.len());
                            let sa = sa.get_unchecked_mut(..s_idx.len());
                            let mut sa_init = sa_init.range_to_mut(s_idx.len());
                            let t = BitLayout::to_bits_no_mut(&t);
                            sa_init.clear();
                            clear(offset_dict);
                            // safe size < offset_dict.len() && tmp_end_s.len() <= offset_dict.len() && sa.len() == s_idx.len()
                            induced_sort(
                                &s_idx,
                                &res_lms,
                                &t,
                                offset_dict,
                                tmp_end_s.get_unchecked_mut(..s_idx.len()),
                                sa,
                                &mut sa_init,
                            );
                            res_lms = Vec::from(sa);
                        }
                        TState::RecEnd(s_idx, idx_lms, t) => {
                            // safe size == idx_lms.len()
                            let offset_dict = offset_dict.get_unchecked_mut(..s_idx.len());
                            let sa = sa.get_unchecked_mut(..s_idx.len());
                            let mut sa_init = sa_init.range_to_mut(s_idx.len());
                            let t = BitLayout::to_bits_no_mut(&t);
                            sa_init.clear();
                            clear(offset_dict);
                            // safe size < offset_dict.len() && tmp_end_s.len() <= offset_dict.len() && sa.len() == s_idx.len()
                            let sa_lms = res_lms
                                .iter()
                                .map(|&x| *idx_lms.get_unchecked(x.to_usize()))
                                .collect::<Vec<_>>();
                            induced_sort(
                                &s_idx,
                                &sa_lms,
                                &t,
                                offset_dict,
                                tmp_end_s.get_unchecked_mut(..s_idx.len()),
                                sa,
                                &mut sa_init,
                            );
                            res_lms = Vec::from(sa);
                        }
                    }
                }
            }
        }
    }
}
