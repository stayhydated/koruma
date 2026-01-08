//! Contains validation for koruma.
//!
//! This module provides:
//! - `ContainsValidation` validator to check if a string contains a specified substring
//!
//! # Example
//! ```ignore
//! use koruma::Koruma;
//! use koruma_collection::validators::contains::ContainsValidation;
//!
//! #[derive(Koruma)]
//! struct User {
//!     #[koruma(ContainsValidation<_>(substring = "test"))]
//!     email: String,
//! }
//! ```

use koruma::{Validate, validator};

/// Validates that a string contains a specified substring.
#[validator]
#[cfg_attr(feature = "showcase", showcase(
    name = "Contains 'test'",
    description = "Validates that the input contains the substring 'test'",
    create = |input: &str| {
        ContainsValidation::builder()
            .substring("test".to_string())
            .with_value(input.to_string())
            .build()
    }
))]
#[derive(Clone, Debug)]
#[cfg_attr(feature = "fluent", derive(es_fluent::EsFluent))]
pub struct ContainsValidation<T: AsRef<str>> {
    /// The substring to search for
    pub substring: String,
    /// The string being validated (stored for error context)
    #[koruma(value)]
    #[cfg_attr(feature = "fluent", fluent(value(|x: &T| x.as_ref().to_string())))]
    pub actual: T,
}

impl<T: AsRef<str>> Validate<T> for ContainsValidation<T> {
    fn validate(&self, value: &T) -> bool {
        let s = value.as_ref();
        s.contains(&self.substring)
    }
}

#[cfg(feature = "fmt")]
impl<T: AsRef<str>> std::fmt::Display for ContainsValidation<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "value does not contain \"{}\"", self.substring)
    }
}
