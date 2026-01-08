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
