use std;
use std::io;
use std::io::BufRead;
use std::string::FromUtf8Error;

use error::{UwcError, Result};
use encoding_rs::{UTF_8, Decoder, DecoderResult};

/// An iterator over `&str`s read from a `BufRead`.
pub struct UStrChunksIter<'a, R: BufRead + 'a> {
    /// The `BufRead` to read from.
    pub reader: &'a mut R,

    /// Marks whether this iterator should keep reading from the reader or not. It
    /// will become false if the underlying reader has been closed, or some
    /// error has occurred.
    keep_reading: bool,

    /// The byte stream decoder.
    decoder: Decoder,
}

impl<'a, R: BufRead> UStrChunksIter<'a, R> {
    pub fn new(reader: &'a mut R) -> UStrChunksIter<'a, R> {
        UStrChunksIter {
            reader,
            keep_reading: true,
            decoder: UTF_8.new_decoder(),
        }
    }
}

impl<'a, R: BufRead> Iterator for UStrChunksIter<'a, R> {
    type Item = Result<String>;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.keep_reading {
            return None;
        }

        let mut output : String;

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

            debug!("Read buf: {:?}", buf);

            output = String::with_capacity(buf.len());

            if buf.len() == 0 {
                self.keep_reading = false;
            }

            let (decode_result, bytes_read) = self.decoder.decode_to_string_without_replacement(
                buf,
                &mut output,
                !self.keep_reading,
            );

            // TODO: handle results of this call

            debug!("decode result: ({:?}, {})", decode_result, bytes_read);

            match decode_result {
                DecoderResult::Malformed(err_len, extra_bits) => {
                    self.keep_reading = false;
                    let mut error_data = Vec::new();
                    error_data.copy_from_slice(buf);

                    return Some(Err(UwcError::MalformedInputError{
                        input: error_data,
                        start_error_index: bytes_read - extra_bits as usize - err_len as usize,
                    }))
                },

                DecoderResult::OutputFull => {

                }

                _ => {},
            }
        }

        // if there is no data, return None instead of a blank string
        if output.len() == 0 {
            return None;
        }

        self.reader.consume(output.len());

        Some(Ok(output))
    }
}

/// Attempts to convert the given bytes into a UTF-8 String. It will return the
/// longest valid String inside the given byte vector, discarding any trailing
/// bytes that are invalid or form an incomplete UTF-8 grapheme cluster.
fn from_utf8_longest_valid(data: Vec<u8>) -> std::result::Result<String, FromUtf8Error> {
    // Attempt to convert the bytes to a String. If it's immediately successful,
    // just return it straight away. Otherwise, collect the error to see up to
    // what point it was valid.
    let error = match String::from_utf8(data) {
        Ok(s) => return Ok(s),
        Err(e) => e,
    };

    let valid_up_to = error.utf8_error().valid_up_to();

    // if not even the first byte was valid, then just return the original error
    if valid_up_to <= 0 {
        return Err(error);
    }

    // get the bytes back from the error
    let mut data = error.into_bytes();

    // discard everything up to the first invalid byte
    data.truncate(valid_up_to);

    // try to convert again
    String::from_utf8(data)
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
        let _ = env_logger::init();
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
        let _ = env_logger::init();

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

    #[test]
    fn test_buffered_stops_in_middle_japanese() {
        let _ = env_logger::init();

        let cursor = io::Cursor::new(
            "私はガラスを食べられます。それは私を傷つけません。"
                .as_bytes(),
        );

        let mut reader = BufReader::with_capacity(10, cursor);
        let mut chunks = UStrChunksIter::new(&mut reader);

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
