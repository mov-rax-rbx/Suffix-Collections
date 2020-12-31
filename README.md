# Suffix Collections

![Build Status](https://github.com/mov-rax-rbx/Suffix-Collections/workflows/Rust/badge.svg)
[![LICENSE](https://img.shields.io/crates/l/suff_collections)](LICENSE)
[![Crates](https://img.shields.io/crates/v/suff_collections)](https://crates.io/crates/suff_collections)
[![Documentation](https://docs.rs/suff_collections/badge.svg)](https://docs.rs/suff_collections)

Fast realization of suffix array and suffix tree for substring search, longest common prefix array (lcp array).

## Example
* **SuffixTree**
```rust
     use suff_collections::{array::*, tree::*, lcp::*};

     // let word = "Some word";
     let word: &str = "Some word\0";
     let find: &str = "word";

     // construct suffix tree
     let st: SuffixTree = SuffixTree::new(word);

     // finds the entry position of the line 'find' in 'word'
     let res: Option<usize> = st.find(find);

     // construct lcp
     // lcp[i] = max_pref(sa[i], sa[i - 1]) && lcp.len() == sa.len()
     // let lcp: LCP<u8> = st.lcp_stack::<u8>();
     // let lcp: LCP<u16> = st.lcp_stack::<u16>();
     // let lcp: LCP<u32> = st.lcp_stack::<u32>();
     // let lcp: LCP<usize> = st.lcp_stack::<usize>();
     let lcp: LCP<usize> = st.lcp_rec::<usize>();

     // convert suffix tree to suffix array
     // let sa = SuffixArray::<u8>::from_stack(st);
     // let sa = SuffixArray::<u16>::from_stack(st);
     // let sa = SuffixArray::<u32>::from_stack(st);
     // let sa = SuffixArray::<usize>::from_stack(st);
     let sa = SuffixArray::<usize>::from_rec(st);
```

* **SuffixArray**
```rust
     use suff_collections::{array::*, tree::*, lcp::*};

     // let word = "Some word";
     let word: &str = "Some word\0";
     let find: &str = "word";

     // construct suffix array
     // let sa = SuffixArray::<usize>::new_stack(word);
     // let sa = SuffixArray::<u8>::new(word);
     // let sa = SuffixArray::<u16>::new(word);
     // let sa = SuffixArray::<u32>::new(word);
     let sa = SuffixArray::<usize>::new(word);

     // construct lcp
     // lcp[i] = max_pref(sa[i], sa[i - 1]) && lcp.len() == sa.len()
     let lcp: LCP<usize> = sa.lcp();

     // finds the entry position of the line 'find' in 'word'
     // O(|find| * log(|word|))
     let res: Option<usize> = sa.find(find);

     // finds all the entry position of the line 'find' in 'word'
     // O(|find| * log(|word|))
     let res_all: &[usize] = sa.find_all(find);

     // finds the entry position of the line 'find' in 'word'
     // O(|word|)
     let res: Option<usize> = sa.find_big(&sa.lcp(), find);

     // finds all the entry position of the line 'find' in 'word'
     // O(|word|)
     let res_all: &[usize] = sa.find_all_big(&sa.lcp(), find);

     // convert suffix array to suffix tree
     let st = SuffixTree::from(sa);
```
All construction and search work for O(n). For the suffix tree implementation the [Ukkonen algorithm][2] is taken and for the suffix array implementation the [SA-IS algorithm][1] is taken.

[1]: https://www.researchgate.net/profile/Daricks_Wai_Hong_Chan/publication/221577802_Linear_Suffix_Array_Construction_by_Almost_Pure_Induced-Sorting/links/00b495318a21ba484f000000/Linear-Suffix-Array-Construction-by-Almost-Pure-Induced-Sorting.pdf?origin=publication_detail

[2]: https://web.stanford.edu/~mjkay/gusfield.pdf

# Some benches *(thank [Criterion](https://github.com/bheisler/criterion.rs))*

* ### *sufix_array\<usize>.lcp()*
        time:   [401.96 us 403.67 us 405.75 us]

* ### *sufix_array\<u32>.lcp()*
        time:   [371.61 us 373.21 us 375.08 us]

* ### *SuffixArray::\<usize>::new(LINE)*
        time:   [2.2718 ms 2.2871 ms 2.3069 ms]

* ### *SuffixArray::\<usize>::new_compress(LINE)*
        time:   [2.5368 ms 2.5529 ms 2.5703 ms]

* ### *SuffixArray::\<u32>::new(LINE)*
        time:   [1.9684 ms 1.9752 ms 1.9828 ms]

* ### *SuffixArray::\<u32>::new_compress(LINE)*
        time:   [2.2396 ms 2.2533 ms 2.2697 ms]

* ### *SuffixArray::\<usize>::new_stack(LINE)*
        time:   [2.3389 ms 2.3777 ms 2.4314 ms]

* ### *SuffixArray::\<usize>::new_stack_compress(LINE)*
        time:   [2.6015 ms 2.6166 ms 2.6327 ms]

* ### *SuffixArray::\<u32>::new_stack(LINE)*
        time:   [2.0221 ms 2.0310 ms 2.0405 ms]

* ### *SuffixArray::\<u32>::new_stack_compress(LINE)*
        time:   [2.2962 ms 2.3078 ms 2.3210 ms]

* ### *suffix_array\<usize>.find(FIND) ~ O(|find| * log(|word|))*
        time:   [4.1127 us 4.1273 us 4.1415 us]

* ### *suffix_array\<u32>.find(FIND) ~ O(|find| * log(|word|))*
        time:   [4.1591 us 4.1695 us 4.1808 us]

* ### *suffix_array\<usize>.find_all(FIND) ~ O(|find| * log(|word|))*
        time:   [4.1205 us 4.1366 us 4.1528 us]

* ### *suffix_array\<u32>.find_all(FIND) ~ O(|find| * log(|word|))*
        time:   [4.1452 us 4.1560 us 4.1678 us]

* ### *suffix_array\<usize>.find_big(FIND) ~ O(|word|)*
        time:   [22.410 us 22.507 us 22.607 us]

* ### *suffix_array\<u32>.find_big(FIND) ~ O(|word|)*
        time:   [19.944 us 20.299 us 20.752 us]

* ### *suffix_array\<usize>.find_big_all(FIND) ~ O(|word|)*
        time:   [22.613 us 22.715 us 22.820 us]

* ### *suffix_array\<u32>.find_big_all(FIND) ~ O(|word|)*
        time:   [19.946 us 20.283 us 20.672 us]

* ### *SuffixTree::from(suffix_array)*
        time:   [13.533 ms 13.666 ms 13.812 ms]

* ### *Suffix tree build Ukkonen*
        time:   [15.316 ms 15.433 ms 15.558 ms]

* ### *Suffix tree find*
        time:   [20.506 us 20.839 us 21.221 us]

* ### *SuffixArray::\<usize>::from_rec(suffix_tree)*
        time:   [16.392 ms 16.494 ms 16.602 ms]

* ### *SuffixArray::\<u32>::from_rec(suffix_tree)*
        time:   [16.310 ms 16.411 ms 16.516 ms]

* ### *SuffixArray::\<usize>::from_stack(suffix_tree)*
        time:   [16.084 ms 16.210 ms 16.348 ms]

* ### *SuffixArray::\<u32>::from_stack(suffix_tree)*
        time:   [16.150 ms 16.260 ms 16.374 ms]

* ### *suffix_tree.lcp_rec::\<usize>()*
        time:   [6.7994 ms 6.8324 ms 6.8688 ms]

* ### *suffix_tree.lcp_rec::\<u32>()*
        time:   [6.5991 ms 6.6267 ms 6.6577 ms]

* ### *suffix_tree.lcp_stack::\<usize>()*
        time:   [6.2382 ms 6.2643 ms 6.2934 ms]

* ### *suffix_tree.lcp_stack::\<u32>()*
        time:   [6.2558 ms 6.2853 ms 6.3185 ms]