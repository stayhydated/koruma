//! String validation validators.
//!
//! This module contains validators for string-based validation scenarios.

/// Trait alias for types that can be treated as string references.
///
/// This is a stable Rust pattern for trait aliases, providing a convenient
/// bound for validators that work with string-like types.
pub trait StringLike: AsRef<str> {}

impl<T: AsRef<str>> StringLike for T {}

mod alphanumeric;
mod ascii;
#[cfg(feature = "heck")]
mod case;
mod contains;
mod matches;
#[cfg(feature = "regex")]
mod pattern;
mod prefix;
mod suffix;

pub use alphanumeric::AlphanumericValidation;
pub use ascii::AsciiValidation;
#[cfg(feature = "heck")]
pub use case::{Case, CaseValidation};
pub use contains::ContainsValidation;
pub use matches::MatchesValidation;
#[cfg(feature = "regex")]
pub use pattern::PatternValidation;
pub use prefix::PrefixValidation;
pub use suffix::SuffixValidation;
