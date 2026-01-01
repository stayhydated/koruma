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

pub mod negative;
pub mod non_negative;
pub mod non_positive;
pub mod positive;
pub mod range;
