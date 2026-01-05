//! Credit card validation for koruma.
//!
//! This module provides:
//! - `CreditCardValidation` validator to check if a string is a valid credit card number
//!
//! # Example
//! ```ignore
//! use koruma::Koruma;
//! use koruma_collection::validators::credit_card::CreditCardValidation;
//!
//! #[derive(Koruma)]
//! struct Payment {
//!     #[koruma(CreditCardValidation<_>)]
//!     card_number: String,
//! }
//! ```

use koruma::{Validate, validator};

/// Validates that a string is a valid credit card number.
#[validator]
#[derive(Clone, Debug)]
#[cfg_attr(feature = "fluent", derive(es_fluent::EsFluent))]
pub struct CreditCardValidation<T: AsRef<str>> {
    /// The string being validated (stored for error context)
    #[koruma(value)]
    #[cfg_attr(feature = "fluent", fluent(value(|x: &T| x.as_ref().to_string())))]
    pub actual: T,
}

impl<T: AsRef<str>> Validate<T> for CreditCardValidation<T> {
    fn validate(&self, value: &T) -> bool {
        let s = value.as_ref();
        match card_validate::Validate::from(s) {
            Ok(_) => true,
            Err(_) => false,
        }
    }
}

#[cfg(feature = "fmt")]
impl<T: AsRef<str>> std::fmt::Display for CreditCardValidation<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "not a valid credit card number")
    }
}
