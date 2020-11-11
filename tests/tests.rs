use suff_collections;
use self::suff_collections::tree::*;
use self::suff_collections::array::*;

fn to_normal_line(line: &str) -> String {
    if line.as_bytes().last() == Some(&0) {
        line.to_owned()
    } else {
        core::str::from_utf8(&line.as_bytes().iter().chain(&[0]).map(|&x| x).collect::<Vec<_>>()).unwrap().to_owned()
    }
}

fn trust_find(line: &str, find: &str) -> Option<usize> {
    let line = to_normal_line(line);
    let true_find = line.find(find);
    true_find
}
fn trust_find_all(line: &str, find: &str) -> Vec<usize> {
    let line = to_normal_line(line);
    if line.len() < find.len() {
        return vec![];
    }

    (0..line.len() - find.len()).into_iter()
        .filter(|&i| line.as_bytes()[i..i + find.len()].eq(find.as_bytes()))
        .collect()
}
fn trust_find_sa(line: &str, find: &str) -> Option<usize> {
    let line = to_normal_line(line);
    let sa = SuffixArray::new(&line);
    sa.iter()
        .find(|&&x| line.as_bytes()[x..].iter().take(find.len()).eq(&find.as_bytes()[..]))
        .and_then(|&x| Some(x))
}
fn trust_suffix_array(line: &str) -> Vec<usize> {
    let line = to_normal_line(line);
    let mut sa = (0..line.len()).map(|x| x).collect::<Vec<_>>();
    sa.sort_by(|&a, &b| line.as_bytes()[a..].cmp(&line.as_bytes()[b..]));
    sa
}

fn trust_lcp(line: &str) -> Vec<usize> {
    let line = to_normal_line(line);
    let sa = trust_suffix_array(&line);
    let cmp = sa.iter().zip(sa.iter().skip(1))
        .map(|(&i, &j)| {
            for ((i, &x), &y) in line.as_bytes()[i..].iter().enumerate().zip(line.as_bytes()[j..].iter()) {
                if x != y {
                    return i;
                }
            }
            core::cmp::min(line.len() - i, line.len() - j)
        }).collect::<Vec<_>>();
    [0].iter().chain(cmp.iter()).map(|&x| x).collect()
}

