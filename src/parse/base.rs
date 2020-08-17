use std::marker::PhantomData;
use std::ops::Neg;
use std::str::FromStr;

use num::Integer;

use crate::parse::combinators::{MapParserExt, OptionalParserExt};
use crate::parse::parser::{ParseError, Parser, ParseResult, ReadSeek};
use crate::parse::parser;

// Parses a string.
pub struct StringParser<'a> {
    string: &'a str,
}

impl<'a> Parser for StringParser<'a> {
    type Output = &'a str;

    fn parse(&self, reader: &mut impl ReadSeek) -> ParseResult<&'a str> {
        parser::backtrack_on_fail(reader, |r| {
            let mut buf = vec![0];
            for b in self.string.bytes() {
                r.read_exact(&mut buf)?;
                if b != buf[0] {
                    return Err(ParseError);
                }
            }
            Ok(self.string)
        })
    }
}

pub fn string(string: &str) -> StringParser {
    StringParser { string }
}

// Parses an optional minus sign, returning a function which takes an integer and returns its value negated if a minus
// sign was present, and its value without modification otherwise.
pub fn sign<I: Integer + Neg<Output=I>>() -> impl Parser<Output=fn(I) -> I> {
    string("-").optional().map(|str| match str {
        Some(_) => |i: I| -i,
        _ => |i: I| i,
    })
}

// Parses a nonnegative decimal (base 10) number into an integer type. The representation can contain any number of
// leading zeroes, meaning `"005"` -> `5`, etc.
pub struct NonNegDecimalParser<I: Integer + FromStr> {
    phantom: PhantomData<I>,
}

impl<I: Integer + FromStr> Parser for NonNegDecimalParser<I> {
    type Output = I;

    fn parse(&self, reader: &mut impl ReadSeek) -> ParseResult<I> {
        parser::backtrack_on_fail(reader, |r| {
            let mut string = String::new();
            let mut buf = vec![0];

            while r.read(&mut buf)? > 0 {
                if !buf[0].is_ascii_digit() {
                    parser::seek_back_one(r)?;
                    break;
                }
                string.push(buf[0] as char);
            }
            string.parse::<I>().map_err(|_| ParseError)
        })
    }
}

pub fn non_neg_decimal<I: Integer + FromStr>() -> NonNegDecimalParser<I> {
    NonNegDecimalParser { phantom: PhantomData }
}

// Parses a decimal (base 10) number into an integer type. The representation can contain any number of leading zeroes,
// meaning `"01"` -> `1`, `"-0032"` -> `-32`, etc.
pub struct DecimalParser<I: Integer + FromStr + Neg<Output=I>> {
    phantom: PhantomData<I>,
}

impl<I: Integer + FromStr + Neg<Output=I>> Parser for DecimalParser<I> {
    type Output = I;

    fn parse(&self, reader: &mut impl ReadSeek) -> ParseResult<I> {
        parser::backtrack_on_fail(reader, |r| {
            let sign = sign().parse(r)?;
            Ok(sign(non_neg_decimal().parse(r)?))
        })
    }
}

pub fn decimal<I: Integer + FromStr + Neg<Output=I>>() -> DecimalParser<I> {
    DecimalParser { phantom: PhantomData }
}
