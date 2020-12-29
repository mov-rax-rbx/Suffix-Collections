//! Implementation of the [suffix array](https://www.researchgate.net/profile/Daricks_Wai_Hong_Chan/publication/221577802_Linear_Suffix_Array_Construction_by_Almost_Pure_Induced-Sorting/links/00b495318a21ba484f000000/Linear-Suffix-Array-Construction-by-Almost-Pure-Induced-Sorting.pdf?origin=publication_detail)
//! construction of which is performed in linear time

//! # Examples
//!
//! ```
//!     use suff_collections::{array::*, tree::*, lcp::*};
//!
//!     // let word = "Some word";
//!     let word: &str = "Some word\0";
//!     let find: &str = "word";
//!
//!     // construct suffix array
//!     // let sa = SuffixArray::<usize>::new_stack(word);
//!     // let sa = SuffixArray::<u8>::new(word);
//!     // let sa = SuffixArray::<u16>::new(word);
//!     // let sa = SuffixArray::<u32>::new(word);
//!     let sa = SuffixArray::<usize>::new(word);
//!
//!     // construct lcp
//!     // lcp[i] = max_pref(sa[i], sa[i - 1]) && lcp.len() == sa.len()
//!     let lcp: LCP<usize> = sa.lcp();
//!
//!     // finds the entry position of the line 'find' in 'word'
//!     // O(|find| * log(|word|))
//!     let res: Option<usize> = sa.find(find);
//!
//!     // finds all the entry position of the line 'find' in 'word'
//!     // O(|find| * log(|word|))
//!     let res_all: &[usize] = sa.find_all(find);
//!
//!     // finds the entry position of the line 'find' in 'word'
//!     // O(|word|)
//!     let res: Option<usize> = sa.find_big(&sa.lcp(), find);
//!
//!     // finds all the entry position of the line 'find' in 'word'
//!     // O(|word|)
//!     let res_all: &[usize] = sa.find_all_big(&sa.lcp(), find);
//!
//!     // convert suffix array to suffix tree
//!     let st = SuffixTree::from(sa);
//! ```

use alloc::vec::{Vec, IntoIter};
use alloc::borrow::{Cow, ToOwned};
use core::{str, slice::Iter, option::Option, cmp::{max, Eq}};

use crate::{bit::*, tree::*, lcp::*, canonic_word};
use build_suffix_array::SaType;

#[repr(transparent)]
struct BitSlice<'t>(&'t mut [bool]);

impl<'t> Bit for BitSlice<'t> {
    #[inline]
    unsafe fn set_unchecked(&mut self, idx: usize) {
        *self.0.get_unchecked_mut(idx) = true;
    }
    #[inline]
    unsafe fn get_unchecked(&self, idx: usize) -> bool {
        *self.0.get_unchecked(idx)
    }
    #[inline]
    unsafe fn range_to_mut(&mut self, to: usize) -> Self {
        // safe cast because Rust can't deduce that we won't return multiple references to the same value
        Self(&mut *(self.0.get_unchecked_mut(..to) as *mut _))
    }
    #[inline]
    fn clear(&mut self) {
        self.0.iter_mut().for_each(|x| *x = Default::default());
    }
}

#[derive(Debug, Clone)]
pub struct SuffixArray<'sa, T: SaType<T>> {
    word: Cow<'sa, str>,
    sa: Vec<T>
}

impl<'sa, T: SaType<T>> IntoIterator for SuffixArray<'sa, T> {
    type Item = T;
    type IntoIter = IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.sa.into_iter()
    }
}

impl<'sa, T: SaType<T>> IntoIterator for &'sa SuffixArray<'sa, T> {
    type Item = &'sa T;
    type IntoIter = Iter<'sa, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.sa.iter()
    }
}

impl<'sa, T: SaType<T>> SuffixArray<'sa, T> {
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
    /// If word.len() > Type::MAX then panic.
    pub fn new_compress(word: &'sa str) -> Self {
        assert!(word.len() < build_suffix_array::Max::max());
        if word.is_empty() {
            return Self {
                word: Cow::from(""),
                sa: vec![],
            };
        }
        let word = canonic_word(word);
        let mut offset_dict = vec![
            (T::zero(), T::zero());
            max(word.len(), SuffixArray::<T>::DICT_SIZE)
        ];
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
            offset_dict.len() > *word.as_bytes().iter().max().unwrap() as usize &&
            offset_dict.len() >= word.as_bytes().len() &&
            tmp_end_s.len() == offset_dict.len() &&
            sa.len() == word.as_bytes().len() &&
            sa_init.len() * 8 >= sa.len() &&
            *word.as_bytes().last().unwrap() == 0
        );
        unsafe {
            build_suffix_array::suffix_array(
                word.as_bytes(),
                &mut offset_dict,
                &mut tmp_end_s,
                &mut sa,
                &mut BitArrMut(&mut sa_init)
            );
        }

