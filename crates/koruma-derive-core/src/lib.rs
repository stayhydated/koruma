//! Core parsing types and utilities for koruma derive macros.
//!
//! This crate provides a public API for parsing `#[koruma(...)]` attributes,
//! allowing consumers to analyze koruma validation metadata without depending
//! on the proc-macro crate directly.
//!
//! # Example
//!
//! ```ignore
//! use koruma_derive_core::{parse_field, ParseFieldResult, FieldInfo};
//! use syn::Field;
//!
//! fn analyze_field(field: &Field) {
//!     match parse_field(field) {
//!         ParseFieldResult::Valid(info) => {
//!             println!("Field {} has {} validators", info.name, info.field_validators.len());
//!             for v in &info.field_validators {
//!                 println!("  - {}", v.name());
//!             }
//!         }
//!         ParseFieldResult::Skip => println!("Field skipped"),
//!         ParseFieldResult::Error(e) => println!("Parse error: {}", e),
//!     }
//! }
//! ```

mod parse;
mod utils;

#[cfg(test)]
mod tests;

// Re-export parsing types
pub use parse::{
    FieldInfo, KorumaAttr, ParseFieldResult, StructOptions, ValidationInfo, ValidatorAttr,
    find_value_field, parse_field, parse_struct_options,
};

#[cfg(feature = "showcase")]
pub use parse::{ShowcaseAttr, find_showcase_attr};

// Re-export utility functions
pub use utils::{
    contains_infer_type, expr_as_simple_ident, first_generic_arg, is_option_infer_type,
    is_option_type, option_inner_type, substitute_infer_type, vec_inner_type,
};
