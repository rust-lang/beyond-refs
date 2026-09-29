#![expect(dead_code)]
use std::marker::PhantomData;

pub struct Point {
    x: i32,
    y: i32,
}

pub struct SoAPoint<const N: usize> {
    xs: [i32; N],
    ys: [i32; N],
}

pub struct SoAElementPoint<'a> {
    idx: usize,
    _lt: PhantomData<&'a SoAPoint<1>>,
}

fn main() {}
