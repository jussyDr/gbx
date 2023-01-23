//! A (incomplete) GameBox (.Gbx) file reader and writer for Trackmania (2020).

/// Error types.
pub mod error;

mod classes {
    /// Types for `Block`.
    pub mod block;
    /// Types for `Ghost`.
    pub mod ghost;
    /// Types for `Map`.
    pub mod map;
}

mod gbx;
mod header;
mod reader;
mod ref_table;
mod types;

pub use block::Block;
pub use classes::{block, ghost, map};
pub use ghost::Ghost;
pub use map::Map;
pub use types::{ExternalFileRef, FileRef, InternalFileRef, RcStr, Vec3};
