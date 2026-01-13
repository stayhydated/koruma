//! Core expansion logic for koruma derive macros.
//!
//! This module contains the actual TokenStream generation that can be tested.

pub(crate) mod codegen;
pub(crate) mod derive;
pub(crate) mod display;
#[cfg(feature = "fluent")]
pub(crate) mod fluent;
pub(crate) mod validator;

pub use derive::expand_koruma;
pub use display::expand_koruma_all_display;
#[cfg(feature = "fluent")]
pub use fluent::expand_koruma_all_fluent;
pub use validator::expand_validator;

// Re-exports for tests and internal usage
#[allow(unused_imports)]
pub(crate) use codegen::{
    effective_validation_type, transform_arg_value, validator_type_for_field,
    validator_wants_full_type,
};

// Re-export parsing types from koruma-derive-core
#[cfg(feature = "showcase")]
#[allow(unused_imports)]
pub(crate) use koruma_derive_core::ShowcaseAttr;
#[cfg(feature = "showcase")]
#[allow(unused_imports)]
pub(crate) use koruma_derive_core::find_showcase_attr;
#[allow(unused_imports)]
pub(crate) use koruma_derive_core::{
    FieldInfo, KorumaAttr, ParseFieldResult, StructOptions, ValidatorAttr, find_value_field,
    parse_field, parse_struct_options,
};

// Re-export utility functions from koruma-derive-core
#[allow(unused_imports)]
pub(crate) use koruma_derive_core::{
    contains_infer_type, expr_as_simple_ident, first_generic_arg, is_option_infer_type,
    is_option_type, option_inner_type, substitute_infer_type, vec_inner_type,
};
