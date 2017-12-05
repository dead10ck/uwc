use io;
use std;
use std::str;

/// An error that can occur during a run of `uwc`.
#[derive(Debug, Fail)]
pub enum UwcError {
    #[fail(display = "io error occurred: {}", _0)]
    IoError(io::Error),

    #[fail(display = "utf8 error error occurred: {}", _0)]
    Utf8Error(str::Utf8Error),
}

pub type Result<T> = std::result::Result<T, UwcError>;
