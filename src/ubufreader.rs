use std::io::BufRead;
use std::mem;

use crate::error::{Result, UwcError};

/// An iterator over `&str`s read from a `BufRead`. For now, it reads lines,
/// similar to `BufRead::lines`, but it includes the newline character for
/// accurate counts.
//
// In the future, this should attempt to be more memory-stable by chunking by a
// fixed size, or close to a fixed size, that splits on grapheme cluster
// boundaries.
pub struct UStrChunksIter<'a, R: BufRead + 'a> {
    /// The `BufRead` to read from.
    pub reader: &'a mut R,

    /// Marks whether this iterator should keep reading from the reader or not. It
    /// will become false if the underlying reader has been closed, or some
    /// error has occurred.
    keep_reading: bool,

    /// For line mode. Indicates whether the newline should be kept or not.
    keep_newline: bool,

    /// Internal buffer for reading until a break point is found
    buf: Vec<u8>,
}

impl<'a, R: BufRead> UStrChunksIter<'a, R> {
    pub fn new(reader: &'a mut R, keep_newline: bool) -> UStrChunksIter<'a, R> {
        UStrChunksIter {
            reader,
            keep_reading: true,
            keep_newline: keep_newline,
            buf: Vec::new(),
        }
    }
}

impl<'a, R: BufRead> Iterator for UStrChunksIter<'a, R> {
    type Item = Result<String>;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.keep_reading {
            return None;
        }

        loop {
            let buffer = match self.reader.fill_buf() {
                Ok(buf) => buf,
                Err(err) => {
                    self.keep_reading = false;
                    return Some(Err(UwcError::IoError(err)));
                }
            };

            if buffer.len() == 0 {
                self.keep_reading = false;
                break;
            }

            let mat = crate::constants::NEWLINE_PATTERN.find(buffer);

            // if we didn't find a newline sequence, stuff the bytes into our
            // buffer and keep reading
            if mat.is_none() {
                self.buf.extend_from_slice(buffer);
                let length = buffer.len();
                self.reader.consume(length);
                continue;
            }

            let mat = mat.unwrap();

            let end = match self.keep_newline {
                true => mat.end(),
                false => mat.start(),
            };

            // copy up to the delimiter we found
            self.buf.extend_from_slice(&buffer[..end]);

            // consume the bytes including the delimiter regardless of whether we
            // want to keep the newlines for counting
            let consume_length = mat.end();
            self.reader.consume(consume_length);

            break;
        }

        if !self.keep_reading && self.buf.len() == 0 {
            return None;
        }

        // consume the buffer we've built so far and replace it with a new one
        let new_str_bytes = mem::replace(&mut self.buf, Vec::new());

        let new_str = match String::from_utf8(new_str_bytes) {
            Ok(s) => s,
            Err(err) => {
                self.keep_reading = false;
                return Some(Err(UwcError::Utf8Error(err)));
            }
        };

        Some(Ok(new_str))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use env_logger;
    use log::*;
    use std::io;
    use std::io::BufReader;

    #[test]
    fn test_basic() {
        let _ = env_logger::try_init();
        let mut cursor = io::Cursor::new(b"hello");
        let mut chunks = UStrChunksIter::new(&mut cursor, true);
        let mut s = chunks.next();
        assert_eq!("hello", s.unwrap().unwrap());

        s = chunks.next();
        debug!("{:?}", s);
        assert!(s.is_none());
        assert!(s.is_none());
    }

    #[test]
    fn test_chunks_by_newline() {
        let _ = env_logger::try_init();
        let mut cursor = io::Cursor::new(
            "hello\ngoodbye\r\nwindows?\u{0085}\u{000C}unicode\u{2028}newline\u{2029}sequences"
            .as_bytes());

        let mut chunks = UStrChunksIter::new(&mut cursor, true);
        assert_eq!("hello\n", chunks.next().unwrap().unwrap());
        assert_eq!("goodbye\r\n", chunks.next().unwrap().unwrap());
        assert_eq!("windows?\u{0085}", chunks.next().unwrap().unwrap());
        assert_eq!("\u{000C}", chunks.next().unwrap().unwrap());
        assert_eq!("unicode\u{2028}", chunks.next().unwrap().unwrap());
        assert_eq!("newline\u{2029}", chunks.next().unwrap().unwrap());
        assert_eq!("sequences", chunks.next().unwrap().unwrap());

        assert!(chunks.next().is_none());
        assert!(chunks.next().is_none());
    }

    #[test]
    fn test_chunks_by_newline_no_newlines() {
        let _ = env_logger::try_init();
        let mut cursor = io::Cursor::new(
            "hello\ngoodbye\r\nwindows?\u{0085}\u{000C}unicode\u{2028}newline\u{2029}sequences"
            .as_bytes());

        let mut chunks = UStrChunksIter::new(&mut cursor, false);
        assert_eq!("hello", chunks.next().unwrap().unwrap());
        assert_eq!("goodbye", chunks.next().unwrap().unwrap());
        assert_eq!("windows?", chunks.next().unwrap().unwrap());
        assert_eq!("", chunks.next().unwrap().unwrap());
        assert_eq!("unicode", chunks.next().unwrap().unwrap());
        assert_eq!("newline", chunks.next().unwrap().unwrap());
        assert_eq!("sequences", chunks.next().unwrap().unwrap());

        assert!(chunks.next().is_none());
        assert!(chunks.next().is_none());
    }

    #[test]
    fn test_basic_buffered() {
        let cursor = io::Cursor::new(b"hello");
        let mut reader = BufReader::with_capacity(3, cursor);
        let mut chunks = UStrChunksIter::new(&mut reader, true);
        assert_eq!("hello", chunks.next().unwrap().unwrap());
        assert!(chunks.next().is_none());
        assert!(chunks.next().is_none());
    }

    #[test]
    fn test_buffered_stops_in_middle() {
        // 😬 is 4 bytes
        let cursor = io::Cursor::new("hello 😬 whoops".as_bytes());

        // this should stop reading 2 bytes into the emoji
        let mut reader = BufReader::with_capacity(8, cursor);
        let mut chunks = UStrChunksIter::new(&mut reader, true);

        assert_eq!("hello 😬 whoops", chunks.next().unwrap().unwrap());
        assert!(chunks.next().is_none());
        assert!(chunks.next().is_none());
    }

    #[test]
    fn test_buffered_stops_in_middle_japanese() {
        let _ = env_logger::try_init();

        let cursor =
            io::Cursor::new("私はガラスを食べられます。\nそれは私を傷つけません。".as_bytes());

        // with a capacity of 10, it should stop in the middle of some graphemes
        let mut reader = BufReader::with_capacity(10, cursor);
        let mut chunks = UStrChunksIter::new(&mut reader, true);

        assert_eq!(
            "私はガラスを食べられます。\n",
            chunks.next().unwrap().unwrap()
        );
        assert_eq!("それは私を傷つけません。", chunks.next().unwrap().unwrap());

        assert!(chunks.next().is_none());
        assert!(chunks.next().is_none());
    }
}
