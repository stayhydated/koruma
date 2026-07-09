//! General validation validators.
//!
//! This module contains validators for general validation scenarios
//! that don't fit into other categories.

mod required;

pub use required::RequiredValidation;

#[cfg(feature = "internal-showcase")]
#[doc(hidden)]
#[inline(never)]
pub fn __link_showcase_validators() {}