#[test]
fn ukkonen_test_find_1() {
    let line = "ocaocacao\0";
    let find = "aca";
    let res = SuffixTree::new(line).find(find);
    assert_eq!(res, trust_find(line, find));
}
#[test]
fn ukkonen_test_find_2() {
    let line = "aocacocaoabacaca\0";
    let find = "aoabacac";
    let res = SuffixTree::new(line).find(find);
    assert_eq!(res, trust_find(line, find));
}
#[test]
fn ukkonen_test_find_utf8_1() {
    let line = "色は匂へど 散りぬるを\0";
    let find = "ど 散りぬ";
    let res = SuffixTree::new(line).find(find);
    assert_eq!(res, trust_find(line, find));
}
#[test]
fn ukkonen_test_find_utf8_2() {
    let line = "色は匂へど 散りぬるを\0";
    let find = "ã© æ£ãã¬";
    let res = SuffixTree::new(line).find(find);
    assert_eq!(res, trust_find(line, find));
}
#[test]
fn ukkonen_test_find_utf8_3() {
    let line = "АБВГДЕЖЗИЙКЛМНОПРСТУФХЦЧШЩЪЫЬЭЮЯЗИ
    ЙКЛМНОПРСТУФХЦЧШЩЪЫЬЭЮЯЗИЙКЛМНОПРСТУФХЦЧШЩЪЫЬЭЮЯЗИЙК
    ЗИЙКЛМНОПРСТУФХЦЧШЩЪЫЬЭЮЯSDDDSPЛМНОПРСТУФХЦЧШЩЪЫЬЭЮЯ\0";
    let find = "МНОПРСТУФХЦЧШЩЪЫ";
    let res = SuffixTree::new(line).find(find);
    assert_eq!(res, trust_find(line, find));
}
#[test]
fn ukkonen_test_find_utf8_4() {
    let line = "АБВГДЕЖЗИЙКЛМНОПРСТУФХЦЧШЩЪЫЬЭЮЯЗИ
    ЙКЛМНОПРСТУФХЦЧШЩЪЫЬЭЮЯЗИЙКЛМНОПРСТУФХЦЧШЩЪЫЬЭЮЯЗИЙК
    ЗИЙКЛМНОПРСТУФХЦЧШЩЪЫЬЭЮЯSDDDSPЛМНОПРСТУФХЦЧШЩЪЫЬЭЮЯ\0";
    let find = &line.as_bytes().iter().map(|&x| x as char).collect::<String>();
    let res = SuffixTree::new(line).find(find);
    assert_eq!(res, trust_find(line, find));
}
#[test]
fn ukkonen_test_find_utf8_5() {
    let line = "АБВГДЕЖЗИЙКЛМНОПРСТУФХЦЧШЩЪЫЬЭЮЯЗИ
    ЙКЛМНОПРСТУФХЦЧШЩЪЫЬЭЮЯЗИЙКЛМНОПРСТУФХЦЧШЩЪЫЬЭЮЯЗИЙК
    ЗИЙКЛМНОПРСТУФХЦЧШЩЪЫЬЭЮЯSDDDSPЛМНОПРСТУФХЦЧШЩЪЫЬЭЮЯ\0";
    let find = &line.escape_unicode().collect::<String>();
    let res = SuffixTree::new(line).find(find);
    assert_eq!(res, trust_find(line, find));
}
#[test]
fn suffix_array_test_find_utf8_1() {
    let line = "色は匂へど 散りぬるを\0";
    let find = "ど 散りぬ";
    let sa = SuffixArray::new(line);
    let res = sa.find(find);
    assert_eq!(res, trust_find_sa(line, find));
}
#[test]
fn suffix_array_test_find_utf8_2() {
    let line = "色は匂へど 散りぬるを\0";
    let find = "ã© æ£ãã¬";
    let sa = SuffixArray::new(line);
    let res = sa.find(find);
    assert_eq!(res, trust_find_sa(line, find));
}
#[test]
fn suffix_array_test_find_utf8_3() {
    let line = "АБВГДЕЖЗИЙКЛМНОПРСТУФХЦЧШЩЪЫЬЭЮЯЗИ
    ЙКЛМНОПРСТУФХЦЧШЩЪЫЬЭЮЯЗИЙКЛМНОПРСТУФХЦЧШЩЪЫЬЭЮЯЗИЙК
    ЗИЙКЛМНОПРСТУФХЦЧШЩЪЫЬЭЮЯSDDDSPЛМНОПРСТУФХЦЧШЩЪЫЬЭЮЯ\0";
    let find = "МНОПРСТУФХЦЧШЩЪЫ";
    let sa = SuffixArray::new(line);
    let res = sa.find(find);
    assert_eq!(res, trust_find_sa(line, find));
}
#[test]
fn suffix_array_test_find_utf8_4() {
    let line = "АБВГДЕЖЗИЙКЛМНОПРСТУФХЦЧШЩЪЫЬЭЮЯЗИ
    ЙКЛМНОПРСТУФХЦЧШЩЪЫЬЭЮЯЗИЙКЛМНОПРСТУФХЦЧШЩЪЫЬЭЮЯЗИЙК
    ЗИЙКЛМНОПРСТУФХЦЧШЩЪЫЬЭЮЯSDDDSPЛМНОПРСТУФХЦЧШЩЪЫЬЭЮЯ\0";
    let find = &line.as_bytes().iter().map(|&x| x as char).collect::<String>();
    let sa = SuffixArray::new(line);
    let res = sa.find(find);
    assert_eq!(res, trust_find_sa(line, find));
}
#[test]
fn suffix_array_test_find_utf8_5() {
    let line = "АБВГДЕЖЗИЙКЛМНОПРСТУФХЦЧШЩЪЫЬЭЮЯЗИ
    ЙКЛМНОПРСТУФХЦЧШЩЪЫЬЭЮЯЗИЙКЛМНОПРСТУФХЦЧШЩЪЫЬЭЮЯЗИЙК
    ЗИЙКЛМНОПРСТУФХЦЧШЩЪЫЬЭЮЯSDDDSPЛМНОПРСТУФХЦЧШЩЪЫЬЭЮЯ\0";
    let find = &line.escape_unicode().collect::<String>();
    let sa = SuffixArray::new(line);
    let res = sa.find(find);
    assert_eq!(res, trust_find_sa(line, find));
}
#[test]
fn ukkonen_test_big_find() {
    let line =
"ccacaoca______caocaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacacafgdfvdfvdcsdfsdfsd
ocacaoccacaocacaoacaoacaaocacaocacocacaoaaaaaaaabaaaaaacacaocacaocacaocacaocacaocacaocacaocacao
aocacaocacuuuuuuyyyyyyyyyyyuuuuuuuuuuyyyyyyyyysssssssssssssuuocacaocacaocacaocacaocacaocacaocac
aocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacwqe23\0";
    let find = "cacaoca____";
    let res = SuffixTree::new(line).find(find);
    assert_eq!(res, trust_find(line, find));

    let find = "vdfvdcsdfsdfsd
ocacaoccacaocacaoacaoacaaocacaocacocacaoaaaaaaaabaaaaaacacaocacaocaca";
    let res = SuffixTree::new(line).find(find);
    assert_eq!(res, trust_find(line, find));

    let find = "yyuuuuuuuuuuyyyyyyyyysssssssssssssuuocacaocacaocacaocacaocacaocacaocac
aocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaoca";
    let res = SuffixTree::new(line).find(find);
    assert_eq!(res, trust_find(line, find));

    let find = "cacaocacaocacaocacaocacaocacwqe23";
    let res = SuffixTree::new(line).find(find);
    assert_eq!(res, trust_find(line, find));

    let find = "hhhhhhghggggerrrrrrrrrrrr";
    let res = SuffixTree::new(line).find(find);
    assert_eq!(res, trust_find(line, find));

    let find = "3";
    let res = SuffixTree::new(line).find(find);
    assert_eq!(res, trust_find(line, find));
}
#[test]
fn ukkonen_test_find_3() {
    let line = "amxcbvmcxbv,njsdfaocacaocacuyuysuuocacasldjfhjsm.c,o\0";
    let find = "cacuyuysuuoca";
    let res = SuffixTree::new(line).find(find);
    assert_eq!(res, trust_find(line, find));
}

