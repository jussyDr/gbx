//! A (incomplete) GameBox (.Gbx) file reader and writer for Trackmania (2020).

/// Error types.
pub mod error;

mod classes {
    /// Types for `Block`.
    pub mod block;
    /// Types for `Ghost`.
    pub mod ghost;
    /// Types for `Item`.
    pub mod item;
    /// Types for `Map`.
    pub mod map;
    /// Types for `Model`.
    pub mod model;
}

mod gbx;
mod reader;
mod types;
mod writer;

pub use block::Block;
pub use classes::{block, ghost, item, map, model};
pub use ghost::Ghost;
pub use item::Item;
pub use map::Map;
pub use types::{ExternalFileRef, FileRef, InternalFileRef, RcStr, Rgb, Vec3};
