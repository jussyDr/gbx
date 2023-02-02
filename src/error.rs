use std::error;
use std::fmt::{self, Display};
use std::result;

/// Read error.
#[derive(Debug)]
pub struct ReadError(pub(crate) String);

impl Display for ReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl error::Error for ReadError {}

/// Read result.
pub type ReadResult<T> = result::Result<T, ReadError>;

/// Write error.
#[derive(Debug)]
pub struct WriteError(pub(crate) String);

impl Display for WriteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl error::Error for WriteError {}

/// Write result.
pub type WriteResult = result::Result<(), WriteError>;
