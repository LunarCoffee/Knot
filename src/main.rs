#![feature(type_alias_impl_trait)]
#![feature(seek_convenience)]

use std::io::Cursor;
use std::ops::{Add, Div, Mul, Sub};

use crate::parse::base::*;
use crate::parse::combinators::*;
use crate::parse::parser::Parser;

mod parse;

fn main() {
    let a = [1, 2, 3];
    let [a, b, c] = a;

    add_expr_test();
}

fn add_expr_test() {
    let operand = decimal::<i32>();
    let operation = string("+")
        .or(string("-"))
        .or(string("*"))
        .or(string("/"))
        .map(|op| match op {
            "+" => i32::add,
            "-" => i32::sub,
            "*" => i32::mul,
            _ => i32::div,
        });
    let mut expr = (&operand).and(operation).and(&operand).map(|((a, op), b)| op(a, b));

    let mut input = Cursor::new(b"100*50");
    println!("{:?}", expr.parse(&mut input));
}
