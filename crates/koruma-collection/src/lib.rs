//! Common validators for the koruma validation library.
//!
//! This crate provides ready-to-use validators for common validation scenarios.
//!
//! ## Features
//!
//! - `fmt` (default) - Enables `Display` implementations for error messages
//! - `fluent` - Enables fluent-based i18n for error messages
//! - `tui` - Enables the interactive TUI for testing validators

mod validators;
pub use validators::*;

#[cfg(feature = "fluent")]
pub mod i18n;

#[cfg(feature = "tui")]
pub mod tui;
