//! Prefix validation for koruma.
//!
//! This module provides:
//! - `PrefixValidation` validator to check if a string starts with a specified prefix
//!
//! # Example
//! ```ignore
//! use koruma::Koruma;
//! use koruma_collection::validators::prefix::PrefixValidation;
//!
//! #[derive(Koruma)]
//! struct Config {
//!     #[koruma(PrefixValidation<_>(prefix = "config_"))]
//!     key: String,
//! }
//! ```

use koruma::{KorumaResult, Validate, validator};

/// Validates that a string starts with a specified prefix.
#[validator]
#[derive(Clone, Debug)]
#[cfg_attr(feature = "fluent", derive(es_fluent::EsFluent))]
pub struct PrefixValidation<T: AsRef<str>> {
    /// The prefix to check for
    pub prefix: String,
    /// The string being validated (stored for error context)
    #[koruma(value)]
    #[cfg_attr(feature = "fluent", fluent(value(|x: &T| x.as_ref().to_string())))]
    pub actual: T,
}

impl<T: AsRef<str>> Validate<T> for PrefixValidation<T> {
    fn validate(&self, value: &T) -> KorumaResult {
        let s = value.as_ref();
        if s.starts_with(&self.prefix) {
            Ok(())
        } else {
            Err(())
        }
    }
}

#[cfg(feature = "fmt")]
impl<T: AsRef<str>> std::fmt::Display for PrefixValidation<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "value does not start with \"{}\"", self.prefix)
    }
}
