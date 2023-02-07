mod reader;

pub(crate) use reader::{IdState, NodeState, Reader};

use std::error;
use std::fmt::{self, Display};
use std::result;

/// Read error.
#[derive(Debug)]
pub struct Error(pub(crate) String);

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl error::Error for Error {}

/// Read result.
pub type Result<T> = result::Result<T, Error>;
