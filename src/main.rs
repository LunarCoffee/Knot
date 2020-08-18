#![feature(type_alias_impl_trait)]
#![feature(seek_convenience)]

use std::io::Cursor;
use std::ops::{Add, Div, Mul, Sub};

use crate::parse::combinators::*;
use crate::parse::parser::Parser;
use crate::parse::std_parsers::*;

mod parse;

fn main() {
    expr_test();
}

fn expr_test() {
    let fold_step = |(fst, rest): (_, Vec<(fn(i32, i32) -> i32, _)>)|
        rest.iter().fold(fst, |res, (op, n)| op(res, *n));

    let term_op = string("*").or(string("/")).map(|op| if op == "*" { i32::mul } else { <i32 as Div>::div });
    let expr_op = string("+").or(string("-")).map(|op| if op == "+" { i32::add } else { <i32 as Sub>::sub });

    let spaces = string(" ").many();
    let factor = decimal::<i32>().between(&spaces, &spaces);
    let term = (&factor).and(term_op.and(&factor).many()).map(fold_step);
    let expr = (&term).and(expr_op.and(&term).many()).map(fold_step);

    let mut input = Cursor::new(b"3*4*8/7 + 5*2 - 2 + 1*3");
    println!("{}", expr.parse(&mut input).unwrap_or(-1));
}
