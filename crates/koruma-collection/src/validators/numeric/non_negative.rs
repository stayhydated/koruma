//! Non-negative number validation for koruma.
//!
//! This module provides:
//! - `NonNegativeValidation` validator to check if a numeric value is >= 0
//!
//! # Example
//! ```ignore
//! use koruma::Koruma;
//! use koruma_collection::validators::non_negative::NonNegativeValidation;
//!
//! #[derive(Koruma)]
//! struct Account {
//!     #[koruma(NonNegativeValidation<_>)]
//!     balance: f64,
//! }
//! ```

use koruma::{KorumaResult, Validate, validator};

use super::Numeric;

/// Validates that a numeric value is non-negative (>= 0).
#[validator]
#[derive(Clone, Debug)]
#[cfg_attr(feature = "fluent", derive(es_fluent::EsFluent))]
pub struct NonNegativeValidation<T: Numeric> {
    /// The value being validated (stored for error context)
    #[koruma(value)]
    #[cfg_attr(feature = "fluent", fluent(value(|x: &T| x.to_string())))]
    pub actual: T,
}

impl<T: Numeric> Validate<T> for NonNegativeValidation<T> {
    fn validate(&self, value: &T) -> KorumaResult {
        if *value >= T::default() {
            Ok(())
        } else {
            Err(())
        }
    }
}

#[cfg(feature = "fmt")]
impl<T: Numeric> std::fmt::Display for NonNegativeValidation<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "value {} must be non-negative (>= 0)", self.actual)
    }
}
