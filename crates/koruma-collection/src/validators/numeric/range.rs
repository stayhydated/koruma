//! Range validation for koruma.
//!
//! This module provides:
//! - `RangeValidation` validator to check if a numeric value is within specified bounds
//!
//! # Example
//! ```rust
//! use koruma::Koruma;
//! use koruma_collection::numeric::RangeValidation;
//!
//! #[derive(Koruma)]
//! struct Score {
//!     #[koruma(RangeValidation::<_>(min = 0, max = 100))]
//!     value: u32,
//! }
//! ```

use koruma::{Validate, validator};

/// Validates that a numeric value is within specified bounds.
#[validator]
#[cfg_attr(feature = "showcase", showcase(
    name = "Range [0, 100]",
    description = "Validates that the input is a number between 0 and 100",
    input_type = Numeric,
    create = |input: &str| {
        let num = input.parse::<f64>().unwrap_or(0.0);
        RangeValidation::builder()
            .min(0_f64)
            .max(100_f64)
            .with_value(num)
            .build()
    }
))]
#[derive(Clone, Debug)]
#[cfg_attr(feature = "fluent", derive(es_fluent::EsFluent))]
pub struct RangeValidation<T: PartialOrd + Copy + std::fmt::Display + Clone> {
    /// Minimum allowed value (inclusive)
    #[cfg_attr(feature = "fluent", fluent(value(|x: &T| x.to_string())))]
    pub min: T,
    /// Whether the minimum value is exclusive
    #[cfg_attr(feature = "fluent", fluent(skip))]
    #[builder(default = false)]
    pub exclusive_min: bool,
    /// Maximum allowed value (inclusive)
    #[cfg_attr(feature = "fluent", fluent(value(|x: &T| x.to_string())))]
    pub max: T,
    /// Whether the maximum value is exclusive
    #[cfg_attr(feature = "fluent", fluent(skip))]
    #[builder(default = false)]
    pub exclusive_max: bool,
    /// The value being validated (stored for error context)
    #[koruma(value)]
    #[cfg_attr(feature = "fluent", fluent(value(|x: &T| x.to_string())))]
    pub actual: T,
}

impl<T: PartialOrd + Copy + std::fmt::Display> Validate<T> for RangeValidation<T> {
    fn validate(&self, value: &T) -> bool {
        let lower_ok = if self.exclusive_min {
            *value > self.min
        } else {
            *value >= self.min
        };

        let upper_ok = if self.exclusive_max {
            *value < self.max
        } else {
            *value <= self.max
        };

        lower_ok && upper_ok
    }
}

#[cfg(feature = "fmt")]
impl<T: PartialOrd + Copy + std::fmt::Display + Clone> std::fmt::Display for RangeValidation<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "value {} is not within bounds [{}, {}]",
            self.actual, self.min, self.max
        )
    }
}
