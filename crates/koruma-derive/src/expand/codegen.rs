use koruma_derive_core::{
    ValidatorAttr, contains_infer_type, expr_as_simple_ident, is_option_infer_type,
    option_inner_type, substitute_infer_type_from_source, vec_inner_type,
};
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Expr, Ident, Type};

/// Check if a validator wants the full field type (not unwrapped from Option).
/// This is true for `<Option<_>>` syntax.
pub(crate) fn validator_wants_full_type(v: &ValidatorAttr) -> bool {
    v.explicit_type.as_ref().is_some_and(is_option_infer_type)
}

/// Returns the collection type that `each(...)` should iterate over.
///
/// This unwraps an outer `Option<_>` first so optional collection fields
/// correctly behave like optional collections instead of optional elements.
pub(crate) fn each_collection_type(field_ty: &Type) -> &Type {
    option_inner_type(field_ty).unwrap_or(field_ty)
}

fn each_supported_element_type(ty: &Type) -> Option<&Type> {
    match ty {
        Type::Array(array) => Some(&array.elem),
        Type::Group(group) => each_supported_element_type(&group.elem),
        Type::Paren(paren) => each_supported_element_type(&paren.elem),
        Type::Reference(reference) => each_supported_element_type(&reference.elem),
        Type::Slice(slice) => Some(&slice.elem),
        _ => vec_inner_type(ty),
    }
}

pub(crate) fn validate_each_collection_type(field_ty: &Type) -> Result<(), syn::Error> {
    let collection_ty = each_collection_type(field_ty);
    if each_supported_element_type(collection_ty).is_some() {
        return Ok(());
    }

    let rendered = quote! { #collection_ty }.to_string();
    Err(syn::Error::new_spanned(
        field_ty,
        format!(
            "`each(...)` currently only supports `Vec<T>`, slice fields like `&[T]`, arrays like `[T; N]`, and optional variants of those, found `{rendered}`"
        ),
    ))
}

/// Returns the raw element type used by `each(...)`.
///
/// For `Vec<Option<T>>` this returns `Option<T>`.
/// For `Option<&[T]>` this returns `T`.
///
/// This helper assumes `validate_each_collection_type()` already accepted the field.
pub(crate) fn each_element_type(field_ty: &Type) -> &Type {
    let collection_ty = each_collection_type(field_ty);
    each_supported_element_type(collection_ty)
        .expect("each(...) should be pre-validated to only run on supported collection fields")
}

pub(crate) fn validator_infer_source_type<'a>(
    v: &ValidatorAttr,
    field_ty: &'a Type,
    validate_each: bool,
) -> &'a Type {
    let raw_source = if validate_each {
        each_element_type(field_ty)
    } else {
        field_ty
    };

    if validator_wants_full_type(v) {
        raw_source
    } else {
        option_inner_type(raw_source).unwrap_or(raw_source)
    }
}

pub(crate) fn resolve_explicit_infer_type(
    v: &ValidatorAttr,
    field_ty: &Type,
    validate_each: bool,
) -> Result<Option<Type>, syn::Error> {
    let Some(explicit_ty) = v.explicit_type.as_ref() else {
        return Ok(None);
    };

    if !contains_infer_type(explicit_ty) {
        return Ok(None);
    }

    let infer_source = validator_infer_source_type(v, field_ty, validate_each);
    substitute_infer_type_from_source(explicit_ty, infer_source)
        .map(Some)
        .ok_or_else(|| {
            let rendered_explicit = quote! { #explicit_ty }.to_string();
            let rendered_source = quote! { #infer_source }.to_string();
            syn::Error::new_spanned(
                explicit_ty,
                format!(
                    "cannot infer `_` in `{rendered_explicit}` from `{rendered_source}`; use concrete type arguments or a matching generic shape"
                ),
            )
        })
}

/// Transform a validator arg value for use in generated code.
/// If the expression is a simple identifier that matches a struct field name,
/// transform it to `self.field.clone()`. Otherwise, use the expression as-is.
pub(crate) fn transform_arg_value(arg_value: &Expr, field_names: &[Ident]) -> TokenStream2 {
    if let Some(field_ident) = expr_as_simple_ident(arg_value)
        && field_names.iter().any(|name| name == field_ident)
    {
        quote! { self.#field_ident.clone() }
    } else {
        quote! { #arg_value }
    }
}

/// Helper to generate the type for a validator
///
/// Type inference behavior:
/// - `<_>`: uses the full field type (unwrapping Option if present)
/// - Explicit types containing `_`: infer from the field/element type using generic shape matching
/// - `<SomeType>`: uses the explicit type directly
/// - For `each` validation on `Vec<T>`: uses T
/// - For optional fields `Option<T>`: uses T (validation is skipped if None)
pub(crate) fn validator_type_for_field(
    v: &ValidatorAttr,
    field_ty: &Type,
    validate_each: bool,
) -> TokenStream2 {
    let validator = &v.validator;

    // If explicit type is provided, check if it contains `_` for substitution
    if let Some(ref explicit_ty) = v.explicit_type {
        if contains_infer_type(explicit_ty) {
            let substituted = resolve_explicit_infer_type(v, field_ty, validate_each)
                .expect("explicit infer types should be pre-validated")
                .expect("explicit infer types should resolve to a concrete type");
            return quote! { #validator<#substituted> };
        }
        return quote! { #validator<#explicit_ty> };
    }

    // For `each` validation, unwrap outer Option<Collection<T>> first,
    // then unwrap the collection element type.
    let after_each = if validate_each {
        each_element_type(field_ty)
    } else {
        field_ty
    };

    // Unwrap Option<T> for optional field validation
    let effective_ty = option_inner_type(after_each).unwrap_or(after_each);

    if v.infer_type {
        // <_> means use the field type (after unwrapping Option)
        quote! { #validator<#effective_ty> }
    } else {
        quote! { #validator }
    }
}

/// Get the effective type for validation (unwrapping Option and Vec as needed)
pub(crate) fn effective_validation_type(field_ty: &Type, validate_each: bool) -> &Type {
    // Unwrap outer Option<Collection<T>> first for each validation.
    let after_each = if validate_each {
        each_element_type(field_ty)
    } else {
        field_ty
    };

    // Unwrap Option<T> for optional field validation
    option_inner_type(after_each).unwrap_or(after_each)
}
