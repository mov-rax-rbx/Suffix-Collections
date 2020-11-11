//! Implementation of the suffix array construction of which is performed in linear time
//! https://www.researchgate.net/profile/Daricks_Wai_Hong_Chan/publication/221577802_Linear_Suffix_Array_Construction_by_Almost_Pure_Induced-Sorting/links/00b495318a21ba484f000000/Linear-Suffix-Array-Construction-by-Almost-Pure-Induced-Sorting.pdf?origin=publication_detail


extern crate alloc;

use alloc::vec::{Vec, IntoIter};
use alloc::borrow::Cow;
use core::{str, slice::Iter, option::Option, cmp::{max, Eq}};

use crate::tree::*;
use crate::lcp::*;

#[derive(Debug, Clone)]
pub struct SuffixArray<'sa> {
    word: Cow<'sa, str>,
    sa: Vec<usize>
}

impl<'sa> IntoIterator for SuffixArray<'sa> {
    type Item = usize;
    type IntoIter = IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.sa.into_iter()
    }
}

impl<'sa> IntoIterator for &'sa SuffixArray<'sa> {
    type Item = &'sa usize;
    type IntoIter = Iter<'sa, usize>;

    fn into_iter(self) -> Self::IntoIter {
        self.sa.iter()
    }
}

impl<'sa> SuffixArray<'sa> {
    const BYTE_SIZE: usize = 256;

    /// Construct suffix array recursive. Complexity O(n)
    /// ```
    /// use suff_collections::{array::*, tree::*};
    ///
    /// let sa: SuffixArray = SuffixArray::new("word");
    /// let sa: SuffixArray = SuffixArray::new("word\0");
    /// ```
    /// At the end of the line should hit '\0'.
    /// If there is no '\0' at the end then the line will be copied and added '\0' to the end.
    /// Otherwise, the value will be taken by reference
    pub fn new(word: &'sa str) -> Self {
        if word.is_empty() {
            return Self {
                word: Cow::from(""),
                sa: vec![],
            };
        }
        let word =
            if word.as_bytes().last() == Some(&0) {
                Cow::from(word)
            } else {
                Cow::from(str::from_utf8(&word.as_bytes().iter().chain(&[0]).map(|&x| x).collect::<Vec<_>>()).unwrap().to_owned())
            };

        let mut offset_dict = vec![(0, 0); max(word.len(), SuffixArray::BYTE_SIZE)];
        let mut tmp_end_s = vec![0; offset_dict.len()];
        let mut sa = vec![0; word.len()];
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
            build_suffix_array::suffix_array(word.as_bytes(), &mut offset_dict, &mut tmp_end_s, &mut sa, &mut sa_init);
        }