        Self {
            word: word,
            sa: sa,
        }
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
    /// If word.len() > Type::MAX then panic
    pub fn new_stack_compress(word: &'sa str) -> Self {
        assert!(word.len() < build_suffix_array::Max::max());
        if word.is_empty() {
            return Self {
                word: Cow::from(""),
                sa: vec![],
            };
        }
        let word = canonic_word(word);
        let mut offset_dict = vec![
            (T::zero(), T::zero());
            max(word.len(), SuffixArray::<T>::DICT_SIZE)
        ];
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
            offset_dict.len() > *word.as_bytes().iter().max().unwrap() as usize &&
            offset_dict.len() >= word.as_bytes().len() &&
            tmp_end_s.len() == offset_dict.len() &&
            sa.len() == word.as_bytes().len() &&
            sa_init.len() * 8 >= sa.len() &&
            *word.as_bytes().last().unwrap() == 0
        );
        unsafe {
            build_suffix_array::suffix_array_stack(
                word.as_bytes(),
                &mut offset_dict,
                &mut tmp_end_s,
                &mut sa,
                &mut BitArrMut(&mut sa_init)
            );
        }

        Self {
            word: word,
            sa: sa,
        }
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
    /// If word.len() > Type::MAX then panic.
    pub fn new(word: &'sa str) -> Self {
        assert!(word.len() < build_suffix_array::Max::max());
        if word.is_empty() {
            return Self {
                word: Cow::from(""),
                sa: vec![],
            };
        }
        let word = canonic_word(word);
        let mut offset_dict = vec![
            (T::zero(), T::zero());
            max(word.len(), SuffixArray::<T>::DICT_SIZE)
        ];
        let mut tmp_end_s = vec![T::zero(); offset_dict.len()];
        let mut sa = vec![T::zero(); word.len()];
        let mut sa_init = vec![bool::default(); word.len()];
        // safe because
        //      offset_dict.len() > max(s_idx) && offset_dict.len() >= s_idx.len()
        //      tmp_end_s.len() == offset_dict.len()
        //      sa.len() == s_idx.len()
        //      sa_init.len() == sa.len()
        //      s_idx.len() == word.len()
        //      word.last() == '\0'
        debug_assert!(
            offset_dict.len() > *word.as_bytes().iter().max().unwrap() as usize &&
            offset_dict.len() >= word.as_bytes().len() &&
            tmp_end_s.len() == offset_dict.len() &&
            sa.len() == word.as_bytes().len() &&
            sa_init.len() == sa.len() &&
            *word.as_bytes().last().unwrap() == 0
        );
        unsafe {
            build_suffix_array::suffix_array(
                word.as_bytes(),
                &mut offset_dict,
                &mut tmp_end_s,
                &mut sa,
                &mut BitSlice(sa_init.as_mut_slice())
            );
        }

        Self {
            word: word,
            sa: sa,
        }
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
    /// If word.len() > Type::MAX then panic
    pub fn new_stack(word: &'sa str) -> Self {
        assert!(word.len() < build_suffix_array::Max::max());
        if word.is_empty() {
            return Self {
                word: Cow::from(""),
                sa: vec![],
            };
        }
        let word = canonic_word(word);
        let mut offset_dict = vec![
            (T::zero(), T::zero());
            max(word.len(), SuffixArray::<T>::DICT_SIZE)
        ];
        let mut tmp_end_s = vec![T::zero(); offset_dict.len()];
        let mut sa = vec![T::zero(); word.len()];
        let mut sa_init = vec![bool::default(); word.len()];
        // safe because
        //      offset_dict.len() > max(s_idx) && offset_dict.len() >= s_idx.len()
        //      tmp_end_s.len() == offset_dict.len()
        //      sa.len() == s_idx.len()
        //      sa_init.len() == sa.len()
        //      s_idx.len() == word.len()
        //      word.last() == '\0'
        debug_assert!(
            offset_dict.len() > *word.as_bytes().iter().max().unwrap() as usize &&
            offset_dict.len() >= word.as_bytes().len() &&
            tmp_end_s.len() == offset_dict.len() &&
            sa.len() == word.as_bytes().len() &&
            sa_init.len() == sa.len() &&
            *word.as_bytes().last().unwrap() == 0
        );
        unsafe {
            build_suffix_array::suffix_array_stack(
                word.as_bytes(),
                &mut offset_dict,
                &mut tmp_end_s,
                &mut sa,
                &mut BitSlice(sa_init.as_mut_slice())
            );
        }

        Self {
            word: word,
            sa: sa,
        }
    }

    /// Construct suffix array from suffix tree not recursive. Complexity O(n)
    /// ```
    /// use suff_collections::{array::*, tree::*};
    ///
    /// let st = SuffixTree::new("word\0");
    /// // let sa = SuffixArray::<u8>::from_stack(st);
    /// // let sa = SuffixArray::<u16>::from_stack(st);
    /// // let sa = SuffixArray::<u32>::from_stack(st);
    /// let sa = SuffixArray::<usize>::from_stack(st);
    /// ```
    pub fn from_stack(tree: SuffixTree) -> Self {
        let mut sa = Vec::with_capacity(tree.word().len());
        let mut stack = Vec::with_capacity(tree.word().len());

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

        // correct because sa is correct suffix array && tree.word().last() == '\0'
        return Self {
            word: Cow::from(tree.word().to_owned()),
            sa: sa,
        };

        struct ChildrenIterator<I>
        where
            I: Iterator,
        {
            it: I,
            len: usize,
        }
    }

    /// Recursive construct suffix array from suffix tree. Complexity O(n)
    /// ```
    /// use suff_collections::{array::*, tree::*};
    ///
    /// let st = SuffixTree::new("word\0");
    /// // let sa = SuffixArray::<u8>::from_rec(st);
    /// // let sa = SuffixArray::<u16>::from_rec(st);
    /// // let sa = SuffixArray::<u32>::from_rec(st);
    /// let sa = SuffixArray::<usize>::from_rec(st);
    /// ```
    pub fn from_rec(tree: SuffixTree) -> Self {
        let mut sa = Vec::with_capacity(tree.word().len());
        to_suffix_array_rec_inner(&tree, NodeIdx::root(), 0, &mut sa);
        // correct because sa is correct suffix array && tree.word().last() == '\0'
        Self {
            word: Cow::from(tree.word().to_owned()),
            sa: sa
        }
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
        self.sa.iter().enumerate().for_each(|(i, &x)|
            unsafe { *sa_idx.get_unchecked_mut(x.to_usize()) = T::try_from(i + 1).ok().unwrap() });
        let mut pref_len = T::zero();
        let word = self.word.as_bytes();
        for x in sa_idx {
            if x.to_usize() == self.sa.len() {
                pref_len = T::zero();
                continue;
            }
            // safe max(sa_idx) < sa.len() && x < sa.len() by previous check
            // safe l < word.len() && r < word.len()
            let l = unsafe { *self.sa.get_unchecked((x).to_usize() - 1) };
            let r = unsafe { *self.sa.get_unchecked(x.to_usize()) };
            pref_len = unsafe { count_eq(
                word.get_unchecked(l.to_usize()..),
                word.get_unchecked(r.to_usize()..),
                pref_len
            ) };
            // safe x < sa.len() by previous check && lcp.len() == sa.len()
            unsafe { *lcp.idx_mut(x.to_usize()) = pref_len; }
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
        // safe by previous operation
        Some(unsafe { *self.sa.get_unchecked(start) })
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
        // safe start <= end && 0 <= start < sa.len() && 0 <= end < sa.len()
        unsafe { self.sa.get_unchecked(start..end) }
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
        // safe by previous operation
        Some(unsafe { *self.sa.get_unchecked(idx) })
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
                // safe because 0 <= start < self.sa.len()
                let end = start +
                    lcp.iter().skip(start + 1)
                    .take_while(|&&lcp| find.len() <= lcp.to_usize())
                    .count();
                // safe start <= end && 0 <= start < sa.len() && 0 <= end < sa.len()
                unsafe { self.sa.get_unchecked(start..=end) }
            }
        }
    }

