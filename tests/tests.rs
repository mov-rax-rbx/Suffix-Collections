use self::suff_collections::array::*;
use self::suff_collections::tree::*;
use rand::{distributions::Alphanumeric, prelude::*};
use suff_collections;

const TEST_ITERATIONS: usize = 256;

fn to_normal_line(line: &str) -> String {
    if line.as_bytes().last() == Some(&0) {
        line.to_owned()
    } else {
        core::str::from_utf8(
            &line
                .as_bytes()
                .iter()
                .chain(&[0])
                .map(|&x| x)
                .collect::<Vec<_>>(),
        )
        .unwrap()
        .to_owned()
    }
}

fn trust_find(line: &str, find: &str) -> Option<usize> {
    let line = to_normal_line(line);
    let true_find = line.find(find);
    true_find
}
fn trust_find_all(line: &str, find: &str) -> Vec<usize> {
    if find.is_empty() {
        return vec![];
    }
    let line = to_normal_line(line);
    if line.len() < find.len() {
        return vec![];
    }

    (0..line.len() - find.len())
        .into_iter()
        .filter(|&i| line.as_bytes()[i..i + find.len()].eq(find.as_bytes()))
        .collect()
}
fn trust_find_sa(line: &str, find: &str) -> Option<usize> {
    if find.is_empty() {
        return None;
    }
    let line = to_normal_line(line);
    let sa = SuffixArray::<usize>::new(&line);
    sa.iter()
        .find(|&&x| {
            line.as_bytes()[x..]
                .iter()
                .take(find.len())
                .eq(&find.as_bytes()[..])
        })
        .and_then(|&x| Some(x))
}
fn trust_suffix_array(line: &str) -> Vec<usize> {
    if line.is_empty() {
        return vec![];
    }
    let line = to_normal_line(line);
    let mut sa = (0..line.len()).map(|x| x).collect::<Vec<_>>();
    sa.sort_by(|&a, &b| line.as_bytes()[a..].cmp(&line.as_bytes()[b..]));
    sa
}

fn trust_lcp(line: &str) -> Vec<usize> {
    if line.is_empty() {
        return vec![];
    }
    let line = to_normal_line(line);
    let sa = trust_suffix_array(&line);
    let cmp = sa
        .iter()
        .zip(sa.iter().skip(1))
        .map(|(&i, &j)| {
            for ((i, &x), &y) in line.as_bytes()[i..]
                .iter()
                .enumerate()
                .zip(line.as_bytes()[j..].iter())
            {
                if x != y {
                    return i;
                }
            }
            core::cmp::min(line.len() - i, line.len() - j)
        })
        .collect::<Vec<_>>();
    [0].iter().chain(cmp.iter()).map(|&x| x).collect()
}

#[test]
fn test_build_ukkonen_and_find() {
    let mut rng = thread_rng();

    for _ in 0..TEST_ITERATIONS {
        let cnt = rng.gen_range(2..1024);

        let line = rng
            .clone()
            .sample_iter(&Alphanumeric)
            .take(cnt)
            .map(char::from)
            .collect::<String>();

        let mut start = rng.gen_range(0..cnt);
        let mut end = rng.gen_range(0..cnt);
        if start > end {
            core::mem::swap(&mut start, &mut end)
        }

        let find = &line[start..end];

        let res = SuffixTree::new(&line).find(find);
        assert_eq!(res, trust_find(&line, find));
    }

    let line = String::new();
    let find = &line[0..];
    let res = SuffixTree::new(&line).find(find);
    assert_eq!(res, trust_find(&line, find));
}

#[test]
fn test_suffix_tree_to_suffix_array() {
    let mut rng = thread_rng();

    for _ in 0..TEST_ITERATIONS {
        let cnt = rng.gen_range(1..1024);

        let line = rng
            .clone()
            .sample_iter(&Alphanumeric)
            .take(cnt)
            .map(char::from)
            .collect::<String>();

        let res = SuffixArray::<usize>::from(SuffixTree::new(&line))
            .suffix_array()
            .clone();
        assert_eq!(res, trust_suffix_array(&line));
    }

    let line = String::new();
    let res = SuffixArray::<usize>::from(SuffixTree::new(&line))
        .suffix_array()
        .clone();
    assert_eq!(res, trust_suffix_array(&line));
}

