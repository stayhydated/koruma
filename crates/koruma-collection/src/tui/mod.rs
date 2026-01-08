//! TUI module for interactive validator testing.
//!
//! This module provides a terminal user interface for testing validators interactively.
//! Enable with the `tui` feature flag.

mod app;

pub use app::run;

// Re-export showcase types for convenience
pub use koruma::showcase::{DynValidator, ValidatorShowcase};
