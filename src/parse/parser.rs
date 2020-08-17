use std::error::Error;
use std::fmt::{Display, Formatter};
use std::{fmt, io};
use std::io::{Read, Seek, SeekFrom};

#[derive(Debug, Clone)]
pub struct ParseError;

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str("parse failed")
    }
}

impl<E: Error> From<E> for ParseError {
    fn from(_: E) -> Self {
        ParseError
    }
}

pub type ParseResult<T> = Result<T, ParseError>;

pub trait ReadSeek: Read + Seek {}

impl<T: Read + Seek> ReadSeek for T {}

pub trait Parser {
    type Output;

    // Parses data from `reader` until the parser is finished or an error occurs.
    fn parse(&self, reader: &mut impl ReadSeek) -> ParseResult<Self::Output>;

    // Like `parse`, but ensures `reader` contains no more data to parse.
    fn parse_to_end(&self, reader: &mut impl ReadSeek) -> ParseResult<Self::Output> {
        let result = self.parse(reader)?;
        let mut buf = [0; 1];
        if reader.read(&mut buf).unwrap_or(0) == 0 { Err(ParseError) } else { Ok(result) }
    }
}

impl<P: Parser> Parser for &P {
    type Output = P::Output;

    fn parse(&self, reader: &mut impl ReadSeek) -> ParseResult<Self::Output> {
        P::parse(self, reader)
    }
}

pub fn backtrack_on_fail<T, R, F>(reader: &mut R, mut f: F) -> ParseResult<T>
    where R: ReadSeek,
          F: FnMut(&mut R) -> ParseResult<T>
{
    let initial_pos = reader.stream_position().unwrap();
    let result = f(reader);
    if result.is_err() {
        reader.seek(SeekFrom::Start(initial_pos))?;
    }
    result
}

pub fn seek_back_one(reader: &mut impl ReadSeek) -> io::Result<u64> {
    reader.seek(SeekFrom::Current(-1))
}
