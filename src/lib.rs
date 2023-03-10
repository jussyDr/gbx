//! A (incomplete) GameBox (.Gbx) file reader and writer for Trackmania (2020).
//!
//! GBX files are serialized instances (nodes) of game classes found in the TrackMania games.
//! For more info on the GBX format check out <https://wiki.xaseco.org/wiki/GBX>.
//! For a more complete GBX file reader and writer check out [GBX.NET](https://github.com/BigBang1112/gbx-net).

/// Types for reading GBX nodes.
pub mod read;
/// Types for writing GBX nodes.
pub mod write;

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

mod fmt;
mod types;

pub use block::Block;
pub use classes::{block, ghost, item, map, model};
pub use ghost::Ghost;
pub use item::Item;
pub use map::Map;
pub use types::{ExternalFileRef, FileRef, Id, InternalFileRef, Rgb, Vec3};
