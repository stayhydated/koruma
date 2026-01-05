//! Pattern validation for koruma.
//!
//! This module provides:
//! - `PatternValidation` validator to check if a string matches a regular expression pattern
//!
//! # Example
//! ```ignore
//! use koruma::Koruma;
//! use koruma_collection::validators::pattern::PatternValidation;
//!
//! #[derive(Koruma)]
//! struct User {
//!     #[koruma(PatternValidation<_>(pattern = r"^[a-zA-Z0-9_]+$"))]
//!     username: String,
//! }
//! ```

use koruma::{Validate, validator};

/// Validates that a string matches a regular expression pattern.
#[validator]
#[derive(Clone, Debug)]
#[cfg_attr(feature = "fluent", derive(es_fluent::EsFluent))]
pub struct PatternValidation<T: AsRef<str>> {
    /// The regex pattern to match against
    pub pattern: String,
    /// The string being validated (stored for error context)
    #[koruma(value)]
    #[cfg_attr(feature = "fluent", fluent(value(|x: &T| x.as_ref().to_string())))]
    pub actual: T,
}

impl<T: AsRef<str>> Validate<T> for PatternValidation<T> {
    fn validate(&self, value: &T) -> bool {
        let s = value.as_ref();
        match regex::Regex::new(&self.pattern) {
            Ok(re) => {
                if re.is_match(s) {
                    true
                } else {
                    false
                }
            },
            Err(_) => false, // Invalid regex pattern
        }
    }
}

#[cfg(feature = "fmt")]
impl<T: AsRef<str>> std::fmt::Display for PatternValidation<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "value does not match pattern /{}/", self.pattern)
    }
}
