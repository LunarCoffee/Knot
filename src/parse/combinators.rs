use crate::parse::parser::{ParseError, Parser, ParseResult, ReadSeek};
use crate::parse::parser;

// Parses `first` then `second`, returning the result parsed by both in a tuple.
pub struct AndParser<P1: Parser, P2: Parser> {
    first: P1,
    second: P2,
}

impl<P1: Parser, P2: Parser> Parser for AndParser<P1, P2> {
    type Output = (P1::Output, P2::Output);

    fn parse(&self, reader: &mut impl ReadSeek) -> ParseResult<(P1::Output, P2::Output)> {
        parser::backtrack_on_fail(reader, |r| Ok((self.first.parse(r)?, self.second.parse(r)?)))
    }
}

pub trait AndParserExt: Parser {
    fn and<P2: Parser>(self, second: P2) -> AndParser<Self, P2> where Self: Sized {
        AndParser { first: self, second }
    }
}

impl<P: Parser> AndParserExt for P {}

// Parses each of `parsers` in order, returning the first successful parse (if any).
pub struct AnyParser<P: Parser> {
    parsers: Vec<P>,
}

impl<P: Parser> AnyParser<P> {
    pub fn or(mut self, parser: P) -> Self {
        self.parsers.push(parser);
        self
    }
}

impl<P: Parser> Parser for AnyParser<P> {
    type Output = P::Output;

    fn parse(&self, reader: &mut impl ReadSeek) -> ParseResult<P::Output> {
        parser::backtrack_on_fail(
            reader,
            |r| self.parsers.iter().fold(Err(ParseError), |result, p| result.or_else(|_| p.parse(r))),
        )
    }
}

pub trait AnyParserExt: Parser {
    fn or(self, parser: Self) -> AnyParser<Self> where Self: Sized {
        AnyParser { parsers: vec![self, parser] }
    }
}

impl<P: Parser> AnyParserExt for P {}

// Parses `first` then `second`, returning the result parsed by `second`.
pub struct ThenParser<P1: Parser, P2: Parser> {
    first: P1,
    second: P2,
}

impl<P1: Parser, P2: Parser> Parser for ThenParser<P1, P2> {
    type Output = P2::Output;

    fn parse(&self, reader: &mut impl ReadSeek) -> ParseResult<P2::Output> {
        parser::backtrack_on_fail(reader, |r| {
            self.first.parse(r)?;
            Ok(self.second.parse(r)?)
        })
    }
}

pub trait ThenParserExt: Parser {
    fn then<P2: Parser>(self, second: P2) -> ThenParser<Self, P2> where Self: Sized {
        ThenParser { first: self, second }
    }
}

impl<P: Parser> ThenParserExt for P {}

// Parses `first` then `second`, returning the result parsed by `first`.
pub struct WithParser<P1: Parser, P2: Parser> {
    first: P1,
    second: P2,
}

impl<P1: Parser, P2: Parser> Parser for WithParser<P1, P2> {
    type Output = P1::Output;

    fn parse(&self, reader: &mut impl ReadSeek) -> ParseResult<P1::Output> {
        parser::backtrack_on_fail(reader, |r| {
            let first_res = self.first.parse(r)?;
            self.second.parse(r)?;
            Ok(first_res)
        })
    }
}

pub trait WithParserExt: Parser {
    fn with<P2: Parser>(self, second: P2) -> WithParser<Self, P2> where Self: Sized {
        WithParser { first: self, second }
    }
}

impl<P: Parser> WithParserExt for P {}

// Runs `parser`, returning its result if successful, returning `None` otherwise.
pub struct OptionalParser<P: Parser> {
    parser: P
}

impl<P: Parser> Parser for OptionalParser<P> {
    type Output = Option<P::Output>;

    fn parse(&self, reader: &mut impl ReadSeek) -> ParseResult<Option<P::Output>> {
        parser::backtrack_on_fail(reader, |r| Ok(self.parser.parse(r).ok()))
    }
}

pub trait OptionalParserExt: Parser {
    fn optional(self) -> OptionalParser<Self> where Self: Sized {
        OptionalParser { parser: self }
    }
}

impl<P: Parser> OptionalParserExt for P {}

// A parser which maps `mapping_fn` over `parser`.
pub struct MapParser<T, U, P: Parser<Output=T>, F: Fn(T) -> U> {
    parser: P,
    mapping_fn: F,
}

impl<T, U, P: Parser<Output=T>, F: Fn(T) -> U> Parser for MapParser<T, U, P, F> {
    type Output = U;

    fn parse(&self, reader: &mut impl ReadSeek) -> ParseResult<Self::Output> {
        parser::backtrack_on_fail(reader, |r| self.parser.parse(r).map(&self.mapping_fn))
    }
}

pub trait MapParserExt<T>: Parser<Output=T> {
    fn map<U, F: Fn(T) -> U>(self, f: F) -> MapParser<T, U, Self, F> where Self: Sized {
        MapParser { parser: self, mapping_fn: f }
    }
}

impl<T, P: Parser<Output=T>> MapParserExt<T> for P {}
