//! Alphanumeric validation for koruma.
//!
//! This module provides:
//! - `AlphanumericValidation` validator to check if a string contains only alphanumeric characters
//!
//! # Example
//! ```ignore
//! use koruma::Koruma;
//! use koruma_collection::validators::alphanumeric::AlphanumericValidation;
//!
//! #[derive(Koruma)]
//! struct User {
//!     #[koruma(AlphanumericValidation<_>)]
//!     username: String,
//! }
//! ```

use koruma::{Validate, validator};

/// Validates that a string contains only alphanumeric characters.
#[validator]
#[derive(Clone, Debug)]
#[cfg_attr(feature = "fluent", derive(es_fluent::EsFluent))]
pub struct AlphanumericValidation<T: AsRef<str>> {
    /// The string being validated (stored for error context)
    #[koruma(value)]
    #[cfg_attr(feature = "fluent", fluent(value(|x: &T| x.as_ref().to_string())))]
    pub actual: T,
}

impl<T: AsRef<str>> Validate<T> for AlphanumericValidation<T> {
    fn validate(&self, value: &T) -> bool {
        let s = value.as_ref();
        if s.chars().all(|c| c.is_alphanumeric()) {
            true
        } else {
            false
        }
    }
}

#[cfg(feature = "fmt")]
impl<T: AsRef<str>> std::fmt::Display for AlphanumericValidation<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "value contains non-alphanumeric characters")
    }
}