#[test]
fn test_to_suffix_array_rec_1() {
    let line =
    "ccacaoca______caocaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacacafgdfvdfvdcsdfsdfsd
    ocacaoccacaocacaoacaoacaaocacaocacocacaoaaaaaaaabaaaaaacacaocacaocacaocacaocacaocacaocacaocacao
    aocacaocacuuuuuuyyyyyyyyyyyuuuuuuuuuuyyyyyyyyysssssssssssssuuocacaocacaocacaocacaocacaocacaocac
    aocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacwqe23\0";

    let res = SuffixArray::from_rec(SuffixTree::new(line)).suffix_array().clone();
    assert_eq!(res, trust_suffix_array(line));
}

#[test]
fn test_to_suffix_array_stack_2() {
    let line =
    "ccacaoca______caocaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacacafgdfvdfvdcsdfsdfsd
    ocacaoccacaocacaoacaoacaaocacaocacocacaoaaaaaaaabaaaaaacacaocacaocacaocacaocacaocacaocacaocacao
    aocacaocacuuuuuuyyyyyyyyyyyuuuuuuuuuuyyyyyyyyysssssssssssssuuocacaocacaocacaocacaocacaocacaocac
    aocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacwqe23\0";

    let res = SuffixArray::from_stack(SuffixTree::new(line)).suffix_array().clone();
    assert_eq!(res, trust_suffix_array(line));
}

#[test]
fn test_to_suffix_tree_1() {
    let line =
    "ccacaoca______caocaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacacafgdfvdfvdcsdfsdfsd
    ocacaoccacaocacaoacaoacaaocacaocacocacaoaaaaaaaabaaaaaacacaocacaocacaocacaocacaocacaocacaocacao
    aocacaocacuuuuuuyyyyyyyyyyyuuuuuuuuuuyyyyyyyyysssssssssssssuuocacaocacaocacaocacaocacaocacaocac
    aocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacwqe23\0";

    let res = SuffixArray::from_stack(
        SuffixTree::from(SuffixArray::new(line))
    ).suffix_array().clone();
    assert_eq!(res, trust_suffix_array(line));
}

