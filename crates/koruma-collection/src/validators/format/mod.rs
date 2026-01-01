//! Format validation validators.
//!
//! This module contains validators for specific format validation scenarios
//! such as emails, URLs, phone numbers, credit cards, and IP addresses.

#[cfg(feature = "credit-card")]
pub mod credit_card;
#[cfg(feature = "email")]
pub mod email;
pub mod ip;
#[cfg(feature = "phone-number")]
pub mod phone_number;
#[cfg(feature = "url")]
pub mod url;
