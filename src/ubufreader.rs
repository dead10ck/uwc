use std::io::BufRead;

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
}

impl<'a, R: BufRead> UStrChunksIter<'a, R> {
    pub fn new(reader: &'a mut R, keep_newline: bool) -> UStrChunksIter<'a, R> {
        UStrChunksIter {
            reader,
            keep_reading: true,
            keep_newline: keep_newline,
        }
    }
}

impl<'a, R: BufRead> Iterator for UStrChunksIter<'a, R> {
    type Item = Result<String>;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.keep_reading {
            return None;
        }

        let mut output = String::new();

        let read_bytes = match self.reader.read_line(&mut output) {
            Ok(b) => b,
            Err(e) => {
                self.keep_reading = false;
                return Some(Err(UwcError::IoError(e)));
            }
        };

        if read_bytes == 0 {
            self.keep_reading = false;
            return None;
        }

        // TODO: This is only necessary while we are using BufRead::read_line,
        // since this is exactly the byte sequence that it splits on. Once we
        // implement our own line splitter that includes all valid Unicode line
        // breaks, this code will need revision.
        //
        // Follow the example of std::io::Lines:
        // https://doc.rust-lang.org/src/std/io/mod.rs.html#2120
        if !self.keep_newline && output.ends_with("\n") {
            output.pop();

            if output.ends_with("\r") {
                output.pop();
            }
        }

        Some(Ok(output))
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
        let _ = env_logger::init();
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
        let _ = env_logger::init();
        let mut cursor = io::Cursor::new(b"hello\ngoodbye\r\nwindows?");
        let mut chunks = UStrChunksIter::new(&mut cursor, true);
        assert_eq!("hello\n", chunks.next().unwrap().unwrap());
        assert_eq!("goodbye\r\n", chunks.next().unwrap().unwrap());
        assert_eq!("windows?", chunks.next().unwrap().unwrap());

        assert!(chunks.next().is_none());
        assert!(chunks.next().is_none());
    }

    // TODO: Run these tests when the iterator does not chunk by newlines any more.
    #[test]
    #[ignore]
    fn test_basic_buffered() {
        let cursor = io::Cursor::new(b"hello");
        let mut reader = BufReader::with_capacity(3, cursor);
        let mut chunks = UStrChunksIter::new(&mut reader, true);
        assert_eq!("hel", chunks.next().unwrap().unwrap());
        assert_eq!("lo", chunks.next().unwrap().unwrap());
        assert!(chunks.next().is_none());
        assert!(chunks.next().is_none());
    }

    // TODO: Run these tests when the iterator does not chunk by newlines any more.
    #[test]
    #[ignore]
    fn test_buffered_stops_in_middle() {
        // 😬 is 4 bytes
        let cursor = io::Cursor::new("hello 😬 whoops".as_bytes());

        // this should stop reading 2 bytes into the emoji
        let mut reader = BufReader::with_capacity(8, cursor);
        let mut chunks = UStrChunksIter::new(&mut reader, true);

        assert_eq!("hello ", chunks.next().unwrap().unwrap());
        assert_eq!("😬 whoops", chunks.next().unwrap().unwrap());
        assert!(chunks.next().is_none());
        assert!(chunks.next().is_none());
    }

    // TODO: Run these tests when the iterator does not chunk by newlines any more.
    #[test]
    #[ignore]
    fn test_buffered_stops_in_middle_japanese() {
        let _ = env_logger::init();

        let cursor = io::Cursor::new(
            "私はガラスを食べられます。それは私を傷つけません。"
                .as_bytes(),
        );

        let mut reader = BufReader::with_capacity(10, cursor);
        let mut chunks = UStrChunksIter::new(&mut reader, true);

        assert_eq!("私はガ", chunks.next().unwrap().unwrap());
        assert_eq!("ラスを", chunks.next().unwrap().unwrap());
        assert_eq!("食べら", chunks.next().unwrap().unwrap());
        assert_eq!("れます", chunks.next().unwrap().unwrap());
        assert_eq!("。それ", chunks.next().unwrap().unwrap());
        assert_eq!("は私を", chunks.next().unwrap().unwrap());
        assert_eq!("傷つけ", chunks.next().unwrap().unwrap());
        assert_eq!("ません", chunks.next().unwrap().unwrap());
        assert_eq!("。", chunks.next().unwrap().unwrap());
        assert!(chunks.next().is_none());
        assert!(chunks.next().is_none());
    }
}
