use std::fmt::{self, Display};
use std::io;
use std::str::Utf8Error;

/// Error while reading.
#[derive(Debug)]
pub enum ReadError {
    Generic(String),
    Utf8(Utf8Error),
    Io(io::Error),
}

impl From<Utf8Error> for ReadError {
    fn from(err: Utf8Error) -> Self {
        Self::Utf8(err)
    }
}

impl From<io::Error> for ReadError {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

impl Display for ReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

impl std::error::Error for ReadError {}

/// Read result.
pub type ReadResult<T> = std::result::Result<T, ReadError>;

/// Error while writing.
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
        todo!()
    }
}

impl std::error::Error for WriteError {}

/// Write result.
pub type WriteResult = std::result::Result<(), WriteError>;
