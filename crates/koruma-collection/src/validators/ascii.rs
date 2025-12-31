//! ASCII validation for koruma.
//!
//! This module provides:
//! - `AsciiValidation` validator to check if a string contains only ASCII characters
//!
//! # Example
//! ```ignore
//! use koruma::Koruma;
//! use koruma_collection::validators::ascii::AsciiValidation;
//!
//! #[derive(Koruma)]
//! struct User {
//!     #[koruma(AsciiValidation<_>)]
//!     username: String,
//! }
//! ```

use koruma::{KorumaResult, Validate, validator};

/// Validates that a string contains only ASCII characters.
#[validator]
#[derive(Clone, Debug)]
#[cfg_attr(feature = "fluent", derive(es_fluent::EsFluent))]
pub struct AsciiValidation<T: AsRef<str>> {
    /// The string being validated (stored for error context)
    #[koruma(value)]
    #[cfg_attr(feature = "fluent", fluent(value(|x: &T| x.as_ref().to_string())))]
    pub actual: T,
}

impl<T: AsRef<str>> Validate<T> for AsciiValidation<T> {
    fn validate(&self, value: &T) -> KorumaResult {
        let s = value.as_ref();
        if s.is_ascii() {
            Ok(())
        } else {
            Err(())
        }
    }
}

#[cfg(feature = "fmt")]
impl<T: AsRef<str>> std::fmt::Display for AsciiValidation<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "value contains non-ASCII characters")
    }
}