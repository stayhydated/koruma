//! Positive number validation for koruma.
//!
//! This module provides:
//! - `PositiveValidation` validator to check if a numeric value is strictly positive (> 0)
//!
//! # Example
//! ```ignore
//! use koruma::Koruma;
//! use koruma_collection::validators::positive::PositiveValidation;
//!
//! #[derive(Koruma)]
//! struct Order {
//!     #[koruma(PositiveValidation<_>)]
//!     quantity: i32,
//! }
//! ```

use koruma::{Validate, validator};

use super::Numeric;

/// Validates that a numeric value is strictly positive (> 0).
#[validator]
#[cfg_attr(feature = "showcase", showcase(
    name = "Positive Number",
    description = "Validates that the input is a positive number (> 0)",
    input_type = Numeric,
    create = |input: &str| {
        let num = input.parse::<f64>().unwrap_or(0.0);
        PositiveValidation::builder()
            .with_value(num)
            .build()
    }
))]
#[derive(Clone, Debug)]
#[cfg_attr(feature = "fluent", derive(es_fluent::EsFluent))]
pub struct PositiveValidation<T: Numeric> {
    /// The value being validated (stored for error context)
    #[koruma(value)]
    #[cfg_attr(feature = "fluent", fluent(value(|x: &T| x.to_string())))]
    pub actual: T,
}

impl<T: Numeric> Validate<T> for PositiveValidation<T> {
    fn validate(&self, value: &T) -> bool {
        *value > T::default()
    }
}

#[cfg(feature = "fmt")]
impl<T: Numeric> std::fmt::Display for PositiveValidation<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "value {} must be positive (> 0)", self.actual)
    }
}
