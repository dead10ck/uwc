use std::fs::{self, File};
use std::io::{self, Read};
use std::path::Path;

/// The string used to identify stdin.
pub const STDIN_IDENTIFIER: &str = "-";

/// Choose between a regular file and stdin.
pub enum Input {
    File(fs::File),
    Stdin(io::Stdin),
}

impl Input {
    pub fn new<P: AsRef<Path>>(path: P) -> io::Result<Input> {
        let path = path.as_ref();

        if path.as_os_str() == STDIN_IDENTIFIER {
            return Ok(Input::Stdin(io::stdin()));
        }

        let file = File::open(path)?;
        Ok(Input::File(file))
    }
}

impl Read for Input {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match *self {
            Input::File(ref mut file) => file.read(buf),
            Input::Stdin(ref mut stdin) => stdin.read(buf),
        }
    }
}
