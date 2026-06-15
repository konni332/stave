#![allow(dead_code)]
use stave_macros::builder;

#[builder]
struct Cache<'a, T: Clone, const N: usize> {
    #[stave(required)]
    items: [T; N],
    #[stave(required)]
    name: &'a str,
    note: String,
}

fn main() {}
