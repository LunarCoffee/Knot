#![feature(associated_type_defaults)]
#![feature(seek_convenience)]
#![feature(type_alias_impl_trait)]
#![feature(box_syntax)]

use std::io::Cursor;
use std::ops::{Add, Div, Mul, Sub};

use crate::parse::combinators::*;
use crate::parse::parser::Parser;
use crate::parse::std_parsers::*;

mod parse;

fn main() {
    let mut input = Cursor::new("3*4*(-(2+6)+10*(2+4+-1))/7+5*(4+3)*2-2+-1*3".as_bytes()); // 137
    println!("{}", expr.parse_to_end(&mut input).unwrap_or(-1));
}

fn fold_step((first, rest): (i32, Vec<(fn(i32, i32) -> i32, i32)>)) -> i32 {
    rest.iter().fold(first, |res, (op, n)| op(res, *n))
}

fn factor() -> impl Parser<Output=i32> {
    non_neg_decimal::<i32>
        .or(expr.between("(", ")").recursive())
        .or("-".then(factor).map(|n| -n).recursive())
}

fn term() -> impl Parser<Output=i32> {
    let op = "*".or("/").map(|op| if op == "*" { i32::mul } else { <i32 as Div>::div });
    factor.and(op.and(factor).many()).map(fold_step)
}

fn expr() -> impl Parser<Output=i32> {
    let op = "+".or("-").map(|op| if op == "+" { i32::add } else { <i32 as Sub>::sub });
    term.and(op.and(term).many()).map(fold_step)
}
