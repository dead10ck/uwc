use io;
use std;

/// An error that can occur during a run of `uwc`.
#[derive(Debug, Fail)]
pub enum UwcError {
    #[fail(display = "io error occurred: {}", _0)]
    IoError(io::Error),
}

pub type Result<T> = std::result::Result<T, UwcError>;
