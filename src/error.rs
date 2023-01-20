use std::fmt::{self, Display};
use std::io;
use std::str::Utf8Error;

/// Error while reading.
#[derive(Debug)]
pub enum ReadError {
    Generic(String),
    Utf8(Utf8Error),
    Lzo(minilzo::Error),
    Io(io::Error),
}

impl From<Utf8Error> for ReadError {
    fn from(err: Utf8Error) -> Self {
        Self::Utf8(err)
    }
}

impl From<minilzo::Error> for ReadError {
    fn from(err: minilzo::Error) -> Self {
        Self::Lzo(err)
    }
}

impl From<io::Error> for ReadError {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

impl Display for ReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            ReadError::Generic(ref message) => f.write_str(message),
            ReadError::Utf8(ref err) => Display::fmt(err, f),
            ReadError::Lzo(ref err) => Display::fmt(err, f),
            ReadError::Io(ref err) => Display::fmt(err, f),
        }
    }
}

impl std::error::Error for ReadError {}

/// Read result.
pub type ReadResult<T> = std::result::Result<T, ReadError>;
