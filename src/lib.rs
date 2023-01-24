//! A (incomplete) GameBox (.Gbx) file reader and writer for Trackmania (2020).

/// Error types.
pub mod error;

mod classes {
    /// Types for `Ghost`.
    pub mod ghost;
    /// Types for `Map`.
    pub mod map;

    pub mod item_model;
}

mod gbx;
mod header;
mod reader;
mod ref_table;
mod types;

pub use classes::{
    ghost,
    item_model::{block, item},
    map,
};

pub use block::Block;
pub use ghost::Ghost;
pub use item::Item;
pub use map::Map;
pub use types::{ExternalFileRef, FileRef, InternalFileRef, RcStr, Vec3};
