use std::io::{Cursor, SeekFrom};
use std::marker::PhantomData;

use crate::parse::parser::{ParseError, Parser, ParseResult, ReadSeek};
use crate::parse::parser;

// Parses `first` then `second`, returning the result parsed by both in a tuple.
pub struct AndParser<P1: Parser, P2: Parser> {
    first: P1,
    second: P2,
}

impl<P1: Parser, P2: Parser> Parser for AndParser<P1, P2> {
    type Output = (P1::Output, P2::Output);

    fn parse(&self, reader: &mut impl ReadSeek) -> ParseResult<Self::Output> {
        parser::backtrack_on_fail(reader, |r| {
            let first = self.first.parse(r)?;
            let second = self.second.parse(r)?;
            Ok((first, second))
        })
    }
}

pub trait AndParserExt: Parser {
    fn and<P: Parser>(self, second: P) -> AndParser<Self, P> where Self: Sized {
        AndParser { first: self, second }
    }
}

impl<P: Parser> AndParserExt for P {}

// Returns the result of `first` if successful, otherwise returning the result of `second`.
pub struct OrParser<T, P1: Parser<Output=T>, P2: Parser<Output=T>> {
    first: P1,
    second: P2,
}

impl<T, P1: Parser<Output=T>, P2: Parser<Output=T>> Parser for OrParser<T, P1, P2> {
    type Output = T;

    fn parse(&self, reader: &mut impl ReadSeek) -> ParseResult<Self::Output> {
        parser::backtrack_on_fail(reader, |r| self.first.parse(r).or_else(|_| self.second.parse(r)))
    }
}

pub trait OrParserExt: Parser {
    fn or<P: Parser<Output=Self::Output>>(self, second: P) -> OrParser<Self::Output, Self, P> where Self: Sized {
        OrParser { first: self, second }
    }
}

impl<P: Parser> OrParserExt for P {}

// Parses `parser` `times` times, returning all results. One failure causes the entire parse to fail.
pub struct ExactParser<P: Parser> {
    parser: P,
    times: usize,
}

impl<P: Parser> Parser for ExactParser<P> {
    type Output = Vec<P::Output>;

    fn parse(&self, reader: &mut impl ReadSeek) -> ParseResult<Self::Output> {
        parser::backtrack_on_fail(reader, |r| {
            let mut results = Vec::with_capacity(self.times);
            for _ in 0..self.times {
                results.push(self.parser.parse(r)?);
            }
            Ok(results)
        })
    }
}

pub trait ExactParserExt: Parser {
    fn exact(self, times: usize) -> ExactParser<Self> where Self: Sized {
        ExactParser { parser: self, times }
    }
}

impl<P: Parser> ExactParserExt for P {}

// Parses `first` then `second`, returning the result parsed by `second`.
pub struct ThenParser<P1: Parser, P2: Parser> {
    first: P1,
    second: P2,
}

impl<P1: Parser, P2: Parser> Parser for ThenParser<P1, P2> {
    type Output = P2::Output;

