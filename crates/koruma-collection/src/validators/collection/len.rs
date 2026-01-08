//! Length validation for collections.
//!
//! This module provides:
//! - `LenValidation` validator with min/max bounds
//!
//! # Example
//! ```ignore
//! use koruma::Koruma;
//! use koruma_collection::validators::collection::len::LenValidation;
//!
//! #[derive(Koruma)]
//! struct Order {
//!     #[koruma(LenValidation<_>(min = 1, max = 5))]
//!     items: Vec<String>,
//! }
//! ```

use koruma::{Validate, validator};

use super::HasLen;

/// Validates that a collection's length is within the specified bounds.
///
/// Works with any type that implements `HasLen + Clone`.
#[validator]
#[cfg_attr(feature = "showcase", showcase(
    name = "Length",
    description = "Validates string length is between 1 and 10",
    create = |input: &str| {
        LenValidation::builder()
            .min(1)
            .max(10)
            .with_value(input.to_string())
            .build()
    }
))]
#[derive(Clone, Debug)]
#[cfg_attr(feature = "fluent", derive(es_fluent::EsFluent))]
pub struct LenValidation<T: HasLen> {
    /// Minimum allowed length (inclusive)
    pub min: usize,
    /// Maximum allowed length (inclusive)
    pub max: usize,
    /// The collection being validated (stored for error context)
    #[koruma(value)]
    #[cfg_attr(feature = "fluent", fluent(value(|x: &T| x.len())))]
    pub actual: T,
}

impl<T: HasLen + Clone> Validate<T> for LenValidation<T> {
    fn validate(&self, value: &T) -> bool {
        let len = value.len();
        !(len < self.min || len > self.max)
    }
}

#[cfg(feature = "fmt")]
impl<T: HasLen + Clone> std::fmt::Display for LenValidation<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "length {} is not within bounds [{}, {}]",
            self.actual.len(),
            self.min,
            self.max
        )
    }
}
