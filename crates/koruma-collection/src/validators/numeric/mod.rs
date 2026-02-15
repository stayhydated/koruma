//! Numeric validation validators.
//!
//! This module contains validators for numeric validation scenarios.

use std::fmt::Display;

/// Trait for numeric types that can be validated for positivity, negativity, and ranges.
///
/// This trait is automatically implemented for all types that satisfy the bounds:
/// `PartialOrd + Default + Copy + Display`.
///
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

#[cfg(feature = "internal-showcase")]
#[doc(hidden)]
#[inline(never)]
pub fn __link_showcase_validators() {
    negative::__koruma_showcase_anchor_negative_validation();
    non_negative::__koruma_showcase_anchor_non_negative_validation();
    non_positive::__koruma_showcase_anchor_non_positive_validation();
    positive::__koruma_showcase_anchor_positive_validation();
    range::__koruma_showcase_anchor_range_validation();
}