    fn parse(&self, reader: &mut impl ReadSeek) -> ParseResult<Self::Output> {
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

    fn parse(&self, reader: &mut impl ReadSeek) -> ParseResult<Self::Output> {
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

    fn parse(&self, reader: &mut impl ReadSeek) -> ParseResult<Self::Output> {
        parser::backtrack_on_fail(reader, |r| Ok(self.parser.parse(r).ok()))
    }
}

pub trait OptionalParserExt: Parser {
    fn optional(self) -> OptionalParser<Self> where Self: Sized {
        OptionalParser { parser: self }
    }
}

impl<P: Parser> OptionalParserExt for P {}

// Runs `parser` zero (one if `min_one` is true) or more times, returning the results in a list.
pub struct ManyParser<P: Parser> {
    parser: P,
    min_one: bool,
}

impl<P: Parser> Parser for ManyParser<P> {
    type Output = Vec<P::Output>;

    fn parse(&self, reader: &mut impl ReadSeek) -> ParseResult<Self::Output> {
        parser::backtrack_on_fail(reader, |r| {
            let mut results = vec![];
            while let Ok(result) = self.parser.parse(r) {
                results.push(result);
            }
            if results.is_empty() && self.min_one { Err(ParseError) } else { Ok(results) }
        })
    }
}

pub trait ManyParserExt: Parser {
    fn many(self) -> ManyParser<Self> where Self: Sized {
        ManyParser { parser: self, min_one: false }
    }

    // Ensures at least one parse is finished.
    fn many1(self) -> ManyParser<Self> where Self: Sized {
        ManyParser { parser: self, min_one: true }
    }
}

impl<P: Parser> ManyParserExt for P {}

// Runs `prefix`, `parser`, and `suffix` in order, returning the result of `parser` if all are successful.
pub struct BetweenParser<P1: Parser, P2: Parser, P3: Parser> {
    prefix: P1,
    parser: P2,
    suffix: P3,
}

impl<P1: Parser, P2: Parser, P3: Parser> Parser for BetweenParser<P1, P2, P3> {
    type Output = P2::Output;

    fn parse(&self, reader: &mut impl ReadSeek) -> ParseResult<Self::Output> {
        parser::backtrack_on_fail(reader, |r| {
            self.prefix.parse(r)?;
            let result = self.parser.parse(r)?;
            self.suffix.parse(r)?;
            Ok(result)
        })
    }
}

pub trait BetweenParserExt: Parser {
    fn between<P1: Parser, P2: Parser>(self, prefix: P1, suffix: P2) -> BetweenParser<P1, Self, P2> where Self: Sized {
        BetweenParser { prefix, parser: self, suffix }
    }
}

impl<P: Parser> BetweenParserExt for P {}

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

pub trait MapParserExt<U, F: Fn(Self::Output) -> U>: Parser {
    fn map(self, mapping_fn: F) -> MapParser<Self::Output, U, Self, F> where Self: Sized {
        MapParser { parser: self, mapping_fn }
    }
}

impl<U, P: Parser, F: Fn(P::Output) -> U> MapParserExt<U, F> for P {}

// Parser which wraps another parser. This is useful when writing parsers for grammars with mutually recursive rules,
// since this type contains only the output type `T`, avoiding the problem of infinitely expanding types.
pub struct MutualRecursionParser<'a, T> {
    func: Box<dyn Fn(&mut dyn ReadSeek) -> ParseResult<T> + 'a>,
    phantom: PhantomData<T>,
}

impl<'a, T> MutualRecursionParser<'a, T> {
    pub fn new(parser: impl Parser<Output=T> + 'a) -> Self {
        MutualRecursionParser {
            phantom: PhantomData,
            func: box move |reader| {
                // This is rather inefficient but I can't be bothered to think of a better solution at the moment.
                let mut buf = Cursor::new(vec![]);
                let current = reader.stream_position()?;
                reader.read_to_end(buf.get_mut())?;

                // Parse then seek the original reader to the correct position, taking into account backtracking and
                // the result of the parse.
                let result = parser.parse(&mut buf);
                let seek_offset = if result.is_err() { current } else { current + buf.position() };
                reader.seek(SeekFrom::Start(seek_offset))?;
                result
            },
        }
    }
}

impl<'a, T> Parser for MutualRecursionParser<'a, T> {
    type Output = T;

    fn parse(&self, reader: &mut impl ReadSeek) -> ParseResult<Self::Output> where Self: Sized {
        (self.func)(reader)
    }
}

pub trait MutualRecursionParserExt: Parser {
    fn recursive<'a>(self) -> MutualRecursionParser<'a, Self::Output> where Self: Sized + 'a {
        MutualRecursionParser::new(self)
    }
}

impl<P: Parser> MutualRecursionParserExt for P {}
