//! Core expansion logic for koruma derive macros.
//!
//! This module contains the actual TokenStream generation that can be tested.

use quote::format_ident;
use syn::{Field, Fields, Member};
use syn_cfg_attr::AttributeHelpers;

pub(crate) mod codegen;
pub(crate) mod crate_path;
pub(crate) mod derive;
pub(crate) mod derive_constructors;
pub(crate) mod derive_field_errors;
pub(crate) mod derive_main_error;
pub(crate) mod derive_newtype;
pub(crate) mod derive_shared;
pub(crate) mod derive_validation;
pub(crate) mod display;
pub(crate) mod error_bag;
#[cfg(feature = "fluent")]
pub(crate) mod fluent;
pub(crate) mod names;
pub(crate) mod plan;
pub(crate) mod validator;

pub(crate) use crate_path::koruma_crate_path;
pub use derive::expand_koruma;
pub use display::expand_koruma_all_display;
#[cfg(feature = "fluent")]
pub use fluent::expand_koruma_all_fluent;
use names::tuple_field_ident;
pub use validator::expand_validator;

use self::error_bag::ErrorBag;

// Re-exports for tests and internal usage
#[allow(unused_imports)]
pub(crate) use codegen::classify_each_collection;
#[cfg(test)]
pub(crate) use codegen::effective_validation_type;
#[allow(unused_imports)]
pub(crate) use names::validator_names;

// Re-export parsing types from koruma-derive-core
#[cfg(feature = "internal-showcase")]
#[allow(unused_imports)]
pub(crate) use koruma_derive_core::find_showcase_attr;
#[allow(unused_imports)]
pub(crate) use koruma_derive_core::{
    DataFieldKorumaAttr, FieldInfo, ParsedFieldSpec, StructOptions, ValidatorAttr, parse_field,
    parse_struct_options,
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
    let mut errors = ErrorBag::new();

    for (i, field) in fields.iter().enumerate() {
        if let Some(info) = errors.push_result(parse_field(field, i)).flatten() {
            field_infos.push(info);
        }
    }

    errors.finish()?;

    if struct_options.is_some_and(|options| options.is_newtype())
        && fields.len() == 1
        && field_infos.is_empty()
    {
        let Some((index, field)) = fields.iter().enumerate().next() else {
            return Ok(field_infos);
        };

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
        let parsed: DataFieldKorumaAttr = attr.parse_args()?;
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
            tuple_field_ident(index),
            Member::Unnamed(syn::Index::from(index)),
        ),
    };

    FieldInfo {
        name,
        member,
        ty: field.ty.clone(),
        index,
        validation: ParsedFieldSpec::Newtype {
            marker: format_ident!("newtype"),
            field_validators: Vec::new(),
        },
    }
}
