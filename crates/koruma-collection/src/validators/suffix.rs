//! Suffix validation for koruma.
//!
//! This module provides:
//! - `SuffixValidation` validator to check if a string ends with a specified suffix
//!
//! # Example
//! ```ignore
//! use koruma::Koruma;
//! use koruma_collection::validators::suffix::SuffixValidation;
//!
//! #[derive(Koruma)]
//! struct File {
//!     #[koruma(SuffixValidation<_>(suffix = ".txt"))]
//!     name: String,
//! }
//! ```

use koruma::{KorumaResult, Validate, validator};

/// Validates that a string ends with a specified suffix.
#[validator]
#[derive(Clone, Debug)]
#[cfg_attr(feature = "fluent", derive(es_fluent::EsFluent))]
pub struct SuffixValidation<T: AsRef<str>> {
    /// The suffix to check for
    pub suffix: String,
    /// The string being validated (stored for error context)
    #[koruma(value)]
    #[cfg_attr(feature = "fluent", fluent(value(|x: &T| x.as_ref().to_string())))]
    pub actual: T,
}

impl<T: AsRef<str>> Validate<T> for SuffixValidation<T> {
    fn validate(&self, value: &T) -> KorumaResult {
        let s = value.as_ref();
        if s.ends_with(&self.suffix) {
            Ok(())
        } else {
            Err(())
        }
    }
}

#[cfg(feature = "fmt")]
impl<T: AsRef<str>> std::fmt::Display for SuffixValidation<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "value does not end with \"{}\"", self.suffix)
    }
}