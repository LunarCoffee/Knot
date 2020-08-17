#![feature(type_alias_impl_trait)]
#![feature(seek_convenience)]

use std::io::Cursor;

use crate::parse::std_parsers::*;
use crate::parse::combinators::*;
use crate::parse::parser::Parser;

mod parse;

fn main() {
    add_expr_test();
}

fn add_expr_test() {
    let operand = decimal::<i32>();
    let operation = string("+");
    let expr = (&operand)
        .and(operation.then(&operand).many())
        .between(string("["), string("]"))
        .map(|(first, rest)| first + rest.iter().sum::<i32>());

    let mut input = Cursor::new(b"[100+50+-1+2+3]");
    println!("{:?}", expr.parse(&mut input));
}
