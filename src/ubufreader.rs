use std::io;
use std::io::BufRead;

use error::{UwcError, Result};

/// An iterator over `&str`s read from a `BufRead`.
pub struct UStrChunksIter<'a, R: BufRead + 'a> {
    /// The `BufRead` to read from.
    pub reader: &'a mut R,

    /// Marks whether this iterator should keep reading from the reader or not. It
    /// will become false if the underlying reader has been closed, or some
    /// error has occurred.
    keep_reading: bool,
}

impl<'a, R: BufRead> UStrChunksIter<'a, R> {
    pub fn new(reader: &'a mut R) -> UStrChunksIter<'a, R> {
        UStrChunksIter{ reader, keep_reading: true }
    }
}

impl<'a, R: BufRead> Iterator for UStrChunksIter<'a, R> {
    type Item = Result<String>;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.keep_reading {
            return None;
        }

        let mut data = Vec::new();

        {
            let buf = match self.reader.fill_buf() {
                Ok(b) => b,
                Err(e) => {
                    match e.kind() {
                        io::ErrorKind::BrokenPipe => {
                            self.keep_reading = false;
                            return None;
                        }
                        _ => {
                            self.keep_reading = false;
                            return Some(Err(UwcError::IoError(e)));
                        }
                    }
                }
            };

            if buf.len() == 0 {
                self.keep_reading = false;
                return None;
            }

            data.extend(buf);
        }

        let string = match String::from_utf8(data) {
            Ok(s) => s,
            Err(e) => return Some(Err(UwcError::Utf8Error(e))),
        };

        self.reader.consume(string.len());

        Some(Ok(string))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::io;
    use std::io::BufReader;
    use env_logger;

    #[test]
    fn test_basic() {
        let _ = env_logger::init();
        let mut cursor = io::Cursor::new(b"hello");
        let mut chunks = UStrChunksIter::new(&mut cursor);
        let mut s = chunks.next();
        assert_eq!("hello", s.unwrap().unwrap());

        s = chunks.next();
        debug!("{:?}", s);
        assert!(s.is_none());
        assert!(s.is_none());
    }

    #[test]
    fn test_basic_buffered() {
        let cursor = io::Cursor::new(b"hello");
        let mut reader = BufReader::with_capacity(3, cursor);
        let mut chunks = UStrChunksIter::new(&mut reader);
        assert_eq!("hel", chunks.next().unwrap().unwrap());
        assert_eq!("lo", chunks.next().unwrap().unwrap());
        assert!(chunks.next().is_none());
        assert!(chunks.next().is_none());
    }

    #[test]
    fn test_buffered_stops_in_middle() {
        // 😬 is 4 bytes
        let cursor = io::Cursor::new("hello 😬 whoops".as_bytes());

        // this should stop reading 2 bytes into the emoji
        let mut reader = BufReader::with_capacity(8, cursor);
        let mut chunks = UStrChunksIter::new(&mut reader);

        assert_eq!("hello ", chunks.next().unwrap().unwrap());
        assert_eq!("😬 whoops", chunks.next().unwrap().unwrap());
        assert!(chunks.next().is_none());
        assert!(chunks.next().is_none());
    }
}
