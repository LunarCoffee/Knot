use std::marker::PhantomData;
use std::ops::Neg;
use std::str::FromStr;

use num::Integer;

use crate::parse;
use crate::parse::{ParseError, Parser, ParseResult, ReadSeek};
use crate::parse::combinators::{AndParserExt, ManyParserExt, MapParserExt, OptionalParserExt};

// Parses a sequence of bytes.
pub struct ByteSeqParser<'a> {
    bytes: &'a [u8],
}

impl<'a> Parser for ByteSeqParser<'a> {
    type Output = &'a [u8];

    fn parse(&self, reader: &mut impl ReadSeek) -> ParseResult<Self::Output> {
        parse::backtrack_on_fail(reader, |r| {
            let mut buf = [0];
            for b in self.bytes {
                r.read_exact(&mut buf)?;
                if *b != buf[0] {
                    let error = match String::from_utf8(self.bytes.to_vec()) {
                        Ok(string) => format!("expected '{}'", string),
                        _ => format!("expected bytes {:?}", self.bytes),
                    };
                    return Err(ParseError::new(&error));
                }
            }
            Ok(self.bytes)
        })
    }
}

pub fn bytes(bytes: &[u8]) -> ByteSeqParser {
    ByteSeqParser { bytes }
}

// Parses a string.
pub fn string(string: &str) -> impl Parser<Output=String> + '_ {
    bytes(string.as_bytes()).map(|b| String::from_utf8_lossy(b).to_string())
}

// Parses and discards any amount of whitespace.
pub fn spaces() -> impl Parser<Output=()> {
    " ".many().map(|_| ())
}

// Parses an optional minus sign, returning a function which takes an integer and returns its value negated if a minus
// sign was present, and its value without modification otherwise.
pub fn sign<I: Integer + Neg<Output=I>>() -> impl Parser<Output=fn(I) -> I> {
    "-".optional().map(|str| match str {
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

    fn parse(&self, reader: &mut impl ReadSeek) -> ParseResult<Self::Output> {
        parse::backtrack_on_fail(reader, |r| {
            let mut string = String::new();
            let mut buf = [0];

            while r.read(&mut buf)? > 0 {
                if !buf[0].is_ascii_digit() {
                    parse::seek_back_one(r)?;
                    break;
                }
                string.push(buf[0] as char);
            }

            string.parse::<I>().map_err(|_| {
                let message = format!("decimal integer literal too large: {}", string);
                ParseError::new(&message)
            })
        })
    }
}

pub fn non_neg_decimal<I: Integer + FromStr>() -> NonNegDecimalParser<I> {
    NonNegDecimalParser { phantom: PhantomData }
}

// Parses a decimal (base 10) number into an integer type. The representation can contain any number of leading zeroes,
// meaning `"01"` -> `1`, `"-0032"` -> `-32`, etc.
pub fn decimal<I: Integer + FromStr + Neg<Output=I>>() -> impl Parser<Output=I> {
    sign.and(non_neg_decimal).map(|(sign_fn, n)| sign_fn(n))
}

// Parses an EOF error.
pub struct EofParser;

impl Parser for EofParser {
    type Output = ();

    fn parse(&self, reader: &mut impl ReadSeek) -> ParseResult<Self::Output> {
        parse::backtrack_on_fail(reader, |r| {
            let mut buf = [0];
            match r.read(&mut buf) {
                Ok(0) => Ok(()),
                Ok(_) => Err(ParseError::new(&format!("unexpected '{}'", buf[0] as char))),
                _ => Err(ParseError::new("expected eof"))
            }
        })
    }
}

pub fn eof() -> EofParser {
    EofParser
}
