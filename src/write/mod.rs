mod writer;

pub(crate) use writer::{IdState, NodeState, Writer};

use std::error;
use std::fmt::{self, Display};
use std::result;

/// Write error.
#[derive(Debug)]
pub struct Error(pub(crate) String);

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl error::Error for Error {}

/// Write result.
pub type Result = result::Result<(), Error>;
