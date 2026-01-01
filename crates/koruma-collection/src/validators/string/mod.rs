//! String validation validators.
//!
//! This module contains validators for string-based validation scenarios.

/// Trait alias for types that can be treated as string references.
///
/// This is a stable Rust pattern for trait aliases, providing a convenient
/// bound for validators that work with string-like types.
pub trait StringLike: AsRef<str> {}

impl<T: AsRef<str>> StringLike for T {}

pub mod alphanumeric;
pub mod ascii;
pub mod contains;
pub mod matches;
#[cfg(feature = "regex")]
pub mod pattern;
pub mod prefix;
pub mod suffix;
