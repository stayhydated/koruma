//! Validators module for koruma-collection
//!
//! This module contains all the validation implementations for common validation scenarios.

pub mod alphanumeric;
pub mod ascii;
pub mod contains;
#[cfg(feature = "credit-card")]
pub mod credit_card;
#[cfg(feature = "email")]
pub mod email;
pub mod ip;
pub mod len;
pub mod matches;
#[cfg(feature = "regex")]
pub mod pattern;
#[cfg(feature = "phone-number")]
pub mod phone_number;
pub mod prefix;
pub mod range;
pub mod required;
pub mod suffix;
#[cfg(feature = "url")]
pub mod url;
