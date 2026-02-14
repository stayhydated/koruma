#![doc = include_str!("../README.md")]

mod validators;
pub use validators::*;

#[doc(hidden)]
#[cfg(feature = "fluent")]
pub mod i18n;