#[test]
fn test_to_suffix_tree_2() {
    let line = "ccamxcbvmcxbv,njsdfaocacaocacuyuysuuocacasldjfhjsm.c,o\0";

    let res = SuffixArray::from_stack(
        SuffixTree::from(SuffixArray::new(line))
    ).suffix_array().clone();
    assert_eq!(res, trust_suffix_array(line));
}

#[test]
fn test_to_suffix_tree_3() {
    let line = "АБВГДЕЖЗИЙКЛМНОПРСТУФХЦЧШЩЪЫЬЭЮЯЗИ
    ЙКЛМНОПРСТУФХЦЧШЩЪЫЬЭЮЯЗИЙКЛМНОПРСТУФХЦЧШЩЪЫЬЭЮЯЗИЙК
    ЗИЙКЛМНОПРСТУФХЦЧШЩЪЫЬЭЮЯSDDDSPЛМНОПРСТУФХЦЧШЩЪЫЬЭЮЯ\0";

    let res = SuffixArray::from_stack(
        SuffixTree::from(SuffixArray::new(line))
    ).suffix_array().clone();
    assert_eq!(res, trust_suffix_array(line));
}

#[test]
fn test_to_suffix_tree_4() {
    let line = "I'll make the big change. First of all though, I've got to get up, my train leaves at five.\" 
    And he looked over at the alarm clock, ticking on the chest of drawers. \"God in Heaven!\" he thought. It was 
    half past six and the hands were quietly moving forwards, it was even later than half past, more like quarter to 
    seven. Had the alarm clock not rung? He could see from the bed that it had been set for four o'clock as it should 
    have been; it certainly must have rung. Yes, but was it possible to quietly sleep through that furniture-rattling 
    noise? True, he had not slept peacefully, but probably all the more deeply because of that. What should he do now? 
    The next train went at seven; if he were to catch that he would have to rush like mad and the collection of samples 
    was still not packed, and he did not at all feel particularly fresh and lively. And even if he did catch the train 
    he would not avoid his boss's anger as the office assistant would have been there to see the five o'clock train go, 
    he would have put in his report about Gregor's not being there a long time ago. The office assistant was the boss's 
    man, spineless, and with no understanding. What about if he reported sick? But that would be extremely strained and 
    suspicious as in fifteen years of service Gregor had never once yet been ill. H";

    let res = SuffixArray::from_stack(
        SuffixTree::from(SuffixArray::new(line))
    ).suffix_array().clone();
    assert_eq!(res, trust_suffix_array(line));
}

#[test]
fn test_suffix_array_1() {
    let line =
    "ccacaoca______caocaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacacafgdfvdfvdcsdfsdfsd
    ocacaoccacaocacaoacaoacaaocacaocacocacaoaaaaaaaabaaaaaacacaocacaocacaocacaocacaocacaocacaocacao
    aocacaocacuuuuuuyyyyyyyyyyyuuuuuuuuuuyyyyyyyyysssssssssssssuuocacaocacaocacaocacaocacaocacaocac
    aocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacwqe23\0";

    let res = SuffixArray::new(line).suffix_array().clone();
    assert_eq!(res, trust_suffix_array(line));
}

#[test]
fn test_suffix_array_2() {
    let line = "mmiissiissiippii\0";
    let res = SuffixArray::new(line).suffix_array().clone();
    assert_eq!(res, trust_suffix_array(line));
}

#[test]
fn test_suffix_array_3() {
    let line = "ACGTGCCTAGCCTACCGTGCC\0";
    let res = SuffixArray::new(line).suffix_array().clone();
    assert_eq!(res, trust_suffix_array(line));
}

#[test]
fn test_suffix_array_4() {
    let line =
    "ccacaoca______caocaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacacafgdfvdfvdcsdfsdfsd
    ocacaoccacaocacaoacaoacaaocacaocacocacaoaaaaaaaabaaaaaacacaocacaocacaocacaocacaocacaocacaocacao
    aocacaocacuuuuuuyyyyyyyyyyyuuuuuuuuuuyyyyyyyyysssssssssssssuuocacaocacaocacaocacaocacaocacaocac
    aocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacwqe23\0";

    let res = SuffixArray::new(line).suffix_array().clone();
    assert_eq!(res, trust_suffix_array(line));
}

