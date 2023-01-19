use std::ops::{Add, Deref};
use std::rc::Rc;

/// A 3-dimensional vector.
#[derive(Clone, PartialEq, Eq, Hash, Default, Debug)]
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

impl Copy for Vec3<u8> {}

/// Reference to a file.
#[derive(Clone, Debug)]
pub enum FileRef {
    /// Reference to an internal file.
    Internal {
        /// Internal path to the file.
        path: String,
    },
    /// Reference to an external file.
    External {
        /// Hash of the file.
        hash: [u8; 32],
        /// Internal path to the file.
        path: String,
        /// External URL from where the file can be downloaded.
        locator_url: String,
    },
}

/// Reference counted, immutable string.
#[derive(Clone, Default)]
#[repr(transparent)]
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
