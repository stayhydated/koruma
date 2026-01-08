//! Non-empty validation for strings and collections.
//!
//! This module provides:
//! - `NonEmptyValidation` validator to check if a string/collection is not empty
//!
//! This is a convenience validator that wraps length checking with a min of 1.
//!
//! # Example
//! ```ignore
//! use koruma::Koruma;
//! use koruma_collection::validators::non_empty::NonEmptyValidation;
//!
//! #[derive(Koruma)]
//! struct User {
//!     #[koruma(NonEmptyValidation<_>)]
//!     name: String,
//! }
//! ```

use koruma::{Validate, validator};

use super::HasLen;

/// Validates that a string or collection is not empty.
///
/// Works with any type that implements `HasLen + Clone`.
#[validator]
#[derive(Clone, Debug)]
#[cfg_attr(feature = "fluent", derive(es_fluent::EsFluent))]
pub struct NonEmptyValidation<T: HasLen> {
    /// The value being validated (stored for error context)
    #[koruma(value)]
    #[cfg_attr(feature = "fluent", fluent(skip))]
    pub actual: T,
}

impl<T: HasLen + Clone> Validate<T> for NonEmptyValidation<T> {
    fn validate(&self, value: &T) -> bool {
        !value.is_empty()
    }
}

#[cfg(feature = "fmt")]
impl<T: HasLen + Clone> std::fmt::Display for NonEmptyValidation<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "value must not be empty")
    }
}
