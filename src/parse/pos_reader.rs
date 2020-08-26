use std::io::{ErrorKind, Read, Seek, SeekFrom};
use std::io;

use crate::parse::types::ReadSeek;

// Wrapper around a `ReadSeek` which stores position information, useful for reporting errors.
pub struct PositionReader<'a, R: ReadSeek> {
    reader: &'a mut R,
    stream_len: u64,

    pos: u64,
    line: u64,
    col: u64,

    // Stores the lengths of lines previously read so that the column can be determined when seeking backwards. This
    // also requires `reader` to initially be at the beginning of its stream; otherwise, seeking before the initial
    // position would make determining the column prohibitively expensive.
    line_lens: Vec<u64>,
}

impl<'a, R: ReadSeek> PositionReader<'a, R> {
    pub fn new(reader: &'a mut R) -> Option<Self> {
        if reader.stream_position().unwrap_or(1) > 0 {
            None
        } else {
            let stream_len = reader.stream_len().unwrap();
            Some(PositionReader {
                reader,
                stream_len,
                pos: 0,
                line: 0,
                col: 0,
                line_lens: vec![0],
            })
        }
    }

    pub fn position(&self) -> u64 {
        self.pos
    }

    pub fn line(&self) -> u64 {
        self.line
    }

    pub fn col(&self) -> u64 {
        self.col
    }
}

impl<'a, R: ReadSeek> Read for PositionReader<'a, R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let read = self.reader.read(buf)?;
        for byte in &buf[..read] {
            if *byte == b'\n' {
                self.line += 1;
                self.col = 0;
                self.line_lens.push(0);
            }
            self.pos += 1;
            self.col += 1;
            self.line_lens[self.line as usize] += 1;
        }
        Ok(read)
    }
}

impl<'a, R: ReadSeek> Seek for PositionReader<'a, R> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let offset_from_start = match pos {
            SeekFrom::Start(offset) => offset as i64,
            SeekFrom::End(offset) => self.stream_len as i64 - offset - 1,
            SeekFrom::Current(offset) => self.pos as i64 + offset,
        };

        let diff = offset_from_start - self.pos as i64;
        if diff == 0 {
            return self.stream_position();
        }
        let is_forward = diff > 0;
        let seeked = num::abs(diff) as u64;

        let mut buf = vec![0; seeked as usize];
        if is_forward {
            self.read_exact(&mut buf).unwrap();
            Ok(self.pos)
        } else {
            self.reader.seek(pos)?;
            self.reader.read_exact(&mut buf).unwrap();

            // Modify positions accordingly.
            let line_diff = buf.iter().filter(|b| **b == b'\n').count() as u64;
            self.line -= line_diff;
            self.pos -= seeked;

            // Calculate the column number.
            let n_before_line_feed = buf.iter().take_while(|b| **b != b'\n').count() as u64;
            self.line_lens[self.line as usize] -= n_before_line_feed;
            self.col = self.line_lens[self.line as usize];
            self.reader.seek(pos)
        }
    }

    fn stream_len(&mut self) -> io::Result<u64> {
        Ok(self.stream_len)
    }

    fn stream_position(&mut self) -> io::Result<u64> {
        Ok(self.pos)
    }
}
