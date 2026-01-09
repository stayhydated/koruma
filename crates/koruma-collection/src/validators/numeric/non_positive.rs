//! Non-positive number validation for koruma.
//!
//! This module provides:
//! - `NonPositiveValidation` validator to check if a numeric value is <= 0
//!
//! # Example
//! ```rust
//! use koruma::Koruma;
//! use koruma_collection::numeric::NonPositiveValidation;
//!
//! #[derive(Koruma)]
//! struct Debit {
//!     #[koruma(NonPositiveValidation::<_>)]
//!     amount: f64,
//! }
//! ```

use koruma::{Validate, validator};

use super::Numeric;

/// Validates that a numeric value is non-positive (<= 0).
#[validator]
#[cfg_attr(feature = "showcase", showcase(
    name = "Non-Positive Number",
    description = "Validates that the input is a non-positive number (<= 0)",
    input_type = Numeric,
    create = |input: &str| {
        let num = input.parse::<f64>().unwrap_or(0.0);
        NonPositiveValidation::builder()
            .with_value(num)
            .build()
    }
))]
#[derive(Clone, Debug)]
#[cfg_attr(feature = "fluent", derive(es_fluent::EsFluent))]
pub struct NonPositiveValidation<T: Numeric> {
    /// The value being validated (stored for error context)
    #[koruma(value)]
    #[cfg_attr(feature = "fluent", fluent(value(|x: &T| x.to_string())))]
    pub actual: T,
}

impl<T: Numeric> Validate<T> for NonPositiveValidation<T> {
    fn validate(&self, value: &T) -> bool {
        *value <= T::default()
    }
}

#[cfg(feature = "fmt")]
impl<T: Numeric> std::fmt::Display for NonPositiveValidation<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "value {} must be non-positive (<= 0)", self.actual)
    }
}
