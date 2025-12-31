//! Phone number validation for koruma.
//!
//! This module provides:
//! - `PhoneNumberValidation` validator to check if a string is a valid phone number
//!
//! # Example
//! ```ignore
//! use koruma::Koruma;
//! use koruma_collection::validators::phone_number::PhoneNumberValidation;
//!
//! #[derive(Koruma)]
//! struct Contact {
//!     #[koruma(PhoneNumberValidation<_>)]
//!     phone: String,
//! }
//! ```

use koruma::{KorumaResult, Validate, validator};

/// Validates that a string is a valid phone number.
#[validator]
#[derive(Clone, Debug)]
#[cfg_attr(feature = "fluent", derive(es_fluent::EsFluent))]
pub struct PhoneNumberValidation<T: AsRef<str>> {
    /// The string being validated (stored for error context)
    #[koruma(value)]
    #[cfg_attr(feature = "fluent", fluent(value(|x: &T| x.as_ref().to_string())))]
    pub actual: T,
}

impl<T: AsRef<str>> Validate<T> for PhoneNumberValidation<T> {
    fn validate(&self, value: &T) -> KorumaResult {
        use std::str::FromStr as _;

        let s = value.as_ref();
        match phonenumber::PhoneNumber::from_str(s) {
            Ok(number) => {
                if number.is_valid() {
                    Ok(())
                } else {
                    Err(())
                }
            },
            Err(_) => Err(()),
        }
    }
}

#[cfg(feature = "fmt")]
impl<T: AsRef<str>> std::fmt::Display for PhoneNumberValidation<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "not a valid phone number")
    }
}
