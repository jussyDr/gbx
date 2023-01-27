use std::fmt::{self, Display};
use std::io;
use std::str::Utf8Error;

/// Read error.
#[derive(Debug)]
pub enum ReadError {
    Generic(String),
    Utf8(Utf8Error),
    Lzo(lzo1x::Error),
    Io(io::Error),
}

impl From<Utf8Error> for ReadError {
    fn from(err: Utf8Error) -> Self {
        Self::Utf8(err)
    }
}

impl From<lzo1x::Error> for ReadError {
    fn from(err: lzo1x::Error) -> Self {
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
            Self::Generic(ref message) => f.write_str(message),
            Self::Utf8(ref err) => Display::fmt(err, f),
            Self::Lzo(ref err) => Display::fmt(err, f),
            Self::Io(ref err) => Display::fmt(err, f),
        }
    }
}

impl std::error::Error for ReadError {}

/// Read result.
pub type ReadResult<T> = std::result::Result<T, ReadError>;

/// Write error.
#[derive(Debug)]
pub enum WriteError {
    Io(io::Error),
}

impl From<io::Error> for WriteError {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

impl Display for WriteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Io(ref err) => Display::fmt(err, f),
        }
    }
}

impl std::error::Error for WriteError {}

/// Write Result.
pub type WriteResult = std::result::Result<(), WriteError>;
