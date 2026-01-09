//! Phone number validation for koruma.
//!
//! This module provides:
//! - `PhoneNumberValidation` validator to check if a string is a valid phone number
//!
//! # Example
//! ```rust
//! use koruma::Koruma;
//! use koruma_collection::format::PhoneNumberValidation;
//!
//! #[derive(Koruma)]
//! struct Contact {
//!     #[koruma(PhoneNumberValidation::<_>)]
//!     phone: String,
//! }
//! ```

use koruma::{Validate, validator};

/// Validates that a string is a valid phone number.
#[validator]
#[cfg_attr(feature = "showcase", showcase(
    name = "Phone Number",
    description = "Validates that the input is a valid phone number",
    create = |input: &str| {
        PhoneNumberValidation::builder()
            .with_value(input.to_string())
            .build()
    }
))]
#[derive(Clone, Debug)]
#[cfg_attr(feature = "fluent", derive(es_fluent::EsFluent))]
pub struct PhoneNumberValidation<T: AsRef<str>> {
    /// The string being validated (stored for error context)
    #[koruma(value)]
    #[cfg_attr(feature = "fluent", fluent(value(|x: &T| x.as_ref().to_string())))]
    pub actual: T,
}

impl<T: AsRef<str>> Validate<T> for PhoneNumberValidation<T> {
    fn validate(&self, value: &T) -> bool {
        use std::str::FromStr as _;

        let s = value.as_ref();
        match phonenumber::PhoneNumber::from_str(s) {
            Ok(number) => number.is_valid(),
            Err(_) => false,
        }
    }
}

#[cfg(feature = "fmt")]
impl<T: AsRef<str>> std::fmt::Display for PhoneNumberValidation<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "not a valid phone number")
    }
}
