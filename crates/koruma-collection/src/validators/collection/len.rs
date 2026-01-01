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

use koruma::{KorumaResult, Validate, validator};

use super::HasLen;

/// Validates that a collection's length is within the specified bounds.
///
/// Works with any type that implements `HasLen + Clone`.
#[validator]
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
    fn validate(&self, value: &T) -> KorumaResult {
        let len = value.len();
        if len < self.min || len > self.max {
            Err(())
        } else {
            Ok(())
        }
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
