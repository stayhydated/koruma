//! Email validation for koruma.
//!
//! This module provides:
//! - `EmailValidation` validator to check if a string is a valid email address
//!
//! # Example
//! ```ignore
//! use koruma::Koruma;
//! use koruma_collection::validators::email::EmailValidation;
//!
//! #[derive(Koruma)]
//! struct User {
//!     #[koruma(EmailValidation<_>)]
//!     email: String,
//! }
//! ```

use koruma::{Validate, validator};

/// Validates that a string is a valid email address.
#[validator]
#[derive(Clone, Debug)]
#[cfg_attr(feature = "fluent", derive(es_fluent::EsFluent))]
pub struct EmailValidation<T: AsRef<str>> {
    /// The string being validated (stored for error context)
    #[koruma(value)]
    #[cfg_attr(feature = "fluent", fluent(value(|x: &T| x.as_ref().to_string())))]
    pub actual: T,
}

impl<T: AsRef<str>> Validate<T> for EmailValidation<T> {
    fn validate(&self, value: &T) -> bool {
        let s = value.as_ref();
        // Basic email validation - check for @ symbol and proper format
        if s.is_empty() {
            return false;
        }

        let parts: Vec<&str> = s.split('@').collect();
        if parts.len() != 2 {
            return false;
        }

        let (user, domain) = (parts[0], parts[1]);

        if user.is_empty() || domain.is_empty() {
            return false;
        }

        // Check user length
        if user.len() > 64 {
            return false;
        }

        // Check domain length
        if domain.len() > 255 {
            return false;
        }

        // Validate user part - alphanumeric and some special characters
        let user_regex = regex::Regex::new(r"^[a-zA-Z0-9.!#$%&'*+/=?^_`{|}~-]+\z").unwrap();
        if !user_regex.is_match(user) {
            return false;
        }

        // Validate domain part
        let domain_regex = regex::Regex::new(r"^[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$").unwrap();
        if !domain_regex.is_match(domain) {
            // Check if it's an IP address in brackets
            if !domain.starts_with('[') || !domain.ends_with(']') {
                return false;
            }

            let ip_part = &domain[1..domain.len() - 1];
            if ip_part.parse::<std::net::IpAddr>().is_err() {
                return false;
            }
        }

        true
    }
}

#[cfg(feature = "fmt")]
impl<T: AsRef<str>> std::fmt::Display for EmailValidation<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "not a valid email address")
    }
}