    // O(|find| * log(|word|))
    fn find_pos(&self, find: &str) -> (usize, usize) {
        if find.is_empty() {
            return (0, 0);
        }
        let (word, find) = (self.word.as_bytes(), find.as_bytes());
        // start is searched for by means of binary search
        let start = binary_search(&self.sa, |&idx| {
            // safe because binary search correct =>
            //  0 <= idx < self.sa.len() && max(sa) < word.len()
            unsafe { word.get_unchecked(idx.to_usize()..) < find }
        });
        // skip all matches
        // safe because 0 <= start < self.sa.len()
        let end = start + unsafe {
            binary_search(self.sa.get_unchecked(start..), |&idx| {
                // safe because binary search correct =>
                //  0 <= idx < self.sa.len() && max(sa) < word.len()
                idx.to_usize() + find.len() < word.len()
                && word.get_unchecked(idx.to_usize()..idx.to_usize() + find.len()) == find
            })
        };

        (start, end)
    }
    // O(|word|)
    fn find_pos_big(&self, lcp: &LCP<T>, find: &str) -> Option<usize> {
        if find.is_empty() {
            return None;
        }
        let (word, find) = (self.word.as_bytes(), find.as_bytes());
        // entry of the first character (byte) is searched for by means of binary search
        // safe because binary search correct =>
        //  0 <= idx < self.sa.len() && max(sa) < word.len() && !find.is_empty()
        let start = binary_search(&self.sa,
            |&idx| unsafe { word.get_unchecked(idx.to_usize()) < find.get_unchecked(0) });

        // loop continue while total_eq < find.len() and total_eq never decrement
        let mut total_eq = 0;
        for (&idx, i) in self.sa.iter().skip(start).zip((start + 1..).into_iter()) {
            // safe word_idx < word.len() && total_eq < find.len()
            total_eq = unsafe {
                count_eq(
                    word.get_unchecked(idx.to_usize()..),
                    find.get_unchecked(..),
                    total_eq
                ) };
            if total_eq == find.len() {
                return Some(i - 1);
            }

            // safe i < lcp.len()
            if i < lcp.len() && total_eq > unsafe { lcp.idx(i).to_usize() } {
                return None;
            }
        }
        None
    }
}

