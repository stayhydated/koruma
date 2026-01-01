//! Range validation for koruma.
//!
//! This module provides:
//! - `RangeValidation` validator to check if a numeric value is within specified bounds
//!
//! # Example
//! ```ignore
//! use koruma::Koruma;
//! use koruma_collection::validators::range::RangeValidation;
//!
//! #[derive(Koruma)]
//! struct Score {
//!     #[koruma(RangeValidation<_>(min = 0, max = 100))]
//!     value: u32,
//! }
//! ```

use koruma::{KorumaResult, Validate, validator};

/// Validates that a numeric value is within specified bounds.
#[validator]
#[derive(Clone, Debug)]
#[cfg_attr(feature = "fluent", derive(es_fluent::EsFluent))]
pub struct RangeValidation<T: PartialOrd + Copy + std::fmt::Display + Clone> {
    /// Minimum allowed value (inclusive)
    #[cfg_attr(feature = "fluent", fluent(value(|x: &T| x.to_string())))]
    pub min: T,
    /// Maximum allowed value (inclusive)
    #[cfg_attr(feature = "fluent", fluent(value(|x: &T| x.to_string())))]
    pub max: T,
    /// The value being validated (stored for error context)
    #[koruma(value)]
    #[cfg_attr(feature = "fluent", fluent(value(|x: &T| x.to_string())))]
    pub actual: T,
}

impl<T: PartialOrd + Copy + std::fmt::Display> Validate<T> for RangeValidation<T> {
    fn validate(&self, value: &T) -> KorumaResult {
        if *value >= self.min && *value <= self.max {
            Ok(())
        } else {
            Err(())
        }
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
