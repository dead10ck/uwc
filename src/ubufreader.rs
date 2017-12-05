use std::str;
use std::io;
use std::io::BufRead;

use error::{UwcError, Result};

/// An iterator over `&str` chunks.
trait UStrChunks: BufRead {
    
    /// Returns 
    fn unicode_str_chunks<'a>(&'a mut self) -> UStrChunksIter<'a, Self> where Self: Sized;
}

/// An iterator over `&str`s read from a `BufRead`.
struct UStrChunksIter<'a, R: BufRead + 'a> {

    /// The `BufRead` to read from.
    pub reader: &'a mut R,

    /// Marks whether this iterator should keep reading from the reader or not. It
    /// will become false if the underlying reader has been closed, or some
    /// error has occurred.
    keep_reading: bool,
}

impl <'a, R: BufRead> Iterator for UStrChunksIter<'a, R> {
    type Item = Result<&'a str>;

    fn next(&mut self) -> Option<Self::Item> {
        let buf = match self.reader.fill_buf() {
            Ok(b) => b,
            Err(e) => match e.kind() {
                io::ErrorKind::BrokenPipe => return self.stop(None),
                _ => return self.stop(Some(Err(UwcError::IoError(e)))),
            }
        };

        let string = match str::from_utf8(buf) {
            Ok(s) => s,
            Err(e) => return Some(Err(UwcError::Utf8Error(e))),
        };

        Some(Ok(string))
    }
}

impl <'a, R: BufRead> UStrChunksIter<'a, R> {
    fn stop<T>(&mut self, ret: T) -> T {
        self.keep_reading = false;
        ret
    }
}

/*
impl<R: BufRead> UBufReader<R> {
    pub fn new(reader: R) -> UBufReader<R> {
        UBufReader{ reader }
    }

    pub unicode_str_chunks(&self) -> StrChunks {
        StrChunks{ self.reader }
    }
}
*/