fn binary_search<T>(x: &[T], cmp: impl Fn(&T) -> bool) -> usize {
    let mut start = 0;
    let mut cnt =
        if !x.is_empty() {
            x.len() - 1
        } else {
            0
        };
    while cnt > 0 {
        let mid = start + cnt / 2;
        // safe 0 <= mid < x.len()
        let this = unsafe { x.get_unchecked(mid) };
        if cmp(this) {
            start = mid + 1;
            cnt -= cnt / 2 + 1;
        } else {
            cnt /= 2;
        }
    }
    start
}

fn count_eq<T: Eq, P: SaType<P>>(cmp1: &[T], cmp2: &[T], mut acc: P) -> P {
    while acc.to_usize() < cmp1.len() && acc.to_usize() < cmp2.len()
    // safe by previous check
        && unsafe { *cmp1.get_unchecked(acc.to_usize()) == *cmp2.get_unchecked(acc.to_usize()) }
    {
        acc += P::one();
    }
    acc
}

fn to_suffix_array_rec_inner<T: SaType<T>>(tree: &SuffixTree, node_idx: NodeIdx, len: usize, sa: &mut Vec<T>) {
    let node = tree.node(node_idx);
    if node.children().is_empty() {
        sa.push(T::try_from(node.pos() - len).ok().unwrap());
        return;
    }
    for (_, &child) in node.children().iter() {
        to_suffix_array_rec_inner(&tree, child, len + node.len(), sa);
    }
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
        L,
        S
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
    use core::ops::{Add, AddAssign, Sub, SubAssign};
    use core::convert::TryFrom;
    use super::*;

    pub trait SaType<T>:
        AddAssign + Add<Output = T>
        + SubAssign + Sub<Output = T>
        + ToUsize + Zero + One + Max
        + TryFrom<usize> + Ord + Eq
        + Clone + Copy + Default
    {}

    macro_rules! impl_SaType {
        ($($tp:ident),* $(,)?) => {
            $(
                impl SaType<$tp> for $tp {}
            )*
        };
    }
    impl_ToUsize!(u8, u16, u32, u64, usize);
    impl_One!(u8, u16, u32, u64, usize);
    impl_Zero!(u8, u16, u32, u64, usize);
    impl_Max!(u8, u16, u32, u64, usize);
    impl_SaType!(u8, u16, u32, u64, usize);

    const LEN_NAIVE_SORT: usize = 50;
    #[inline]
    unsafe fn naive_sort<T: ToUsize + Ord + Copy, P: Ord>(sort: &mut [T], s_idx: &[P]) {
        // safe if max(sort) < s_idx.len()
        debug_assert!(sort.iter().max().unwrap().to_usize() < s_idx.len());
        sort.sort_by(|&a, &b|
            s_idx.get_unchecked(a.to_usize()..).cmp(&s_idx.get_unchecked(b.to_usize()..)));
    }
    #[inline]
    unsafe fn create_bucket_dict<T: ToUsize + Copy, Scalar: SaType<Scalar>>(
        s_idx: &[T],
        offset_dict: &mut [(Scalar, Scalar)]
    ) {
        // safe because max(s_idx) < offset_dict.len()
        s_idx.iter().for_each(|&x| offset_dict.get_unchecked_mut(x.to_usize()).0 += Scalar::one());
        offset_dict.iter_mut().fold(Scalar::zero(), |acc, offs| {
            let cnt = offs.0;
            *offs = (acc, acc + cnt);
            acc + cnt
        });
    }
    #[inline]
    unsafe fn add_lms_to_end<T: ToUsize + Copy, Scalar: SaType<Scalar>>(
        s_idx: &[T],
        sa_lms: &[Scalar],
        offset_dict: &mut [(Scalar, Scalar)],
        tmp_end_s: &mut [Scalar],
        sa: &mut [Scalar],
        sa_init: &mut impl Bit
    ) {
        offset_dict.iter().zip(tmp_end_s.iter_mut()).for_each(|(&(_, x), end_s)| *end_s = x);
        sa_lms.iter().rev().for_each(|&x| {
            // safe x == sa_lms && max(sa_lms) < s_idx_len() && max(s_idx) < tmp_end_s.len() &&
            // max(tmp_end_s) < sa.len() && sa_init.len() == sa.len()
            let ptr_end_s = tmp_end_s.get_unchecked_mut(s_idx.get_unchecked(x.to_usize()).to_usize());
            *ptr_end_s -= Scalar::one();
            *sa.get_unchecked_mut(ptr_end_s.to_usize()) = x;
            sa_init.set_unchecked(ptr_end_s.to_usize());
        });
    }
    #[allow(non_snake_case)]
    #[inline]
    unsafe fn add_L_to_start<T: ToUsize + Copy, Scalar: SaType<Scalar>>(
        s_idx: &[T],
        t: &[TSuff],
        offset_dict: &mut [(Scalar, Scalar)],
        sa: &mut [Scalar],
        sa_init: &mut impl Bit
    ) {
        for x in 0..sa.len() {
            // safe x < sa.len() && sa_init.len() == sa.len()
            let mut idx = *sa.get_unchecked(x);
            if idx > Scalar::zero() && sa_init.get_unchecked(x) == true {
                idx -= Scalar::one();
                // safe idx == x && 0 < x < sa.len() && t.len() == s_idx.len() &&
                // max(s_idx) < offset_dict.len() && max(offset_dict) < sa.len()
                if *t.get_unchecked(idx.to_usize()) == TSuff::L {
                    let ptr_offset =
                        offset_dict.get_unchecked_mut(s_idx.get_unchecked(idx.to_usize()).to_usize());
                    *sa.get_unchecked_mut(ptr_offset.0.to_usize()) = idx;
                    sa_init.set_unchecked(ptr_offset.0.to_usize());
                    ptr_offset.0 += Scalar::one();
                }
            }
        }
    }
    #[allow(non_snake_case)]
    #[inline]
    unsafe fn add_S_to_end<T: ToUsize + Copy, Scalar: SaType<Scalar>>(
        s_idx: &[T],
        t: &[TSuff],
        offset_dict: &mut [(Scalar, Scalar)],
        sa: &mut [Scalar],
        sa_init: &mut impl Bit
    ) {
        for x in (0..sa.len()).rev() {
            // safe x < sa.len() && sa_init.len() == sa.len()
            let mut idx = *sa.get_unchecked(x);
            if idx > Scalar::zero() && sa_init.get_unchecked(x) == true {
                idx -= Scalar::one();
                // safe idx == x && 0 < x < sa.len() && t.len() == s_idx.len() &&
                // max(s_idx) < offset_dict.len() && max(offset_dict) < sa.len()
                if *t.get_unchecked(idx.to_usize()) == TSuff::S {
                    let ptr_offset =
                        offset_dict.get_unchecked_mut(s_idx.get_unchecked(idx.to_usize()).to_usize());
                    ptr_offset.1 -= Scalar::one();
                    *sa.get_unchecked_mut(ptr_offset.1.to_usize()) = idx;
                    sa_init.set_unchecked(ptr_offset.1.to_usize());
                }
            }
        }
    }
    #[inline]
    unsafe fn induced_sort<T: ToUsize + Copy, Scalar: SaType<Scalar>>(
        s_idx: &[T],
        sa_lms: &[Scalar],
        t: &[TSuff],
        offset_dict: &mut [(Scalar, Scalar)],
        tmp_end_s: &mut [Scalar],
        sa: &mut [Scalar],
        sa_init: &mut impl Bit
    ) {
        create_bucket_dict(s_idx, offset_dict);
        add_lms_to_end(s_idx, sa_lms, offset_dict, tmp_end_s, sa, sa_init);
        add_L_to_start(s_idx, t, offset_dict, sa, sa_init);
        add_S_to_end(s_idx, t, offset_dict, sa, sa_init);
    }
    #[inline]
    unsafe fn calc_type<T: Ord>(s_idx: &[T]) -> Vec<TSuff> {
        let mut t = vec![TSuff::S; s_idx.len()];
        s_idx.windows(2).rev().zip((0..s_idx.len() - 1).into_iter().rev())
            .for_each(|(s_idx, i)| {
                if s_idx[0] > s_idx[1] {
                    *t.get_unchecked_mut(i) = TSuff::L;
                } else if s_idx[0] == s_idx[1] {
                    *t.get_unchecked_mut(i) = *t.get_unchecked(i + 1);
                }
        });
        t
    }
    #[inline]
    fn calc_lms<Scalar: SaType<Scalar>>(t: &[TSuff]) -> Vec<Scalar> {
        t.windows(2).enumerate()
            .filter(|&(_, x)| x[0] == TSuff::L && x[1] == TSuff::S)
            .map(|(i, _)| Scalar::try_from(i + 1).ok().unwrap()).collect::<Vec<_>>()
    }
    #[inline]
    unsafe fn sublms_is_eq<T: Eq>(s_idx: &[T], t: &[TSuff], x: usize, prev: usize) -> bool {
        for i in 0.. {
            // safe because sublms is TSuff::S ... TSuff::S (<= ... > <=) &&
            // we have sentinel symbol in end => \0 - always TSuff::S (> <=) &&
            // 0 <= x < s_idx.len() && s_idx.len() == t.len() &&
            // compare only 2 sublms not all word
            if *s_idx.get_unchecked(x + i) != *s_idx.get_unchecked(prev + i)
            || *t.get_unchecked(x + i) != *t.get_unchecked(prev + i) {
                return false;
            }
            if i != 0
            && (
                (*t.get_unchecked(x + i - 1) == TSuff::L && *t.get_unchecked(x + i) == TSuff::S)
                || (*t.get_unchecked(prev + i - 1) == TSuff::L && *t.get_unchecked(prev + i) == TSuff::S)
            ) {
                return true;
            }
        }
        false
    }
    #[inline]
    unsafe fn create_new_str<T: Ord, Scalar: SaType<Scalar>>(
        s_idx: &[T],
        offset_dict: &mut [(Scalar, Scalar)],
        sort_sublms: &[Scalar],
        t: &[TSuff],
        idx_lms: &[Scalar],
    ) -> Vec<Scalar> {
        let mut prev = Scalar::try_from(s_idx.len()).ok().unwrap() - Scalar::one();
        // safe prev < offset_dict.len() && s_idx.len() <= offset_dict.len()
        offset_dict.get_unchecked_mut(prev.to_usize()).0 = Scalar::zero();
        let sorted_lms = sort_sublms.iter().skip(1).filter(|&&x|
            // safe 0 < x < sort_sublms.len() && max(sa) < x.len() (sa contains sort sumlms => max(sa) == max(idx_lms))
            x > Scalar::zero()
            && *t.get_unchecked(x.to_usize()) == TSuff::S
            && *t.get_unchecked((x - Scalar::one()).to_usize()) == TSuff::L
        );
        for &x in sorted_lms {
            // safe x < offset_dict.len() && sorted_lms == sa &&
            // sort_sublms.len() == s_idx.len() && s_idx.len() <= offset_dict.len()
            // range(prev) == range(x)
            if sublms_is_eq(&s_idx, &t, x.to_usize(), prev.to_usize()) {
                offset_dict.get_unchecked_mut(x.to_usize()).0 = offset_dict.get_unchecked(prev.to_usize()).0
            } else {
                offset_dict.get_unchecked_mut(x.to_usize()).0 =
                    offset_dict.get_unchecked(prev.to_usize()).0 + Scalar::one()
            }
            prev = x;
        }
        // safe max(idx_lms) < s_idx.len() && s_idx.len() <= offset_dict.len()
        idx_lms.iter().map(|&x| offset_dict.get_unchecked(x.to_usize()).0).collect()
    }
    #[inline]
    unsafe fn pack_lms<T: ToUsize + Copy, Scalar: SaType<Scalar>>(
        idx_lms: &[Scalar],
        s_idx: &[T],
        offset_dict: &mut [(Scalar, Scalar)]
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
        offset_dict: &[(Scalar, Scalar)]
    ) -> Vec<Scalar> {
        // safe max(offset_dict) < idx_lms.len()
        offset_dict.iter()
            .filter(|&&(x, _)| x != Scalar::zero())
            .map(|&(_, y)| *idx_lms.get_unchecked(y.to_usize())).collect()
    }
    #[inline]
    unsafe fn sort_lms_in_new_str<T: ToUsize + Ord + Copy, Scalar: SaType<Scalar>>(
        new_s_idx: &[T],
        offset_dict: &mut [(Scalar, Scalar)],
        tmp_end_s: &mut [Scalar],
        sa: &mut [Scalar],
        sa_init: &mut impl Bit,
        idx_lms: &[Scalar]
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
            &mut sa_init.range_to_mut(new_size)
        );
        // safe new_size <= sa.len() by previous explanation && max(sa) < idx_lms.len()
        sa.get_unchecked(..new_size)
            .iter().map(|&x| *idx_lms.get_unchecked(x.to_usize())).collect()
    }
    #[inline]
    fn clear<T: Default>(src: &mut [T]) {
        src.iter_mut().for_each(|x| *x = T::default());
    }
    #[inline]
    unsafe fn sort_lms<T: ToUsize + Ord + Copy, Scalar: SaType<Scalar>>(
        s_idx: &[T],
        offset_dict: &mut [(Scalar, Scalar)],
        tmp_end_s: &mut [Scalar],
        sa: &mut [Scalar],
        sa_init: &mut impl Bit,
        t: &[TSuff],
        idx_lms: &[Scalar],
    ) -> Vec<Scalar> {
        pack_lms(idx_lms, s_idx, offset_dict);
        if idx_lms.len() == 1 {
            // safe we have sentinel '\0'
            vec![*idx_lms.get_unchecked(0)]
        } else if lms_is_unique(offset_dict) {
            unpack_lms(idx_lms, offset_dict)
        } else if idx_lms.len() <= LEN_NAIVE_SORT {
            let mut sort_lms = idx_lms.clone().to_vec();
            // safe because max(idx_lms) < s_idx.len()
            naive_sort(&mut sort_lms, s_idx);
            sort_lms
        } else if idx_lms.len() > 1 {
            clear(offset_dict);
            induced_sort(s_idx, idx_lms, t, offset_dict, tmp_end_s, sa, sa_init);
            let new_s_idx = create_new_str(s_idx, offset_dict, sa, t, idx_lms);

            sa_init.clear();
            clear(offset_dict);

            sort_lms_in_new_str(&new_s_idx, offset_dict, tmp_end_s, sa, sa_init, idx_lms)
        } else {
            unreachable!();
        }
    }
    /// https://www.researchgate.net/profile/Daricks_Wai_Hong_Chan/publication/221577802_Linear_Suffix_Array_Construction_by_Almost_Pure_Induced-Sorting/links/00b495318a21ba484f000000/Linear-Suffix-Array-Construction-by-Almost-Pure-Induced-Sorting.pdf?origin=publication_detail
    // safe if
    //      offset_dict.len() > max(s_idx) && offset_dict.len() >= s_idx.len()
    //      tmp_end_s.len() >= offset_dict.len()
    //      sa.len() >= s_idx.len()
    //      sa.len() >= s_init.len()
    //      s_idx.last() == 0
    pub(crate) unsafe fn suffix_array<T: ToUsize + Ord + Copy, Scalar: SaType<Scalar>>(
        s_idx: &[T],
        offset_dict: &mut [(Scalar, Scalar)],
        tmp_end_s: &mut [Scalar],
        sa: &mut [Scalar],
        sa_init: &mut impl Bit
    ) {
        let t = calc_type(&s_idx);
        // lms => ... L S ... (... > <= ...)
        let idx_lms = calc_lms(&t);
        let sa_lms = sort_lms(s_idx, offset_dict, tmp_end_s, sa, sa_init, &t, &idx_lms);

        sa_init.clear();
        clear(offset_dict);

        induced_sort(s_idx, &sa_lms, &t, offset_dict, tmp_end_s, sa, sa_init);
    }

    #[derive(Debug)]
    enum TState<Scalar: SaType<Scalar>> {
        Rec (
            Vec<Scalar>,
            usize,
        ),
        RecEnd (
            Vec<Scalar>,
            Vec<Scalar>,
            Vec<TSuff>,
            usize,
        ),
        End (
            Vec<Scalar>,
            Vec<TSuff>,
            usize,
        ),
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
    pub(crate) unsafe fn suffix_array_stack<T: ToUsize + Ord + Copy, Scalar: SaType<Scalar>>(
        s_idx: &[T],
        offset_dict: &mut [(Scalar, Scalar)],
        tmp_end_s: &mut [Scalar],
        sa: &mut [Scalar],
        sa_init: &mut impl Bit
    ) {
        let mut state_stack = Vec::default();

        let t = calc_type(&s_idx);
        let idx_lms = calc_lms(&t);
        pack_lms(&idx_lms, &s_idx, offset_dict);

        let (res_lms, end_state) =
            if idx_lms.len() == 1 {
                // safe we have sentinel => \0
                (vec![*idx_lms.get_unchecked(0)], NTState::End)
            } else if lms_is_unique(offset_dict) {
                (unpack_lms(&idx_lms, offset_dict), NTState::End)
            } else if idx_lms.len() > 1 {
                clear(offset_dict);
                induced_sort(&s_idx, &idx_lms, &t, offset_dict, tmp_end_s, sa, sa_init);
                let new_s_idx = create_new_str(&s_idx, offset_dict, sa, &t, &idx_lms);

                sa_init.clear();
                clear(offset_dict);

                let new_size = idx_lms.len();
                state_stack.push(TState::Rec(new_s_idx, new_size));
                (suffix_array_stack_inner(offset_dict, tmp_end_s, sa, sa_init, state_stack), NTState::RecEnd)
            } else {
                unreachable!();
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
                let sa_lms =
                    res_lms.iter()
                    .map(|&x| *idx_lms.get_unchecked(x.to_usize())).collect::<Vec<_>>();
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
    unsafe fn suffix_array_stack_inner<Scalar: SaType<Scalar>>(
        offset_dict: &mut [(Scalar, Scalar)],
        tmp_end_s: &mut [Scalar],
        sa: &mut [Scalar],
        sa_init: &mut impl Bit,
        mut state_stack: Vec<TState<Scalar>>,
    ) -> Vec<Scalar> {
        let mut res_lms = Vec::default();
        loop {
            match state_stack.pop() {
                None => return res_lms,
                Some(state) => {
                    match state {
                        TState::Rec(s_idx, size) => {
                            // safe size < offset_dict.len()
                            let offset_dict = offset_dict.get_unchecked_mut(..size);
                            // safe s_idx.len() <= sa.len()
                            let sa = sa.get_unchecked_mut(..s_idx.len());
                            // safe s_idx.len() <= sa_init.len()
                            let sa_init = &mut sa_init.range_to_mut(s_idx.len());
                            let t = calc_type(&s_idx);
                            let idx_lms = calc_lms(&t);
                            pack_lms(&idx_lms, &s_idx, offset_dict);

                            if idx_lms.len() == 1 {
                                // safe we have sentinel => \0
                                res_lms = vec![*idx_lms.get_unchecked(0)];

                                state_stack.push(TState::End(s_idx, t, size));
                            } else if lms_is_unique(offset_dict) {
                                res_lms = unpack_lms(&idx_lms, offset_dict);

                                state_stack.push(TState::End(s_idx, t, size));
                            } else if idx_lms.len() <= LEN_NAIVE_SORT {
                                let mut sort_lms = idx_lms.clone().to_vec();
                                // safe because max(idx_lms) < s_idx.len()
                                naive_sort(&mut sort_lms, &s_idx);
                                res_lms = sort_lms;
                                state_stack.push(TState::End(s_idx, t, size));
                            } else if idx_lms.len() > 1 {
                                clear(offset_dict);
                                // safe size < offset_dict.len() && tmp_end_s.len() <= offset_dict.len() && sa.len() == s_idx.len()
                                induced_sort(&s_idx, &idx_lms, &t, offset_dict,
                                    tmp_end_s.get_unchecked_mut(..size),
                                    sa, sa_init);
                                let new_s_idx = create_new_str(&s_idx, offset_dict, sa, &t, &idx_lms);

                                sa_init.clear();
                                clear(offset_dict);

                                let new_size = idx_lms.len();
                                state_stack.push(TState::RecEnd(s_idx, idx_lms, t, size));
                                state_stack.push(TState::Rec(new_s_idx, new_size));
                            } else {
                                unreachable!();
                            };
                        }
                        TState::End(s_idx, t, size) => {
                            // safe size == idx_lms.len()
                            let offset_dict = offset_dict.get_unchecked_mut(..size);
                            let sa = sa.get_unchecked_mut(..s_idx.len());
                            let mut sa_init = sa_init.range_to_mut(s_idx.len());
                            sa_init.clear();
                            clear(offset_dict);
                            // safe size < offset_dict.len() && tmp_end_s.len() <= offset_dict.len() && sa.len() == s_idx.len()
                            induced_sort(&s_idx, &res_lms, &t, offset_dict,
                                tmp_end_s.get_unchecked_mut(..size),
                                sa, &mut sa_init);
                            res_lms = Vec::from(sa);
                        }
                        TState::RecEnd(s_idx, idx_lms, t, size) => {
                            // safe size == idx_lms.len()
                            let offset_dict = offset_dict.get_unchecked_mut(..size);
                            let sa = sa.get_unchecked_mut(..s_idx.len());
                            let mut sa_init = sa_init.range_to_mut(s_idx.len());
                            sa_init.clear();
                            clear(offset_dict);
                            // safe size < offset_dict.len() && tmp_end_s.len() <= offset_dict.len() && sa.len() == s_idx.len()
                            let sa_lms =
                                res_lms.iter()
                                .map(|&x| *idx_lms.get_unchecked(x.to_usize())).collect::<Vec<_>>();
                            induced_sort(&s_idx, &sa_lms, &t, offset_dict,
                                tmp_end_s.get_unchecked_mut(..size),
                                sa, &mut sa_init);
                            res_lms = Vec::from(sa);
                        }
                    }
                }
            }
        }
    }
}