#[test]
fn test_suffix_array_stack_1() {
    let line =
    "ccacaoca______caocaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacacafgdfvdfvdcsdfsdfsd
    ocacaoccacaocacaoacaoacaaocacaocacocacaoaaaaaaaabaaaaaacacaocacaocacaocacaocacaocacaocacaocacao
    aocacaocacuuuuuuyyyyyyyyyyyuuuuuuuuuuyyyyyyyyysssssssssssssuuocacaocacaocacaocacaocacaocacaocac
    aocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacwqe23\0";

    let res = SuffixArray::new_stack(line).suffix_array().clone();
    assert_eq!(res, trust_suffix_array(line));
}

#[test]
fn test_suffix_array_stack_2() {
    let line = "mmiissiissiippii\0";

    let res = SuffixArray::new_stack(line).suffix_array().clone();
    assert_eq!(res, trust_suffix_array(line));
}

#[test]
fn test_suffix_array_stack_3() {
    let line = "ACGTGCCTAGCCTACCGTGCC\0";

    let res = SuffixArray::new_stack(line).suffix_array().clone();
    assert_eq!(res, trust_suffix_array(line));
}

#[test]
fn test_suffix_array_stack_4() {
    let line =
    "ccacaoca______caocaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacacafgdfvdfvdcsdfsdfsd
    ocacaoccacaocacaoacaoacaaocacaocacocacaoaaaaaaaabaaaaaacacaocacaocacaocacaocacaocacaocacaocacao
    aocacaocacuuuuuuyyyyyyyyyyyuuuuuuuuuuyyyyyyyyysssssssssssssuuocacaocacaocacaocacaocacaocacaocac
    aocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacwqe23\0";

    let res = SuffixArray::new_stack(line).suffix_array().clone();
    assert_eq!(res, trust_suffix_array(line));
}

#[test]
fn test_find_suffix_array_big_find_big() {
    let line =
"ccacaoca______caocaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacacafgdfvdfvdcsdfsdfsd
ocacaoccacaocacaoacaoacaaocacaocacocacaoaaaaaaaabaaaaaacacaocacaocacaocacaocacaocacaocacaocacao
aocacaocacuuuuuuyyyyyyyyyyyuuuuuuuuuuyyyyyyyyysssssssssssssuuocacaocacaocacaocacaocacaocacaocac
aocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacwqe23\0";

    let sa = SuffixArray::new(line);

    let find = "cacaoca____";
    let lcp = sa.lcp();
    let res = sa.find_big(&lcp, find);
    assert_eq!(res, trust_find_sa(line, find));

    let find = "vdfvdcsdfsdfsd
ocacaoccacaocacaoacaoacaaocacaocacocacaoaaaaaaaabaaaaaacacaocacaocaca";
    let res = sa.find_big(&lcp, find);
    assert_eq!(res, trust_find_sa(line, find));

    let find = "yyuuuuuuuuuuyyyyyyyyysssssssssssssuuocacaocacaocacaocacaocacaocacaocac
aocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaoca";
    let res = sa.find_big(&lcp, find);
    assert_eq!(res, trust_find_sa(line, find));

    let find = "cacaocacaocacaocacaocacaocacwqe23";
    let res = sa.find_big(&lcp, find);
    assert_eq!(res, trust_find_sa(line, find));

    let find = "hhhhhhghggggerrrrrrrrrrrr";
    let res = sa.find_big(&lcp, find);
    assert_eq!(res, trust_find_sa(line, find));

    let find = "3";
    let res = sa.find_big(&lcp, find);
    assert_eq!(res, trust_find_sa(line, find));
}

