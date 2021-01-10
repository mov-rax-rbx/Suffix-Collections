//! no_std support

//! # Suffix Array
//! Implementation of the [suffix array](https://www.researchgate.net/profile/Daricks_Wai_Hong_Chan/publication/221577802_Linear_Suffix_Array_Construction_by_Almost_Pure_Induced-Sorting/links/00b495318a21ba484f000000/Linear-Suffix-Array-Construction-by-Almost-Pure-Induced-Sorting.pdf?origin=publication_detail)
//! construction of which is performed in linear time
//!
//! ## Examples
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

//! # Suffix Tree
//! Implementation of the [suffix tree](https://web.stanford.edu/~mjkay/gusfield.pdf)
//! construction of which is performed in linear time
//!
//! ## Examples
//!
//! ```
//!     use suff_collections::{array::*, tree::*, lcp::*};
//!
//!     // let word = "Some word";
//!     let word: &str = "Some word\0";
//!     let find: &str = "word";
//!
//!     // construct suffix tree
//!     let st: SuffixTree = SuffixTree::new(word);
//!
//!     // finds the entry position of the line 'find' in 'word'
//!     let res: Option<usize> = st.find(find);
//!
//!     // construct lcp
//!     // lcp[i] = max_pref(sa[i], sa[i - 1]) && lcp.len() == sa.len()
//!     // let lcp: LCP<u8> = st.lcp_stack::<u8>();
//!     // let lcp: LCP<u16> = st.lcp_stack::<u16>();
//!     // let lcp: LCP<u32> = st.lcp_stack::<u32>();
//!     // let lcp: LCP<usize> = st.lcp_stack::<usize>();
//!     let lcp: LCP<usize> = st.lcp_rec::<usize>();
//!
//!     // convert suffix tree to suffix array
//!     // let sa = SuffixArray::<u8>::from_stack(st);
//!     // let sa = SuffixArray::<u16>::from_stack(st);
//!     // let sa = SuffixArray::<u32>::from_stack(st);
//!     // let sa = SuffixArray::<usize>::from_stack(st);
//!     let sa = SuffixArray::<usize>::from_rec(st);
//! ```

#![no_std]
#[macro_use(vec)]
extern crate alloc;
pub mod array;
pub mod lcp;
pub mod tree;

pub(crate) mod bit;

use alloc::borrow::{Cow, ToOwned};
use alloc::vec::Vec;
use core::str;
fn canonic_word<'t>(word: &'t str) -> Cow<'t, str> {
    if word.as_bytes().last() == Some(&0) {
        Cow::from(word)
    } else {
        Cow::from(
            str::from_utf8(
                &word
                    .as_bytes()
                    .iter()
                    .chain(&[0])
                    .map(|&x| x)
                    .collect::<Vec<_>>(),
            )
            .unwrap()
            .to_owned(),
        )
    }
}
