//! Collection validation validators.
//!
//! This module contains validators for collection-based validation scenarios.

/// Trait for types that have a measurable length.
///
/// Implemented for common std collections. Users can implement
/// this for custom collection types.
pub trait HasLen {
    fn len(&self) -> usize;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

// Implementations for std collections
impl<T> HasLen for Vec<T> {
    fn len(&self) -> usize {
        self.len()
    }
}

impl<T> HasLen for std::collections::VecDeque<T> {
    fn len(&self) -> usize {
        self.len()
    }
}

impl<K, V> HasLen for std::collections::HashMap<K, V> {
    fn len(&self) -> usize {
        self.len()
    }
}

impl<K, V> HasLen for std::collections::BTreeMap<K, V> {
    fn len(&self) -> usize {
        self.len()
    }
}

impl<T> HasLen for std::collections::HashSet<T> {
    fn len(&self) -> usize {
        self.len()
    }
}

impl<T> HasLen for std::collections::BTreeSet<T> {
    fn len(&self) -> usize {
        self.len()
    }
}

impl HasLen for String {
    fn len(&self) -> usize {
        self.len()
    }
}

impl HasLen for str {
    fn len(&self) -> usize {
        self.len()
    }
}

impl<T> HasLen for [T] {
    fn len(&self) -> usize {
        self.len()
    }
}

impl<T, const N: usize> HasLen for [T; N] {
    fn len(&self) -> usize {
        N
    }
}

#[cfg(feature = "smallvec")]
impl<T, const N: usize> HasLen for smallvec::SmallVec<[T; N]> {
    fn len(&self) -> usize {
        N
    }
}

mod len;
mod non_empty;

pub use len::LenValidation;
pub use non_empty::NonEmptyValidation;
