//! Core expansion logic for koruma derive macros.
//!
//! This module contains the actual TokenStream generation that can be tested.

use quote::format_ident;
use syn::{Field, Fields, Member};
use syn_cfg_attr::AttributeHelpers;

pub(crate) mod codegen;
pub(crate) mod crate_path;
pub(crate) mod derive;
pub(crate) mod display;
#[cfg(feature = "fluent")]
pub(crate) mod fluent;
pub(crate) mod plan;
pub(crate) mod validator;

pub(crate) use crate_path::koruma_crate_path;
pub use derive::expand_koruma;
pub use display::expand_koruma_all_display;
#[cfg(feature = "fluent")]
pub use fluent::expand_koruma_all_fluent;
pub use validator::expand_validator;

// Re-exports for tests and internal usage
#[allow(unused_imports)]
pub(crate) use codegen::{
    effective_validation_type, reject_legacy_full_option_syntax, resolve_explicit_infer_type,
    transform_arg_value, validate_each_collection_type, validator_builder_expr,
    validator_field_ident, validator_infer_source_type, validator_variant_ident,
    validator_wants_full_type,
};

// Re-export parsing types from koruma-derive-core
#[cfg(feature = "internal-showcase")]
#[allow(unused_imports)]
pub(crate) use koruma_derive_core::find_showcase_attr;
#[allow(unused_imports)]
pub(crate) use koruma_derive_core::{
    FieldAttrAst, FieldInfo, NormalizedFieldSpec, ParseFieldResult, StructOptions, ValidatorAttr,
    find_value_field_strict, parse_field, parse_struct_options,
};
#[cfg(feature = "internal-showcase")]
#[allow(unused_imports)]
pub(crate) use koruma_derive_core::{ShowcaseAttr, ShowcaseInputType, ShowcaseModule};

// Re-export utility functions from koruma-derive-core
#[allow(unused_imports)]
pub(crate) use koruma_derive_core::{
    contains_infer_type, expr_as_simple_ident, first_generic_arg, is_option_type,
    option_inner_type, substitute_infer_type, substitute_infer_type_from_source, vec_inner_type,
};

pub(crate) fn collect_field_infos(
    fields: &Fields,
    struct_options: Option<&StructOptions>,
) -> Result<Vec<FieldInfo>, syn::Error> {
    let mut field_infos = Vec::new();

    for (i, field) in fields.iter().enumerate() {
        match parse_field(field, i) {
            ParseFieldResult::Valid(info) => field_infos.push(*info),
            ParseFieldResult::Skip => {},
            ParseFieldResult::Error(e) => return Err(e),
        }
    }

    if struct_options.is_some_and(|options| options.is_newtype())
        && fields.len() == 1
        && field_infos.is_empty()
    {
        let (index, field) = fields
            .iter()
            .enumerate()
            .next()
            .expect("single-field newtypes should expose one field");

        if has_explicit_koruma_skip(field)? {
            return Err(syn::Error::new_spanned(
                field,
                "struct-level newtypes require their only field to participate in validation; `#[koruma(skip)]` is not allowed",
            ));
        }

        field_infos.push(synthetic_struct_newtype_field_info(field, index));
    }

    Ok(field_infos)
}

fn has_explicit_koruma_skip(field: &Field) -> Result<bool, syn::Error> {
    for attr in field.attrs.to_vec().find_attribute("koruma") {
        let parsed: FieldAttrAst = attr.parse_args()?;
        if parsed.is_skip() {
            return Ok(true);
        }
    }

    Ok(false)
}

fn synthetic_struct_newtype_field_info(field: &Field, index: usize) -> FieldInfo {
    let (name, member) = match field.ident.clone() {
        Some(ident) => (ident.clone(), Member::Named(ident)),
        None => (
            format_ident!("_{}", index),
            Member::Unnamed(syn::Index::from(index)),
        ),
    };

    FieldInfo {
        name,
        member,
        ty: field.ty.clone(),
        validation: NormalizedFieldSpec {
            field_validators: Vec::new(),
            element_validators: Vec::new(),
            mode: koruma_derive_core::FieldMode::Newtype,
        },
    }
}
