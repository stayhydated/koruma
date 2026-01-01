//! Field matching validation for koruma.
//!
//! This module provides:
//! - `MatchesValidation` validator to check if a value matches another field
//!
//! # Example
//! ```ignore
//! use koruma::Koruma;
//! use koruma_collection::validators::matches::MatchesValidation;
//!
//! #[derive(Koruma)]
//! struct User {
//!     password: String,
//!     #[koruma(MatchesValidation<_>(other = password))]
//!     confirm_password: String,
//! }
//! ```

use koruma::{KorumaResult, Validate, validator};

/// Validates that a value matches another value.
#[validator]
#[derive(Clone, Debug)]
#[cfg_attr(feature = "fluent", derive(es_fluent::EsFluent))]
pub struct MatchesValidation<T: std::fmt::Display + Clone> {
    /// The value to match against
    #[cfg_attr(feature = "fluent", fluent(value(|x: &T| x.to_string())))]
    pub other: T,
    /// The value being validated (stored for error context)
    #[koruma(value)]
    #[cfg_attr(feature = "fluent", fluent(value(|x: &T| x.to_string())))]
    pub actual: T,
}

impl<T: PartialEq + std::fmt::Display + Clone> Validate<T> for MatchesValidation<T> {
    fn validate(&self, value: &T) -> KorumaResult {
        if value == &self.other {
            Ok(())
        } else {
            Err(())
        }
    }
}

#[cfg(feature = "fmt")]
impl<T: PartialEq + std::fmt::Debug + std::fmt::Display + Clone> std::fmt::Display
    for MatchesValidation<T>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "value does not match expected value")
    }
}
