//! Required validation for koruma.
//!
//! This module provides:
//! - `RequiredValidation` validator to check if a value is present (not None for Option types)
//!
//! # Example
//! ```ignore
//! use koruma::Koruma;
//! use koruma_collection::validators::required::RequiredValidation;
//!
//! #[derive(Koruma)]
//! struct User {
//!     #[koruma(RequiredValidation<_>)]
//!     name: Option<String>,
//! }
//! ```

use koruma::{KorumaResult, Validate, validator};

/// Validates that a value is present (not None for Option types).
#[validator]
#[derive(Clone, Debug)]
#[cfg_attr(feature = "fluent", derive(es_fluent::EsFluent))]
pub struct RequiredValidation<T> {
    /// The value being validated (stored for error context)
    #[koruma(value)]
    #[cfg_attr(feature = "fluent", fluent(skip))]
    pub actual: Option<T>,
}

impl<T> Validate<Option<T>> for RequiredValidation<Option<T>> {
    fn validate(&self, value: &Option<T>) -> KorumaResult {
        if value.is_some() { Ok(()) } else { Err(()) }
    }
}

#[cfg(feature = "fmt")]
impl<T> std::fmt::Display for RequiredValidation<Option<T>> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "value is required but not present")
    }
}
