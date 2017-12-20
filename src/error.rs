use io;
use std;
use std::string;

use encoding_rs::DecoderResult;

/// An error that can occur during a run of `uwc`.
#[derive(Debug, Fail)]
pub enum UwcError {
    #[fail(display = "io error occurred: {}", _0)]
    IoError(io::Error),

    #[fail(display = "malformed utf8 input: {:?}", input)]
    MalformedInputError {
        /// The input that encountered the error
        input: Vec<u8>,

        /// The start index of the error
        start_error_index: usize,
    },
}

pub type Result<T> = std::result::Result<T, UwcError>;
