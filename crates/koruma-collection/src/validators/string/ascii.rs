//! ASCII validation for koruma.
//!
//! This module provides:
//! - `AsciiValidation` validator to check if a string contains only ASCII characters
//!
//! # Example
//! ```rust
//! use koruma::Koruma;
//! use koruma_collection::string::AsciiValidation;
//!
//! #[derive(Koruma)]
//! struct User {
//!     #[koruma(AsciiValidation::<_>)]
//!     username: String,
//! }
//! ```

use koruma::{Validate, validator};

/// Validates that a string contains only ASCII characters.
#[validator]
#[cfg_attr(feature = "showcase", showcase(
    name = "ASCII",
    description = "Validates that the input contains only ASCII characters",
    create = |input: &str| {
        AsciiValidation::builder()
            .with_value(input.to_string())
            .build()
    }
))]
#[derive(Clone, Debug)]
#[cfg_attr(feature = "fluent", derive(es_fluent::EsFluent))]
pub struct AsciiValidation<T: AsRef<str>> {
    /// The string being validated (stored for error context)
    #[koruma(value)]
    #[cfg_attr(feature = "fluent", fluent(value(|x: &T| x.as_ref().to_string())))]
    pub actual: T,
}

impl<T: AsRef<str>> Validate<T> for AsciiValidation<T> {
    fn validate(&self, value: &T) -> bool {
        let s = value.as_ref();
        s.is_ascii()
    }
}

#[cfg(feature = "fmt")]
impl<T: AsRef<str>> std::fmt::Display for AsciiValidation<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "value contains non-ASCII characters")
    }
}
