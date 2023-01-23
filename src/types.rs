use std::ops::{Add, Deref};
use std::path::PathBuf;
use std::rc::Rc;

/// A 3-dimensional vector.
#[derive(Clone, PartialEq, Eq, Hash, Default, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Vec3<T> {
    /// X component.
    pub x: T,
    /// Y component.
    pub y: T,
    /// Z component.
    pub z: T,
}

impl<T> Vec3<T> {
    /// Create a new vector.
    pub fn new(x: T, y: T, z: T) -> Self {
        Self { x, y, z }
    }
}

impl Copy for Vec3<u8> {}

impl<T> Add for Vec3<T>
where
    T: Add<Output = T>,
{
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

/// Reference to an internal file.
#[derive(Clone, Debug)]
pub struct InternalFileRef {
    /// Internal path to the file.
    pub path: PathBuf,
}

/// Reference to an external file.
#[derive(Clone, Debug)]
pub struct ExternalFileRef {
    /// Hash of the file.
    pub hash: [u8; 32],
    /// Internal path to the file.
    pub path: PathBuf,
    /// External URL from where the file can be downloaded.
    pub locator_url: String,
}

/// Reference to a file.
#[derive(Clone, Debug)]
pub enum FileRef {
    /// Reference to an internal file.
    Internal(InternalFileRef),
    /// Reference to an external file.
    External(ExternalFileRef),
}

impl FileRef {
    /// Converts the file ref to an `InternalFileRef` if internal, else returns `None`.
    pub fn internal(self) -> Option<InternalFileRef> {
        match self {
            FileRef::Internal(internal_file_ref) => Some(internal_file_ref),
            FileRef::External(_) => None,
        }
    }
}

/// Reference counted, immutable string.
#[derive(Clone, Default)]
pub struct RcStr(Option<Rc<str>>);

impl RcStr {
    /// Create a new reference counted string.
    pub fn new(s: String) -> Self {
        Self(Some(s.into()))
    }

    /// Create an empty reference counted string.
    pub const fn empty() -> Self {
        Self(None)
    }

    /// Extract the string slice.
    pub fn as_str(&self) -> &str {
        self
    }
}

impl Deref for RcStr {
    type Target = str;

    fn deref(&self) -> &str {
        match self.0 {
            None => "",
            Some(ref rc) => rc.deref(),
        }
    }
}