#[test]
fn test_suffix_array_to_suffix_tree_to_suffix_array() {
    let mut rng = thread_rng();

    for _ in 0..TEST_ITERATIONS {
        let cnt = rng.gen_range(1..1024);

        let line = rng
            .clone()
            .sample_iter(&Alphanumeric)
            .take(cnt)
            .map(char::from)
            .collect::<String>();

        let res = SuffixArray::<usize>::from(SuffixTree::from(SuffixArray::<usize>::new(&line)))
            .suffix_array()
            .clone();
        assert_eq!(res, trust_suffix_array(&line));
    }

    let line = String::new();
    let res = SuffixArray::<usize>::from(SuffixTree::from(SuffixArray::<usize>::new(&line)))
        .suffix_array()
        .clone();
    assert_eq!(res, trust_suffix_array(&line));
}

#[test]
fn test_build_suffix_array() {
    let mut rng = thread_rng();

    for _ in 0..TEST_ITERATIONS {
        let cnt = rng.gen_range(1..1024);

        let line = rng
            .clone()
            .sample_iter(&Alphanumeric)
            .take(cnt)
            .map(char::from)
            .collect::<String>();

        let res = SuffixArray::<usize>::new(&line).suffix_array().clone();
        assert_eq!(res, trust_suffix_array(&line));
    }

    let line = String::new();
    let res = SuffixArray::<usize>::new(&line).suffix_array().clone();
    assert_eq!(res, trust_suffix_array(&line));
}

#[test]
fn test_build_suffix_array_compress() {
    let mut rng = thread_rng();

    for _ in 0..TEST_ITERATIONS {
        let cnt = rng.gen_range(1..1024);

        let line = rng
            .clone()
            .sample_iter(&Alphanumeric)
            .take(cnt)
            .map(char::from)
            .collect::<String>();

        let res = SuffixArray::<usize>::new_compress(&line)
            .suffix_array()
            .clone();
        assert_eq!(res, trust_suffix_array(&line));
    }

    let line = String::new();
    let res = SuffixArray::<usize>::new_compress(&line)
        .suffix_array()
        .clone();
    assert_eq!(res, trust_suffix_array(&line));
}

#[test]
fn test_build_suffix_array_stack() {
    let mut rng = thread_rng();

    for _ in 0..TEST_ITERATIONS {
        let cnt = rng.gen_range(1..1024);

        let line = rng
            .clone()
            .sample_iter(&Alphanumeric)
            .take(cnt)
            .map(char::from)
            .collect::<String>();

        let res = SuffixArray::<usize>::new_stack(&line)
            .suffix_array()
            .clone();
        assert_eq!(res, trust_suffix_array(&line));
    }

    let line = String::new();
    let res = SuffixArray::<usize>::new_stack(&line)
        .suffix_array()
        .clone();
    assert_eq!(res, trust_suffix_array(&line));
}

#[test]
fn test_build_suffix_array_stack_compress() {
    let mut rng = thread_rng();

    for _ in 0..TEST_ITERATIONS {
        let cnt = rng.gen_range(1..1024);

        let line = rng
            .clone()
            .sample_iter(&Alphanumeric)
            .take(cnt)
            .map(char::from)
            .collect::<String>();

        let res = SuffixArray::<usize>::new_stack_compress(&line)
            .suffix_array()
            .clone();
        assert_eq!(res, trust_suffix_array(&line));
    }

    let line = String::new();
    let res = SuffixArray::<usize>::new_stack_compress(&line)
        .suffix_array()
        .clone();
    assert_eq!(res, trust_suffix_array(&line));
}