#[test]
fn test_find_big_suffix_array_1() {
    let line =
"ccacaoca______caocaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacacafgdfvdfvdcsdfsdfsd
ocacaoccacaocacaoacaoacaaocacaocacocacaoaaaaaaaabaaaaaacacaocacaocacaocacaocacaocacaocacaocacao
aocacaocacuuuuuuyyyyyyyyyyyuuuuuuuuuuyyyyyyyyysssssssssssssuuocacaocacaocacaocacaocacaocacaocac
aocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacwqe23\0";
    let find = "yyuuuuuuuuuuyyyyyyyyysssssssssssssuuocacaocacaocacaocacaocacaocacaocac
aocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaoca";

    let sa = SuffixArray::new(line);
    let res = sa.find_big(&sa.lcp(), find);
    assert_eq!(res, trust_find_sa(line, find));
}

#[test]
fn test_find_suffix_array_1() {
    let line =
"ccacaoca______caocaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacacafgdfvdfvdcsdfsdfsd
ocacaoccacaocacaoacaoacaaocacaocacocacaoaaaaaaaabaaaaaacacaocacaocacaocacaocacaocacaocacaocacao
aocacaocacuuuuuuyyyyyyyyyyyuuuuuuuuuuyyyyyyyyysssssssssssssuuocacaocacaocacaocacaocacaocacaocac
aocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacwqe23\0";
    let find = "yyuuuuuuuuuuyyyyyyyyysssssssssssssuuocacaocacaocacaocacaocacaocacaocac
aocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaoca";

    let sa = SuffixArray::new(line);
    let res = sa.find(find);
    assert_eq!(res, trust_find_sa(line, find));
}

#[test]
fn test_find_suffix_array_big_find() {
    let line =
"ccacaoca______caocaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacacafgdfvdfvdcsdfsdfsd
ocacaoccacaocacaoacaoacaaocacaocacocacaoaaaaaaaabaaaaaacacaocacaocacaocacaocacaocacaocacaocacao
aocacaocacuuuuuuyyyyyyyyyyyuuuuuuuuuuyyyyyyyyysssssssssssssuuocacaocacaocacaocacaocacaocacaocac
aocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacwqe23\0";

    let sa = SuffixArray::new(line);

    let find = "cacaoca____";
    let res = sa.find(find);
    assert_eq!(res, trust_find_sa(line, find));

    let find = "vdfvdcsdfsdfsd
ocacaoccacaocacaoacaoacaaocacaocacocacaoaaaaaaaabaaaaaacacaocacaocaca";
    let res = sa.find(find);
    assert_eq!(res, trust_find_sa(line, find));

    let find = "yyuuuuuuuuuuyyyyyyyyysssssssssssssuuocacaocacaocacaocacaocacaocacaocac
aocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaoca";
    let res = sa.find(find);
    assert_eq!(res, trust_find_sa(line, find));

    let find = "cacaocacaocacaocacaocacaocacwqe23";
    let res = sa.find(find);
    assert_eq!(res, trust_find_sa(line, find));

    let find = "hhhhhhghggggerrrrrrrrrrrr";
    let res = sa.find(find);
    assert_eq!(res, trust_find_sa(line, find));

    let find = "3";
    let res = sa.find(find);
    assert_eq!(res, trust_find_sa(line, find));
}

#[test]
fn test_find_all_suffix_array_1() {
    let line =
"ccacaoca______caocaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacacafgdfvdfvdcsdfsdfsd
ocacaoccacaocacaoacaoacaaocacaocacocacaoaaaaaaaabaaaaaacacaocacaocacaocacaocacaocacaocacaocacao
aocacaocacuuuuuuyyyyyyyyyyyuuuuuuuuuuyyyyyyyyysssssssssssssuuocacaocacaocacaocacaocacaocacaocac
aocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacwqe23\0";
    let find = "caocacaoc";

    let sa = SuffixArray::new(line);
    let mut res = sa.find_all(find).to_vec();
    let mut etalon = trust_find_all(line, find);
    res.sort();
    etalon.sort();
    assert_eq!(res, etalon);
}

