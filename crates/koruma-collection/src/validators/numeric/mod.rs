//! Numeric validation validators.
//!
//! This module contains validators for numeric validation scenarios.

use std::fmt::Display;

/// Trait for numeric types that can be validated for positivity, negativity, and ranges.
///
/// This trait is automatically implemented for all types that satisfy the bounds:
/// `PartialOrd + Default + Copy + Display`.
///
/// This is a stable Rust pattern for trait aliases, providing a convenient
/// bound for numeric validators.
pub trait Numeric: PartialOrd + Default + Copy + Display {}

impl<T: PartialOrd + Default + Copy + Display> Numeric for T {}

mod negative;
mod non_negative;
mod non_positive;
mod positive;
mod range;

pub use negative::NegativeValidation;
pub use non_negative::NonNegativeValidation;
pub use non_positive::NonPositiveValidation;
pub use positive::PositiveValidation;
pub use range::RangeValidation;