#[test]
fn test_build_suffix_array_and_find_big() {
    let mut rng = thread_rng();

    for _ in 0..TEST_ITERATIONS {
        let cnt = rng.gen_range(2..1024);

        let line = rng
            .clone()
            .sample_iter(&Alphanumeric)
            .take(cnt)
            .map(char::from)
            .collect::<String>();

        let mut start = rng.gen_range(0..cnt);
        let mut end = rng.gen_range(0..cnt);
        if start > end {
            core::mem::swap(&mut start, &mut end)
        }

        let find = &line[start..end];

        let sa = SuffixArray::<usize>::new(&line);
        let res = sa.find_big(&sa.lcp(), find);
        assert_eq!(res, trust_find_sa(&line, find));
    }

    let line = String::new();
    let find = &line[0..];
    let sa = SuffixArray::<usize>::new(&line);
    let res = sa.find_big(&sa.lcp(), find);
    assert_eq!(res, trust_find_sa(&line, find));
}

#[test]
fn test_build_suffix_array_and_find_all_big() {
    let mut rng = thread_rng();

    for _ in 0..TEST_ITERATIONS {
        let cnt = rng.gen_range(2..1024);

        let line = rng
            .clone()
            .sample_iter(&Alphanumeric)
            .take(cnt)
            .map(char::from)
            .collect::<String>();

        let mut start = rng.gen_range(0..cnt);
        let mut end = rng.gen_range(0..cnt);
        if start > end {
            core::mem::swap(&mut start, &mut end)
        }

        let find = &line[start..end];

        let sa = SuffixArray::<usize>::new(&line);
        let mut res = sa.find_all_big(&sa.lcp(), find).to_vec();
        let mut etalon = trust_find_all(&line, find);
        res.sort();
        etalon.sort();
        assert_eq!(res, etalon);
    }

    let line = String::new();
    let find = &line[0..];
    let sa = SuffixArray::<usize>::new(&line);
    let mut res = sa.find_all_big(&sa.lcp(), find).to_vec();
    let mut etalon = trust_find_all(&line, find);
    res.sort();
    etalon.sort();
    assert_eq!(res, etalon);
}

#[test]
fn test_build_suffix_array_and_find() {
    let mut rng = thread_rng();

    for _ in 0..TEST_ITERATIONS {
        let cnt = rng.gen_range(2..1024);

        let line = rng
            .clone()
            .sample_iter(&Alphanumeric)
            .take(cnt)
            .map(char::from)
            .collect::<String>();

        let mut start = rng.gen_range(0..cnt);
        let mut end = rng.gen_range(0..cnt);
        if start > end {
            core::mem::swap(&mut start, &mut end)
        }

        let find = &line[start..end];

        let sa = SuffixArray::<usize>::new(&line);
        let res = sa.find(find);
        assert_eq!(res, trust_find_sa(&line, find));
    }

    let line = String::new();
    let find = &line[0..];
    let sa = SuffixArray::<usize>::new(&line);
    let res = sa.find(find);
    assert_eq!(res, trust_find_sa(&line, find));
}

#[test]
fn test_build_suffix_array_and_find_all() {
    let mut rng = thread_rng();

    for _ in 0..TEST_ITERATIONS {
        let cnt = rng.gen_range(2..1024);

        let line = rng
            .clone()
            .sample_iter(&Alphanumeric)
            .take(cnt)
            .map(char::from)
            .collect::<String>();

        let mut start = rng.gen_range(0..cnt);
        let mut end = rng.gen_range(0..cnt);
        if start > end {
            core::mem::swap(&mut start, &mut end)
        }

        let find = &line[start..end];

        let sa = SuffixArray::<usize>::new(&line);
        let mut res = sa.find_all(find).to_vec();
        let mut etalon = trust_find_all(&line, find);
        res.sort();
        etalon.sort();
        assert_eq!(res, etalon);
    }

    let line = String::new();
    let find = &line[0..];
    let sa = SuffixArray::<usize>::new(&line);
    let mut res = sa.find_all(find).to_vec();
    let mut etalon = trust_find_all(&line, find);
    res.sort();
    etalon.sort();
    assert_eq!(res, etalon);
}

