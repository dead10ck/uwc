use crate::io;
use std;

use failure::Fail;

/// An error that can occur during a run of `uwc`.
#[derive(Debug, Fail)]
pub enum UwcError {
    #[fail(display = "io error occurred: {}", _0)]
    IoError(io::Error),

    #[fail(display = "read non-utf8 bytes: {}", _0)]
    Utf8Error(std::string::FromUtf8Error),
}

pub type Result<T> = std::result::Result<T, UwcError>;
