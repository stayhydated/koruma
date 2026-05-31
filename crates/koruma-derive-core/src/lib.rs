#![doc = include_str!("../README.md")]

//! Core parsing types and utilities for koruma derive macros.
//!
//! This crate provides a public API for parsing `#[koruma(...)]` attributes,
//! allowing consumers to analyze koruma validation metadata without depending
//! on the proc-macro crate directly.
//!
//! # Example
//!
//! ```rust
//! use koruma_derive_core::parse_field;
//! use syn::{Field, parse_quote};
//!
//! fn validator_count(field: &Field) -> syn::Result<Option<usize>> {
//!     Ok(parse_field(field, 0)?
//!         .map(|info| info.field_validators().len()))
//! }
//!
//! let field: Field = parse_quote! {
//!     #[koruma(NonEmptyValidation)]
//!     username: String
//! };
//! assert_eq!(validator_count(&field).unwrap(), Some(1));
//! ```

mod parse;
mod utils;

#[cfg(test)]
mod tests;

// Re-export parsing types
pub use parse::{
    BuilderMethodCall, CapturePolicy, DataFieldKorumaAttr, DataFieldKorumaItem,
    ElementValidationSpec, FieldInfo, FieldModifier, FieldModifierKind, FieldValidationSpec,
    KorumaAttrContext, ParsedFieldSpec, ParsedValidatorUse, SetterDefault, StructConstructor,
    StructKorumaAttr, StructKorumaItem, StructNewtypeOptions, StructOptions, ValidatorAttr,
    ValidatorFieldKorumaItem, ValidatorFieldRole, ValidatorFieldSpec, ValidatorSetterArg,
    ValidatorSetterSpec, ValidatorStructSpec, ValidatorTypeArg, ValidatorValueSpec, ValueFieldInfo,
    find_value_field_info_strict, find_value_field_strict, parse_field, parse_struct_options,
    parse_validator_fields_strict,
};

#[cfg(feature = "internal-showcase")]
pub use parse::{ShowcaseAttr, ShowcaseInputType, ShowcaseModule, find_showcase_attr};

// Re-export utility functions
pub use utils::{
    TypeShape, contains_infer_type, expr_as_simple_ident, first_generic_arg, is_option_type,
    option_inner_type, substitute_infer_type, substitute_infer_type_from_source, type_to_ident,
    vec_inner_type,
};
