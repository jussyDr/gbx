use std::borrow::Borrow;
use std::fmt::{self, Debug, Display};
use std::hash::{Hash, Hasher};
use std::ops::{Add, Deref};
use std::path::{Path, PathBuf};
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

impl<T> From<[T; 3]> for Vec3<T>
where
    T: Copy,
{
    fn from(arr: [T; 3]) -> Self {
        Self::new(arr[0], arr[1], arr[2])
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

/// RGB color.
#[derive(Clone, Debug)]
pub struct Rgb {
    /// Red. [0.0, 1.0]
    pub red: f32,
    /// Green. [0.0, 1.0]
    pub green: f32,
    /// Blue. [0.0, 1.0]
    pub blue: f32,
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
    /// Hash digest of the file created using SHA-256.
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

    /// Converts the file ref to an `ExternalFileRef` if external, else returns `None`.
    pub fn external(self) -> Option<ExternalFileRef> {
        match self {
            FileRef::Internal(_) => None,
            FileRef::External(external_file_ref) => Some(external_file_ref),
        }
    }

    /// Internal path to the file.
    pub fn path(&self) -> &Path {
        match *self {
            FileRef::Internal(InternalFileRef { ref path }) => path,
            FileRef::External(ExternalFileRef { ref path, .. }) => path,
        }
    }
}

/// Reference counted, immutable string.
#[derive(Clone, Default)]
pub struct Id(Option<Rc<str>>);

impl Id {
    /// Create a new reference counted string.
    pub fn new(s: String) -> Self {
        Self(Some(s.into()))
    }

    /// Create an empty `Id`.
    ///
    /// This function does not allocate, and the resulting `Id` is not actually reference counted.
    pub const fn empty() -> Self {
        Self(None)
    }

    /// Extract the string slice.
    pub fn as_str(&self) -> &str {
        self
    }
}

impl Deref for Id {
    type Target = str;

    fn deref(&self) -> &str {
        match self.0 {
            None => "",
            Some(ref rc) => rc.deref(),
        }
    }
}

impl Borrow<str> for Id {
    fn borrow(&self) -> &str {
        match self.0 {
            None => "",
            Some(ref rc) => rc.borrow(),
        }
    }
}

impl PartialEq for Id {
    fn eq(&self, other: &Self) -> bool {
        self.as_str().eq(other.as_str())
    }
}

impl Eq for Id {}

impl Hash for Id {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_str().hash(state);
    }
}

impl Debug for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self(Some(ref str)) => Debug::fmt(str, f),
            Self(None) => Debug::fmt("", f),
        }
    }
}

impl Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self(Some(ref str)) => Display::fmt(str, f),
            Self(None) => Display::fmt("", f),
        }
    }
}
