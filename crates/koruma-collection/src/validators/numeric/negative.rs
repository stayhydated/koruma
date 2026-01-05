//! Negative number validation for koruma.
//!
//! This module provides:
//! - `NegativeValidation` validator to check if a numeric value is strictly negative (< 0)
//!
//! # Example
//! ```ignore
//! use koruma::Koruma;
//! use koruma_collection::validators::negative::NegativeValidation;
//!
//! #[derive(Koruma)]
//! struct Temperature {
//!     #[koruma(NegativeValidation<_>)]
//!     celsius: f64,
//! }
//! ```

use koruma::{Validate, validator};

use super::Numeric;

/// Validates that a numeric value is strictly negative (< 0).
#[validator]
#[derive(Clone, Debug)]
#[cfg_attr(feature = "fluent", derive(es_fluent::EsFluent))]
pub struct NegativeValidation<T: Numeric> {
    /// The value being validated (stored for error context)
    #[koruma(value)]
    #[cfg_attr(feature = "fluent", fluent(value(|x: &T| x.to_string())))]
    pub actual: T,
}

impl<T: Numeric> Validate<T> for NegativeValidation<T> {
    fn validate(&self, value: &T) -> bool {
        *value < T::default()
    }
}

#[cfg(feature = "fmt")]
impl<T: Numeric> std::fmt::Display for NegativeValidation<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "value {} must be negative (< 0)", self.actual)
    }
}
