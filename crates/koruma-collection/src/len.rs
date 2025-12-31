//! Length validation for collections.
//!
//! This module provides:
//! - `HasLen` trait for types that have a measurable length
//! - `LenValidation` validator with min/max bounds
//!
//! # Example
//! ```ignore
//! use koruma::Koruma;
//! use koruma_collection::len::{HasLen, LenValidation};
//!
//! #[derive(Koruma)]
//! struct Order {
//!     #[koruma(LenValidation<_>(min = 1, max = 5))]
//!     items: Vec<String>,
//! }
//! ```

use koruma::{KorumaResult, Validate, validator};

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

/// Validates that a collection's length is within the specified bounds.
///
/// Works with any type that implements `HasLen + Clone`.
#[validator]
#[derive(Debug)]
#[cfg_attr(feature = "fluent", derive(es_fluent::EsFluent))]
pub struct LenValidation<T> {
    /// Minimum allowed length (inclusive)
    pub min: usize,
    /// Maximum allowed length (inclusive)
    pub max: usize,
    /// The collection being validated (stored for error context)
    #[koruma(value)]
    pub actual: T,
}

// Manual Clone impl to avoid bound on struct definition
impl<T: Clone> Clone for LenValidation<T> {
    fn clone(&self) -> Self {
        Self {
            min: self.min,
            max: self.max,
            actual: self.actual.clone(),
        }
    }
}

impl<T: HasLen + Clone> Validate<T> for LenValidation<T> {
    fn validate(&self, value: &T) -> KorumaResult {
        let len = value.len();
        if len < self.min || len > self.max {
            Err(())
        } else {
            Ok(())
        }
    }
}

impl<T: HasLen + Clone> LenValidation<T> {
    /// Get the actual length of the collection for error reporting.
    pub fn actual_len(&self) -> usize {
        self.actual.len()
    }
}

// Display implementation for fmt feature
#[cfg(feature = "fmt")]
impl<T: HasLen + Clone> std::fmt::Display for LenValidation<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "length {} is not within bounds [{}, {}]",
            self.actual.len(),
            self.min,
            self.max
        )
    }
}
