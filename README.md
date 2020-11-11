# Suffix Collections

Fast realization of suffix array and suffix tree for substring search, longest common prefix array (lcp array).

## Example
* **SuffixTree**
```rust
    use suff_collections::tree::*;

    // let word = "Some word";
    let word: &str = "Some word\0";
    let find: &str = "word";

    // construct suffix tree
    let st: SuffixTree = SuffixTree::new(word);

    // finds the entry position of the line 'find' in 'word'
    let res: Option<usize> = st.find(find);

    // construct lcp
    // lcp[i] = max_pref(sa[i], sa[i - 1]) && lcp.len() == sa.len()
    // let lcp: LCP = st.lcp_stack();
    let lcp: LCP = st.lcp_rec();

    // convert suffix tree to suffix array
    // let sa: SuffixArray = SuffixArray::from_stack(st);
    let sa: SuffixArray = SuffixArray::from_rec(st);
```

* **SuffixArray**
```rust
    use suff_collections::array::*;

    // let word = "Some word";
    let word: &str = "Some word\0";
    let find: &str = "word";

    // construct suffix array
    // let sa = SuffixArray::new_stack(word);
    let sa: SuffixArray = SuffixArray::new(word);

    // construct lcp
    // lcp[i] = max_pref(sa[i], sa[i - 1]) && lcp.len() == sa.len()
    let lcp: LCP = sa.lcp();

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
    let st: SuffixTree = SuffixTree::new(sa);
```
All construction and search work for O(n). For the suffix tree implementation the [Ukkonen algorithm][2] is taken and for the suffix array implementation the [SA-IS algorithm][1] is taken.

[1]: https://www.researchgate.net/profile/Daricks_Wai_Hong_Chan/publication/221577802_Linear_Suffix_Array_Construction_by_Almost_Pure_Induced-Sorting/links/00b495318a21ba484f000000/Linear-Suffix-Array-Construction-by-Almost-Pure-Induced-Sorting.pdf?origin=publication_detail

[2]: https://web.stanford.edu/~mjkay/gusfield.pdf

# Some benches *(thank [Criterion](https://github.com/bheisler/criterion.rs))*

* ### *Suffix array to lcp*
        time:   [433.13 us 434.87 us 436.63 us]

* ### *Suffix array build*
        time:   [2.3837 ms 2.4029 ms 2.4232 ms]

* ### *Suffix array build stack*
        time:   [2.4006 ms 2.4161 ms 2.4321 ms]

* ### *Suffix array find O(|find| * log(|word|))*
        time:   [4.0171 us 4.0439 us 4.0721 us]

* ### *Suffix array find all O(|find| * log(|word|))*
        time:   [4.0105 us 4.0350 us 4.0611 us]

* ### *Suffix array find O(|word|)*
        time:   [20.475 us 20.869 us 21.263 us]

* ### *Suffix array find all O(|word|)*
        time:   [19.867 us 20.178 us 20.522 us]

* ### *Suffix array to suffix tree build*
        time:   [13.355 ms 13.496 ms 13.643 ms]

* ### *Suffix tree build Ukkonen*
        time:   [15.570 ms 15.723 ms 15.887 ms]

* ### *Suffix tree find*
        time:   [23.576 us 23.771 us 23.973 us]

* ### *Suffix tree to suffix array rec build*
        time:   [16.351 ms 16.479 ms 16.617 ms]

* ### *Suffix tree to suffix array stack build*
        time:   [16.209 ms 16.326 ms 16.446 ms]

* ### *Suffix tree to lcp rec*
        time:   [6.3750 ms 6.4100 ms 6.4482 ms]

* ### *Suffix tree to lcp stack*
        time:   [5.9856 ms 6.0216 ms 6.0601 ms]