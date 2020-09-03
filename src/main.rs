#![feature(associated_type_defaults)]
#![feature(seek_convenience)]
#![feature(type_alias_impl_trait)]
#![feature(box_syntax)]

use std::io::Cursor;

use crate::parse::{ParseError, Parser};
use crate::parse::combinators::*;
use crate::parse::std_parsers::*;

mod lang;
mod parse;

fn main() {
    let mut input = Cursor::new("3*4*((2+6))+10*(2+4+3))/7+5*(4+3)*2-2+1*3".as_bytes());
    println!("{}", match expr.with_position().parse_to_end(&mut input) {
        Ok(result) => result,
        Err(ParseError { reason }) => reason,
    });
}

fn fold_to_postfix((first, rest): (String, Vec<(String, String)>)) -> String {
    rest.iter().fold(first, |res, (op, n)| format!("{} {} {}", res, n, op))
}

fn factor() -> impl Parser<Output=String> {
    let number = non_neg_decimal::<i32>.map(|n| n.to_string());
    let paren_expr = expr.between("(", ")").recursive();
    number.or(paren_expr)
}

fn term() -> impl Parser<Output=String> {
    let op_and_factor = "*".or("/").and(factor);
    factor.and(op_and_factor.many()).map(fold_to_postfix)
}

fn expr() -> impl Parser<Output=String> {
    let op_and_factor = "+".or("-").and(term);
    term.and(op_and_factor.many()).map(fold_to_postfix)
}