        Self {
            word: word,
            sa: sa,
        }
    }

    /// Construct suffix array not recursive. Complexity O(n)
    /// ```
    /// use suff_collections::{array::*, tree::*};
    ///
    /// let sa: SuffixArray = SuffixArray::new_stack("word");
    /// let sa: SuffixArray = SuffixArray::new_stack("word\0");
    /// ```
    /// At the end of the line should hit '\0'.
    /// If there is no '\0' at the end then the line will be copied and added '\0' to the end.
    /// Otherwise, the value will be taken by reference
    pub fn new_stack(word: &'sa str) -> Self {
        if word.is_empty() {
            return Self {
                word: Cow::from(""),
                sa: vec![],
            };
        }
        let word =
            if word.as_bytes().last() == Some(&0) {
                Cow::from(word)
            } else {
                Cow::from(str::from_utf8(&word.as_bytes().iter().chain(&[0]).map(|&x| x).collect::<Vec<_>>()).unwrap().to_owned())
            };
        let mut offset_dict = vec![(0, 0); std::cmp::max(word.len(), SuffixArray::BYTE_SIZE)];
        let mut tmp_end_s = vec![0; offset_dict.len()];
        let mut sa = vec![0; word.len()];
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
            build_suffix_array::suffix_array_stack(word.as_bytes(), &mut offset_dict, &mut tmp_end_s, &mut sa, &mut sa_init)
        };

        Self {
            word: word,
            sa: sa,
        }
    }

    /// Construct suffix array from suffix tree not recursive. Complexity O(n)
    /// ```
    /// use suff_collections::{array::*, tree::*};
    ///
    /// let st: SuffixTree = SuffixTree::new("word\0");
    /// let sa: SuffixArray = SuffixArray::from_stack(st);
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
                        sa.push(node.pos() - x.len);
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
    /// let st: SuffixTree = SuffixTree::new("word\0");
    /// let sa: SuffixArray = SuffixArray::from_rec(st);
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
    /// SuffixArray::new("word\0").iter().for_each(|&idx| println!("idx: {}", idx));
    /// ```
    #[inline]
    pub fn iter(&self) -> Iter<'_, usize> {
        self.sa.iter()
    }

    /// Split suffix array
    /// ```
    /// use suff_collections::array::*;
    ///
    /// let (word, sa) = SuffixArray::new("word").split_owned();
    /// ```
    #[inline]
    pub fn split_owned(self) -> (Cow<'sa, str>, Vec<usize>) {
        (self.word, self.sa)
    }

    /// Return ref on suffix array
    /// ```
    /// use suff_collections::array::*;
    ///
    /// let sa = SuffixArray::new("word");
    /// let sa: &[usize] = sa.suffix_array();
    /// ```
    #[inline]
    pub fn suffix_array(&self) -> &Vec<usize> {
        &self.sa
    }

    /// Return ref on word
    /// ```
    /// use suff_collections::array::*;
    ///
    /// let sa = SuffixArray::new("word");
    /// let word: &str = sa.word();
    /// assert_eq!("word\0", word);
    /// ```
    #[inline]
    pub fn word(&self) -> &str {
        &self.word
    }

    /// lcp[i] = max_pref(sa[i], sa[i - 1]) && lcp.len() == sa.len()
    /// Construct LCP. Complexity O(n)
    /// ```
    /// use suff_collections::array::*;
    ///
    /// let lcp = SuffixArray::new("word").lcp();
    /// ```
    pub fn lcp(&self) -> LCP {
        let mut lcp = LCP::new(vec![0; self.sa.len()]);
        let mut sa_idx = vec![0; self.sa.len()];
        // safe max(sa) < sa_idx.len()
        self.sa.iter().enumerate().for_each(|(i, &x)| unsafe { *sa_idx.get_unchecked_mut(x) = i + 1 });
        let mut pref_len = 0;
        let word = self.word.as_bytes();
        for x in sa_idx {
            if x == self.sa.len() {
                pref_len = 0;
                continue;
            }
            // safe max(sa_idx) < sa.len() && x < sa.len() by previous check
            // safe l < word.len() && r < word.len()
            let l = unsafe { *self.sa.get_unchecked(x - 1) };
            let r = unsafe { *self.sa.get_unchecked(x) };
            pref_len = unsafe { count_eq(word.get_unchecked(l..), word.get_unchecked(r..), pref_len) };
            // safe x < sa.len() by previous check && lcp.len() == sa.len()
            unsafe { *lcp.idx_mut(x) = pref_len; }
            if pref_len > 0 {
                pref_len -= 1;
            }
        }
        lcp
    }

    /// Find substr. Complexity O(|find| * log(|word|))
    /// ```
    /// use suff_collections::array::*;
    ///
    /// let find: Option<usize> = SuffixArray::new("word").find("or");
    /// assert_eq!(find, Some(1));
    /// ```
    #[inline]
    pub fn find(&self, find: &str) -> Option<usize> {
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
    /// let sa = SuffixArray::new("word");
    /// let find: &[usize] = sa.find_all("or");
    /// assert_eq!(find, &[1]);
    /// ```
    #[inline]
    pub fn find_all(&self, find: &str) -> &[usize] {
        let (start, end) = self.find_pos(find);
        // safe start <= end && 0 <= start < sa.len() && 0 <= end < sa.len()
        unsafe { self.sa.get_unchecked(start..end) }
    }

    /// Find substr. Complexity O(|word|)
    /// ```
    /// use suff_collections::array::*;
    ///
    /// let sa = SuffixArray::new("word");
    /// let find: Option<usize> = sa.find_big(&sa.lcp(), "or");
    /// assert_eq!(find, Some(1));
    /// ```
    #[inline]
    pub fn find_big(&self, lcp: &LCP, find: &str) -> Option<usize> {
        let idx = self.find_pos_big(lcp, find)?;
        // safe by previous operation
        Some(unsafe { *self.sa.get_unchecked(idx) })
    }

    /// Find all substr. Complexity O(|word|)
    /// ```
    /// use suff_collections::array::*;
    ///
    /// let sa = SuffixArray::new("word");
    /// let find: &[usize] = sa.find_all_big(&sa.lcp(), "or");
    /// assert_eq!(find, &[1]);
    /// ```
    #[inline]
    pub fn find_all_big(&self, lcp: &LCP, find: &str) -> &[usize] {
        match self.find_pos_big(lcp, find) {
            None => &[],
            Some(start) => {
                // safe because 0 <= start < self.sa.len()
                let end = start + lcp.iter().skip(start + 1).take_while(|&&lcp| find.len() <= lcp).count();
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
            unsafe { word.get_unchecked(idx..) < find }
        });
        // skip all matches
        // safe because 0 <= start < self.sa.len()
        let end = start + unsafe {
            binary_search(self.sa.get_unchecked(start..), |&idx| {
                // safe because binary search correct =>
                //  0 <= idx < self.sa.len() && max(sa) < word.len()
                idx + find.len() < word.len() && word.get_unchecked(idx..idx + find.len()) == find
            })
        };

        (start, end)
    }
    // O(|word|)
    fn find_pos_big(&self, lcp: &LCP, find: &str) -> Option<usize> {
        if find.is_empty() {
            return None;
        }
        let (word, find) = (self.word.as_bytes(), find.as_bytes());
        // entry of the first character (byte) is searched for by means of binary search
        // safe because binary search correct =>
        //  0 <= idx < self.sa.len() && max(sa) < word.len() && !find.is_empty()
        let start = binary_search(&self.sa,
            |&idx| unsafe { word.get_unchecked(idx) < find.get_unchecked(0) });

        // loop continue while total_eq < find.len() and total_eq never decrement
        let mut total_eq = 0;
        for (&idx, i) in self.sa.iter().skip(start).zip((start + 1..).into_iter()) {
            // safe word_idx < word.len() && total_eq < find.len()
            total_eq = unsafe { count_eq(word.get_unchecked(idx..), find.get_unchecked(..), total_eq) };
            if total_eq == find.len() {
                return Some(i - 1);
            }

            // safe i < lcp.len()
            if i < lcp.len() && total_eq > unsafe { *lcp.idx(i) } {
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

fn count_eq<T: Eq>(cmp1: &[T], cmp2: &[T], mut acc: usize) -> usize {
    while acc < cmp1.len() && acc < cmp2.len()
    // safe by previous check
        && unsafe { *cmp1.get_unchecked(acc) == *cmp2.get_unchecked(acc) }
    {
        acc += 1;
    }
    acc
}

fn to_suffix_array_rec_inner(tree: &SuffixTree, node_idx: NodeIdx, len: usize, sa: &mut Vec<usize>) {
    let node = tree.node(node_idx);
    if node.children().is_empty() {
        sa.push(node.pos() - len);
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
mod build_suffix_array {
    #[repr(u8)]
    #[derive(Debug, Copy, Clone, PartialOrd, PartialEq)]
    enum TSuff {
        L,
        S
    }

    pub(crate) trait ToUsize {
        fn to_usize(self) -> usize;
    }
    macro_rules! impl_ToUsize {
        ($($tp:ident),* $(,)?) => {
            $(
                impl ToUsize for $tp {
                    fn to_usize(self) -> usize {
                        self as usize
                    }
                }
            )*
        };
    }
    impl_ToUsize!(u8, u16, u32, u64, usize);
    use core::cmp::Ord;

    const LEN_NAIVE_SORT: usize = 50;
    #[inline]
    unsafe fn naive_sort<T: ToUsize + Ord + Copy, P: Ord>(sort: &mut [T], s_idx: &[P]) {
        // safe if max(sort) < s_idx.len()
        debug_assert!(sort.iter().max().unwrap().to_usize() < s_idx.len());
        sort.sort_by(|&a, &b| s_idx.get_unchecked(a.to_usize()..).cmp(&s_idx.get_unchecked(b.to_usize()..)));
    }
    #[inline]
    unsafe fn create_bucket_dict<T: ToUsize + Copy>(
        s_idx: &[T],
        offset_dict: &mut [(usize, usize)]
    ) {
        // safe because max(s_idx) < offset_dict.len()
        s_idx.iter().for_each(|&x| offset_dict.get_unchecked_mut(x.to_usize()).0 += 1);
        offset_dict.iter_mut().fold(0, |acc, offs| {
            let cnt = offs.0;
            *offs = (acc, acc + cnt);
            acc + cnt
        });
    }
    #[inline]
    unsafe fn add_lms_to_end<T: ToUsize + Copy>(
        s_idx: &[T],
        sa_lms: &[usize],
        offset_dict: &mut [(usize, usize)],
        tmp_end_s: &mut [usize],
        sa: &mut [usize],
        sa_init: &mut [bool]
    ) {
        offset_dict.iter().zip(tmp_end_s.iter_mut()).for_each(|(&(_, x), end_s)| *end_s = x);
        sa_lms.iter().rev().for_each(|&x| {
            // safe x == sa_lms && max(sa_lms) < s_idx_len() && max(s_idx) < tmp_end_s.len() &&
            // max(tmp_end_s) < sa.len() && sa_init.len() == sa.len()
            let ptr_end_s = tmp_end_s.get_unchecked_mut(s_idx.get_unchecked(x).to_usize());
            *ptr_end_s -= 1;
            *sa.get_unchecked_mut(*ptr_end_s) = x;
            *sa_init.get_unchecked_mut(*ptr_end_s) = true;
        });
    }
    #[allow(non_snake_case)]
    #[inline]
    unsafe fn add_L_to_start<T: ToUsize + Copy>(
        s_idx: &[T],
        t: &[TSuff],
        offset_dict: &mut [(usize, usize)],
        sa: &mut [usize],
        sa_init: &mut [bool]
    ) {
        for x in 0..sa.len() {
            // safe x < sa.len() && sa_init.len() == sa.len()
            let mut idx = *sa.get_unchecked(x);
            if idx > 0 && *sa_init.get_unchecked(x) == true {
                idx -= 1;
                // safe idx == x && 0 < x < sa.len() && t.len() == s_idx.len() &&
                // max(s_idx) < offset_dict.len() && max(offset_dict) < sa.len()
                if *t.get_unchecked(idx) == TSuff::L {
                    let ptr_offset = offset_dict.get_unchecked_mut(s_idx.get_unchecked(idx).to_usize());
                    *sa.get_unchecked_mut(ptr_offset.0) = idx;
                    *sa_init.get_unchecked_mut(ptr_offset.0) = true;
                    ptr_offset.0 += 1;
                }
            }
        }
    }
    #[allow(non_snake_case)]
    #[inline]
    unsafe fn add_S_to_end<T: ToUsize + Copy>(
        s_idx: &[T],
        t: &[TSuff],
        offset_dict: &mut [(usize, usize)],
        sa: &mut [usize],
        sa_init: &mut [bool]
    ) {
        for x in (0..sa.len()).rev() {
            // safe x < sa.len() && sa_init.len() == sa.len()
            let mut idx = *sa.get_unchecked(x);
            if idx > 0 && *sa_init.get_unchecked(x) == true {
                idx -= 1;
                // safe idx == x && 0 < x < sa.len() && t.len() == s_idx.len() &&
                // max(s_idx) < offset_dict.len() && max(offset_dict) < sa.len()
                if *t.get_unchecked(idx) == TSuff::S {
                    let ptr_offset = offset_dict.get_unchecked_mut(s_idx.get_unchecked(idx).to_usize());
                    ptr_offset.1 -= 1;
                    *sa.get_unchecked_mut(ptr_offset.1) = idx;
                    *sa_init.get_unchecked_mut(ptr_offset.1) = true;
                }
            }
        }
    }
    #[inline]
    unsafe fn induced_sort<T: ToUsize + Copy>(
        s_idx: &[T],
        sa_lms: &[usize],
        t: &[TSuff],
        offset_dict: &mut [(usize, usize)],
        tmp_end_s: &mut [usize],
        sa: &mut [usize],
        sa_init: &mut [bool]
    ) {
        create_bucket_dict(s_idx, offset_dict);
        add_lms_to_end(s_idx, sa_lms, offset_dict, tmp_end_s, sa, sa_init);
        add_L_to_start(s_idx, t, offset_dict, sa, sa_init);
        add_S_to_end(s_idx, t, offset_dict, sa, sa_init);
    }
    #[inline]
    unsafe fn calc_type<T: Ord>(s_idx: &[T]) -> Vec<TSuff> {
        let mut t = vec![TSuff::S; s_idx.len()];
        for i in (0..s_idx.len() - 1).rev() {
            // safe 0 < i + 1 < s_idx.len() && t.len() == s_idx_len()
            if *s_idx.get_unchecked(i) > *s_idx.get_unchecked(i + 1) {
                *t.get_unchecked_mut(i) = TSuff::L;
            } else if *s_idx.get_unchecked(i) == *s_idx.get_unchecked(i + 1) {
                *t.get_unchecked_mut(i) = *t.get_unchecked(i + 1);
            }
        }
        t
    }
    #[inline]
    fn calc_lms(t: &[TSuff]) -> Vec<usize> {
        t.windows(2).enumerate()
            .filter(|&(_, x)| x[0] == TSuff::L && x[1] == TSuff::S)
            .map(|(i, _)| i + 1).collect::<Vec<_>>()
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
    unsafe fn create_new_str<T: Ord>(
        s_idx: &[T],
        offset_dict: &mut [(usize, usize)],
        sort_sublms: &[usize],
        t: &[TSuff],
        idx_lms: &[usize],
    ) -> Vec<usize> {
        let mut prev = s_idx.len() - 1;
        // safe prev < offset_dict.len() && s_idx.len() <= offset_dict.len()
        offset_dict.get_unchecked_mut(prev).0 = 0;
        let sorted_lms = sort_sublms.iter().skip(1).filter(|&&x|
            // safe 0 < x < sort_sublms.len() && max(sa) < x.len() (sa contains sort sumlms => max(sa) == max(idx_lms))
            x > 0 && *t.get_unchecked(x) == TSuff::S && *t.get_unchecked(x - 1) == TSuff::L
        );
        for &x in sorted_lms {
            // safe x < offset_dict.len() && sorted_lms == sa &&
            // sort_sublms.len() == s_idx.len() && s_idx.len() <= offset_dict.len()
            // range(prev) == range(x)
            if sublms_is_eq(&s_idx, &t, x, prev) {
                offset_dict.get_unchecked_mut(x).0 = offset_dict.get_unchecked(prev).0
            } else {
                offset_dict.get_unchecked_mut(x).0 = offset_dict.get_unchecked(prev).0 + 1
            }
            prev = x;
        }
        // safe max(idx_lms) < s_idx.len() && s_idx.len() <= offset_dict.len()
        idx_lms.iter().map(|&x| offset_dict.get_unchecked(x).0).collect()
    }
    #[inline]
    unsafe fn pack_lms<T: ToUsize + Copy>(idx_lms: &[usize], s_idx: &[T], offset_dict: &mut [(usize, usize)]) {
        idx_lms.iter().enumerate().for_each(|(i, &x)| {
            // safe max(idx_lms) < s_idx.len() &&
            // max(s_idx) < offset_dict.len()
            let index = s_idx.get_unchecked(x).to_usize();
            offset_dict.get_unchecked_mut(index).0 += 1;
            offset_dict.get_unchecked_mut(index).1 = i;
        });
    }
    #[inline]
    fn lms_is_unique(offset_dict: &[(usize, usize)]) -> bool {
        !offset_dict.iter().any(|&(x, _)| x > 1)
    }
    #[inline]
    unsafe fn unpack_lms(idx_lms: &[usize], offset_dict: &[(usize, usize)]) -> Vec<usize> {
        // safe max(offset_dict) < idx_lms.len()
        offset_dict.iter()
            .filter(|&&(x, _)| x != 0)
            .map(|&(_, y)| *idx_lms.get_unchecked(y)).collect()
    }
    #[inline]
    unsafe fn sort_lms_in_new_str<T: ToUsize + Ord + Copy>(
        new_s_idx: &[T],
        offset_dict: &mut [(usize, usize)],
        tmp_end_s: &mut [usize],
        sa: &mut [usize],
        sa_init: &mut [bool],
        idx_lms: &[usize]
    ) -> Vec<usize> {
        let new_size = idx_lms.len();
        // safe new_size == idx_lms.len() && max(idx_lms) < s_idx.len() &&
        // s_idx.len() <= offset_dict.len() && s_idx.len() == tmp_end_s.len() &&
        // s_idx.len() == sa.len() && sa.len() == sa_init.len()
        suffix_array(
            &new_s_idx,
            offset_dict.get_unchecked_mut(..new_size),
            tmp_end_s.get_unchecked_mut(..new_size),
            sa.get_unchecked_mut(..new_size),
            sa_init.get_unchecked_mut(..new_size)
        );
        // safe new_size <= sa.len() by previous explanation && max(sa) < idx_lms.len()
        sa.get_unchecked(..new_size)
            .iter().map(|&x| *idx_lms.get_unchecked(x)).collect()
    }
    #[inline]
    fn clear<T: Default>(src: &mut [T]) {
        src.iter_mut().for_each(|x| *x = T::default());
    }
    #[inline]
    unsafe fn sort_lms<T: ToUsize + Ord + Copy>(
        s_idx: &[T],
        offset_dict: &mut [(usize, usize)],
        tmp_end_s: &mut [usize],
        sa: &mut [usize],
        sa_init: &mut [bool],
        t: &[TSuff],
        idx_lms: &[usize],
    ) -> Vec<usize> {
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

            clear(sa_init);
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
    pub(crate) unsafe fn suffix_array<T: ToUsize + Ord + Copy>(
        s_idx: &[T],
        offset_dict: &mut [(usize, usize)],
        tmp_end_s: &mut [usize],
        sa: &mut [usize],
        sa_init: &mut [bool]
    ) {
        let t = calc_type(&s_idx);
        // lms => ... L S ... (... > <= ...)
        let idx_lms = calc_lms(&t);
        let sa_lms = sort_lms(s_idx, offset_dict, tmp_end_s, sa, sa_init, &t, &idx_lms);

        clear(sa_init);
        clear(offset_dict);

        induced_sort(s_idx, &sa_lms, &t, offset_dict, tmp_end_s, sa, sa_init);
    }

    #[derive(Debug)]
    enum TState {
        Rec (
            Vec<usize>,
            usize,
        ),
        RecEnd (
            Vec<usize>,
            Vec<usize>,
            Vec<TSuff>,
            usize,
        ),
        End (
            Vec<usize>,
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
    pub(crate) unsafe fn suffix_array_stack<T: ToUsize + Ord + Copy>(
        s_idx: &[T],
        offset_dict: &mut [(usize, usize)],
        tmp_end_s: &mut [usize],
        sa: &mut [usize],
        sa_init: &mut [bool]
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

                clear(sa_init);
                clear(offset_dict);

                let new_size = idx_lms.len();
                state_stack.push(TState::Rec(new_s_idx, new_size));
                (suffix_array_stack_inner(offset_dict, tmp_end_s, sa, sa_init, state_stack), NTState::RecEnd)
            } else {
                unreachable!();
            };

        match end_state {
            NTState::End => {
                clear(sa_init);
                clear(offset_dict);
                induced_sort(&s_idx, &res_lms, &t, offset_dict, tmp_end_s, sa, sa_init);
            }
            NTState::RecEnd => {
                clear(sa_init);
                clear(offset_dict);
                // safe size < offset_dict.len() && tmp_end_s.len() <= offset_dict.len() && sa.len() == s_idx.len()
                let sa_lms = res_lms.iter().map(|&x| *idx_lms.get_unchecked(x)).collect::<Vec<_>>();
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
    unsafe fn suffix_array_stack_inner(
        offset_dict: &mut [(usize, usize)],
        tmp_end_s: &mut [usize],
        sa: &mut [usize],
        sa_init: &mut [bool],
        mut state_stack: Vec<TState>,
    ) -> Vec<usize> {
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
                            let sa_init = sa_init.get_unchecked_mut(..s_idx.len());
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

                                clear(sa_init);
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
                            let sa_init = sa_init.get_unchecked_mut(..s_idx.len());
                            clear(sa_init);
                            clear(offset_dict);
                            // safe size < offset_dict.len() && tmp_end_s.len() <= offset_dict.len() && sa.len() == s_idx.len()
                            induced_sort(&s_idx, &res_lms, &t, offset_dict,
                                tmp_end_s.get_unchecked_mut(..size),
                                sa, sa_init);
                            res_lms = Vec::from(sa);
                        }
                        TState::RecEnd(s_idx, idx_lms, t, size) => {
                            // safe size == idx_lms.len()
                            let offset_dict = offset_dict.get_unchecked_mut(..size);
                            let sa = sa.get_unchecked_mut(..s_idx.len());
                            let sa_init = sa_init.get_unchecked_mut(..s_idx.len());
                            clear(sa_init);
                            clear(offset_dict);
                            // safe size < offset_dict.len() && tmp_end_s.len() <= offset_dict.len() && sa.len() == s_idx.len()
                            let sa_lms = res_lms.iter().map(|&x| *idx_lms.get_unchecked(x)).collect::<Vec<_>>();
                            induced_sort(&s_idx, &sa_lms, &t, offset_dict,
                                tmp_end_s.get_unchecked_mut(..size),
                                sa, sa_init);
                            res_lms = Vec::from(sa);
                        }
                    }
                }
            }
        }
    }
}