#[test]
fn test_find_all_suffix_array_2() {
    let line =
"ccacaoca______caocaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacacafgdfvdfvdcsdfsdfsd
ocacaoccacaocacaoacaoacaaocacaocacocacaoaaaaaaaabaaaaaacacaocacaocacaocacaocacaocacaocacaocacao
aocacaocacuuuuuuyyyyyyyyyyyuuuuuuuuuuyyyyyyyyysssssssssssssuuocacaocacaocacaocacaocacaocacaocac
aocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacwqe23\0";
    let find = "o";

    let sa = SuffixArray::new(line);
    let mut res = sa.find_all(find).to_vec();
    let mut etalon = trust_find_all(line, find);
    res.sort();
    etalon.sort();
    assert_eq!(res, etalon);
}

#[test]
fn test_find_all_big_suffix_array_1() {
    let line =
"ccacaoca______caocaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacacafgdfvdfvdcsdfsdfsd
ocacaoccacaocacaoacaoacaaocacaocacocacaoaaaaaaaabaaaaaacacaocacaocacaocacaocacaocacaocacaocacao
aocacaocacuuuuuuyyyyyyyyyyyuuuuuuuuuuyyyyyyyyysssssssssssssuuocacaocacaocacaocacaocacaocacaocac
aocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacwqe23\0";
    let find = "caocacaoc";

    let sa = SuffixArray::new(line);
    let mut res = sa.find_all_big(&sa.lcp(), find).to_vec();
    let mut etalon = trust_find_all(line, find);
    res.sort();
    etalon.sort();
    assert_eq!(res, etalon);
}

#[test]
fn test_find_all_big_suffix_array_2() {
    let line =
"ccacaoca______caocaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacacafgdfvdfvdcsdfsdfsd
ocacaoccacaocacaoacaoacaaocacaocacocacaoaaaaaaaabaaaaaacacaocacaocacaocacaocacaocacaocacaocacao
aocacaocacuuuuuuyyyyyyyyyyyuuuuuuuuuuyyyyyyyyysssssssssssssuuocacaocacaocacaocacaocacaocacaocac
aocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacwqe23\0";
    let find = "o";

    let sa = SuffixArray::new(line);
    let mut res = sa.find_all_big(&sa.lcp(), find).to_vec();
    let mut etalon = trust_find_all(line, find);
    res.sort();
    etalon.sort();
    assert_eq!(res, etalon);
}

#[test]
fn test_lcp_rec_suffix_tree_1() {
    let line = "ocaocacao\0";
    let res = SuffixTree::new(line).lcp_rec().owned().to_vec();
    assert_eq!(res, trust_lcp(line));
}
#[test]
fn test_lcp_rec_suffix_tree_2() {
    let line = "ACGTGCCTAGCCTACCGTGCC\0";
    let res = SuffixTree::new(line).lcp_rec().owned().to_vec();
    assert_eq!(res, trust_lcp(line));
}
#[test]
fn test_lcp_rec_suffix_tree_3() {
    let line = "mmiissiissiippii\0";
    let res = SuffixTree::new(line).lcp_rec().owned().to_vec();
    assert_eq!(res, trust_lcp(line));
}
#[test]
fn test_lcp_rec_suffix_tree_4() {
    let line =
    "ccacaoca______caocaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacacafgdfvdfvdcsdfsdfsd
    ocacaoccacaocacaoacaoacaaocacaocacocacaoaaaaaaaabaaaaaacacaocacaocacaocacaocacaocacaocacaocacao
    aocacaocacuuuuuuyyyyyyyyyyyuuuuuuuuuuyyyyyyyyysssssssssssssuuocacaocacaocacaocacaocacaocacaocac
    aocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacwqe23\0";
    let res = SuffixTree::new(line).lcp_rec().owned().to_vec();
    assert_eq!(res, trust_lcp(line));
}
#[test]
fn test_lcp_rec_suffix_tree_5() {
    let line = "amxcbvmcxbv,njsdfaocacaocacuyuysuuocacasldjfhjsm.c,o\0";
    let res = SuffixTree::new(line).lcp_rec().owned().to_vec();
    assert_eq!(res, trust_lcp(line));
}

