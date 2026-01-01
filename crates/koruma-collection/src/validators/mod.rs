//! Validators module for koruma-collection
//!
//! This module contains all the validation implementations for common validation scenarios,
//! organized into logical sections:
//!
//! - [`string`]: String-based validators (alphanumeric, ascii, contains, etc.)
//! - [`format`]: Format-specific validators (email, URL, phone number, etc.)
//! - [`numeric`]: Numeric validators (positive, negative, range, etc.)
//! - [`collection`]: Collection validators (length, non-empty)
//! - [`general`]: General-purpose validators (required)

pub mod collection;
pub mod format;
pub mod general;
pub mod numeric;
pub mod string;

pub use string::{
    StringLike, alphanumeric::AlphanumericValidation, ascii::AsciiValidation,
    contains::ContainsValidation, matches::MatchesValidation, prefix::PrefixValidation,
    suffix::SuffixValidation,
};

#[cfg(feature = "regex")]
pub use string::pattern::PatternValidation;

pub use format::ip::IpValidation;

#[cfg(feature = "credit-card")]
pub use format::credit_card::CreditCardValidation;

#[cfg(feature = "email")]
pub use format::email::EmailValidation;

#[cfg(feature = "phone-number")]
pub use format::phone_number::PhoneNumberValidation;

#[cfg(feature = "url")]
pub use format::url::UrlValidation;

pub use numeric::{
    Numeric, negative::NegativeValidation, non_negative::NonNegativeValidation,
    non_positive::NonPositiveValidation, positive::PositiveValidation, range::RangeValidation,
};

pub use collection::{HasLen, len::LenValidation, non_empty::NonEmptyValidation};

pub use general::required::RequiredValidation;
