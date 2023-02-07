use std::fmt::{self, Debug};

pub struct DebugOption<'a, T>(pub &'a Option<T>);

impl<T> Debug for DebugOption<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            Some(..) => f.write_str("Some(..)"),
            None => f.write_str("None"),
        }
    }
}

pub struct DebugVec<'a, T>(pub &'a Vec<T>);

impl<T> Debug for DebugVec<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("[..; {}]", self.0.len()))
    }
}
