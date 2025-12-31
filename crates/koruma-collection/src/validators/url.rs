//! URL validation for koruma.
//!
//! This module provides:
//! - `UrlValidation` validator to check if a string is a valid URL
//!
//! # Example
//! ```ignore
//! use koruma::Koruma;
//! use koruma_collection::validators::url::UrlValidation;
//!
//! #[derive(Koruma)]
//! struct Resource {
//!     #[koruma(UrlValidation<_>)]
//!     link: String,
//! }
//! ```

use koruma::{KorumaResult, Validate, validator};

/// Validates that a string is a valid URL.
#[validator]
#[derive(Clone, Debug)]
#[cfg_attr(feature = "fluent", derive(es_fluent::EsFluent))]
#[cfg(feature = "url")]
pub struct UrlValidation<T: AsRef<str>> {
    /// The string being validated (stored for error context)
    #[koruma(value)]
    #[cfg_attr(feature = "fluent", fluent(value(|x: &T| x.as_ref().to_string())))]
    pub actual: T,
}

#[cfg(feature = "url")]
impl<T: AsRef<str>> Validate<T> for UrlValidation<T> {
    fn validate(&self, value: &T) -> KorumaResult {
        let s = value.as_ref();
        match url::Url::parse(s) {
            Ok(_) => Ok(()),
            Err(_) => Err(()),
        }
    }
}

#[cfg(all(feature = "fmt", feature = "url"))]
impl<T: AsRef<str>> std::fmt::Display for UrlValidation<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "not a valid URL")
    }
}
