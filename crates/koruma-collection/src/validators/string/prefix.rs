//! Prefix validation for koruma.
//!
//! This module provides:
//! - `PrefixValidation` validator to check if a string starts with a specified prefix
//!
//! # Example
//! ```rust
//! use koruma::Koruma;
//! use koruma_collection::string::PrefixValidation;
//!
//! #[derive(Koruma)]
//! struct Config {
//!     #[koruma(PrefixValidation::<_>(prefix = "config_"))]
//!     key: String,
//! }
//! ```

use koruma::{Validate, validator};

/// Validates that a string starts with a specified prefix.
#[validator]
#[cfg_attr(feature = "showcase", showcase(
    name = "Prefix 'hello'",
    description = "Validates that the input starts with 'hello'",
    create = |input: &str| {
        PrefixValidation::builder()
            .prefix("hello")
            .with_value(input.to_string())
            .build()
    }
))]
#[derive(Clone, Debug)]
#[cfg_attr(feature = "fluent", derive(es_fluent::EsFluent))]
pub struct PrefixValidation<T: AsRef<str>> {
    /// The prefix to check for
    #[builder(into)]
    pub prefix: String,
    /// The string being validated (stored for error context)
    #[koruma(value)]
    #[cfg_attr(feature = "fluent", fluent(value(|x: &T| x.as_ref().to_string())))]
    pub actual: T,
}

impl<T: AsRef<str>> Validate<T> for PrefixValidation<T> {
    fn validate(&self, value: &T) -> bool {
        let s = value.as_ref();
        s.starts_with(&self.prefix)
    }
}

#[cfg(feature = "fmt")]
impl<T: AsRef<str>> std::fmt::Display for PrefixValidation<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "value does not start with \"{}\"", self.prefix)
    }
}
