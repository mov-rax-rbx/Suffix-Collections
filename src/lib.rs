extern crate alloc;

pub mod tree;
pub mod array;

pub mod lcp;


use alloc::borrow::Cow;
use core::str;
fn canonic_word<'t>(word: &'t str) -> Cow<'t, str> {
    if word.as_bytes().last() == Some(&0) {
        Cow::from(word)
    } else {
        Cow::from(str::from_utf8(&word.as_bytes().iter().chain(&[0]).map(|&x| x).collect::<Vec<_>>()).unwrap().to_owned())
    }
}