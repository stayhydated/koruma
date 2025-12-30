//! Integration tests for the koruma validation library.
//!
//! These tests exercise the full validation system including derive macros.

#[path = "integration/validators.rs"]
mod validators;

#[path = "integration/fixtures.rs"]
mod fixtures;

#[path = "integration/tests.rs"]
#[cfg(test)]
mod tests;