#[test]
fn test_build_suffix_tree_and_lcp() {
    let mut rng = thread_rng();

    for _ in 0..TEST_ITERATIONS {
        let cnt = rng.gen_range(1..1024);

        let line = rng
            .clone()
            .sample_iter(&Alphanumeric)
            .take(cnt)
            .map(char::from)
            .collect::<String>();

        let res = SuffixTree::new(&line).lcp::<usize>().owned().to_vec();
        assert_eq!(res, trust_lcp(&line));
    }

    let line = String::new();
    let res = SuffixTree::new(&line).lcp::<usize>().owned().to_vec();
    assert_eq!(res, trust_lcp(&line));
}

#[test]
fn test_build_online_suffix_tree_to_suffix_tree_and_lcp() {
    let mut rng = thread_rng();

    for _ in 0..TEST_ITERATIONS {
        let cnt = rng.gen_range(1..1024);

        let line = rng
            .clone()
            .sample_iter(&Alphanumeric)
            .take(cnt)
            .map(char::from)
            .collect::<String>();

        let mut ost = OnlineSuffixTree::new();
        ost.add(&line);
        let res = ost.finish().lcp::<usize>().owned().to_vec();
        assert_eq!(res, trust_lcp(&line));

        let mut ost = OnlineSuffixTree::new();
        for i in 0..line.len() {
            ost.add(&line[i..i + 1]);
        }
        let res = ost.finish().lcp::<usize>().owned().to_vec();
        assert_eq!(res, trust_lcp(&line));
    }

    let line = String::new();
    let mut ost = OnlineSuffixTree::new();
    ost.add(&line);
    let res = ost.finish().lcp::<usize>().owned().to_vec();
    assert_eq!(res, vec![0]);

    let mut ost = OnlineSuffixTree::new();
    for i in 0..line.len() {
        ost.add(&line[i..i + 1]);
    }
    let res = ost.finish().lcp::<usize>().owned().to_vec();
    assert_eq!(res, vec![0]);
}

#[test]
fn test_build_suffix_array_and_lcp() {
    let mut rng = thread_rng();

    for _ in 0..TEST_ITERATIONS {
        let cnt = rng.gen_range(1..1024);

        let line = rng
            .clone()
            .sample_iter(&Alphanumeric)
            .take(cnt)
            .map(char::from)
            .collect::<String>();

        let sa = SuffixArray::<usize>::new(&line);
        let res = sa.lcp().owned().to_vec();
        assert_eq!(res, trust_lcp(&line));
    }

    let line = String::new();
    let sa = SuffixArray::<usize>::new(&line);
    let res = sa.lcp().owned().to_vec();
    assert_eq!(res, trust_lcp(&line));
}

#[test]
#[should_panic]
fn test_suffix_array_overflow_1() {
    let line = "a".repeat(u8::MAX as usize);
    let _ = SuffixArray::<u8>::new(&line).lcp().owned().to_vec();
}
#[test]
#[should_panic]
fn test_suffix_array_overflow_2() {
    let line = "a".repeat(u8::MAX as usize);
    let _ = SuffixArray::<u8>::new_compress(&line)
        .lcp()
        .owned()
        .to_vec();
}
#[test]
#[should_panic]
fn test_suffix_array_overflow_3() {
    let line = "a".repeat(u8::MAX as usize);
    let _ = SuffixArray::<u8>::new_stack(&line).lcp().owned().to_vec();
}
#[test]
#[should_panic]
fn test_suffix_array_overflow_4() {
    let line = "a".repeat(u8::MAX as usize);
    let _ = SuffixArray::<u8>::new_stack_compress(&line)
        .lcp()
        .owned()
        .to_vec();
}