#[test]
fn test_lcp_stack_suffix_tree_1() {
    let line = "ocaocacao\0";
    let res = SuffixTree::new(line).lcp_stack().owned().to_vec();
    assert_eq!(res, trust_lcp(line));
}
#[test]
fn test_lcp_stack_suffix_tree_2() {
    let line = "ACGTGCCTAGCCTACCGTGCC\0";
    let res = SuffixTree::new(line).lcp_stack().owned().to_vec();
    assert_eq!(res, trust_lcp(line));
}
#[test]
fn test_lcp_stack_suffix_tree_3() {
    let line = "mmiissiissiippii\0";
    let res = SuffixTree::new(line).lcp_stack().owned().to_vec();
    assert_eq!(res, trust_lcp(line));
}
#[test]
fn test_lcp_stack_suffix_tree_4() {
    let line =
    "ccacaoca______caocaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacacafgdfvdfvdcsdfsdfsd
    ocacaoccacaocacaoacaoacaaocacaocacocacaoaaaaaaaabaaaaaacacaocacaocacaocacaocacaocacaocacaocacao
    aocacaocacuuuuuuyyyyyyyyyyyuuuuuuuuuuyyyyyyyyysssssssssssssuuocacaocacaocacaocacaocacaocacaocac
    aocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacwqe23\0";
    let res = SuffixTree::new(line).lcp_stack().owned().to_vec();
    assert_eq!(res, trust_lcp(line));
}
#[test]
fn test_lcp_stack_suffix_tree_5() {
    let line = "amxcbvmcxbv,njsdfaocacaocacuyuysuuocacasldjfhjsm.c,o\0";
    let res = SuffixTree::new(line).lcp_stack().owned().to_vec();
    assert_eq!(res, trust_lcp(line));
}

#[test]
fn test_lcp_suffix_array_1() {
    let line = "ocaocacao\0";
    let sa = SuffixArray::new(line);
    let res = sa.lcp().owned().to_vec();
    assert_eq!(res, trust_lcp(line));
}
#[test]
fn test_lcp_suffix_array_2() {
    let line = "ACGTGCCTAGCCTACCGTGCC\0";
    let sa = SuffixArray::new(line);
    let res = sa.lcp().owned().to_vec();
    assert_eq!(res, trust_lcp(line));
}
#[test]
fn test_lcp_suffix_array_3() {
    let line = "mmiissiissiippii";
    let sa = SuffixArray::new(line);
    let res = sa.lcp().owned().to_vec();
    assert_eq!(res, trust_lcp(line));
}
#[test]
fn test_lcp_suffix_array_4() {
    let line =
    "ccacaoca______caocaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacacafgdfvdfvdcsdfsdfsd
    ocacaoccacaocacaoacaoacaaocacaocacocacaoaaaaaaaabaaaaaacacaocacaocacaocacaocacaocacaocacaocacao
    aocacaocacuuuuuuyyyyyyyyyyyuuuuuuuuuuyyyyyyyyysssssssssssssuuocacaocacaocacaocacaocacaocacaocac
    aocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacaocacwqe23";
    let sa = SuffixArray::new(line);
    let res = sa.lcp().owned().to_vec();
    assert_eq!(res, trust_lcp(line));
}
#[test]
fn test_lcp_suffix_array_5() {
    let line = "amxcbvmcxbv,njsdfaocacaocacuyuysuuocacasldjfhjsm.c,o";
    let sa = SuffixArray::new(line);
    let res = sa.lcp().owned().to_vec();
    assert_eq!(res, trust_lcp(line));
}

#[test]
fn test_lcp_suffix_array_utf8_1() {
    let line = "色は匂へど 散りぬるを\0";
    let res = SuffixArray::new(line).lcp().owned().to_vec();
    assert_eq!(res, trust_lcp(line));
}

#[test]
fn test_lcp_suffix_array_utf8_2() {
    let line = "АБВГДЕЖЗИЙКЛМНОПРСТУФХЦЧШЩЪЫЬЭЮЯЗИ
    ЙКЛМНОПРСТУФХЦЧШЩЪЫЬЭЮЯЗИЙКЛМНОПРСТУФХЦЧШЩЪЫЬЭЮЯЗИЙК
    ЗИЙКЛМНОПРСТУФХЦЧШЩЪЫЬЭЮЯSDDDSPЛМНОПРСТУФХЦЧШЩЪЫЬЭЮЯ\0";
    let res = SuffixArray::new(line).lcp().owned().to_vec();
    assert_eq!(res, trust_lcp(line));
}