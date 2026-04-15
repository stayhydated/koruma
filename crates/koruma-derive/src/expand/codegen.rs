use koruma_derive_core::{
    ValidatorAttr, contains_infer_type, expr_as_simple_ident, first_generic_arg,
    is_option_infer_type, option_inner_type, substitute_infer_type, vec_inner_type,
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
/// This unwraps an outer `Option<_>` first so `Option<Vec<T>>` correctly
/// behaves like an optional collection instead of an optional element.
pub(crate) fn each_collection_type(field_ty: &Type) -> &Type {
    option_inner_type(field_ty).unwrap_or(field_ty)
}

/// Returns the raw element type used by `each(...)`.
///
/// For `Vec<Option<T>>` this returns `Option<T>`.
/// For `Option<Vec<T>>` this returns `T`.
pub(crate) fn each_element_type(field_ty: &Type) -> &Type {
    let collection_ty = each_collection_type(field_ty);
    vec_inner_type(collection_ty).unwrap_or(collection_ty)
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
/// - `<Vec<_>>`: substitutes `_` with the inner type from the field
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
            // Substitute `_` with the inner type from the field or element.
            // e.g., Vec<_> on field Vec<String> → Vec<String>
            // e.g., HashSet<_> on field HashSet<i32> → HashSet<i32>
            let infer_source = if validate_each {
                each_element_type(field_ty)
            } else {
                field_ty
            };
            let inner_ty = first_generic_arg(infer_source).unwrap_or(infer_source);
            let substituted = substitute_infer_type(explicit_ty, inner_ty);
            return quote! { #validator<#substituted> };
        }
        return quote! { #validator<#explicit_ty> };
    }

    // For `each` validation, unwrap outer Option<Vec<T>> to Vec<T> first,
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
    // Unwrap outer Option<Vec<T>> first for each validation.
    let after_each = if validate_each {
        each_element_type(field_ty)
    } else {
        field_ty
    };

    // Unwrap Option<T> for optional field validation
    option_inner_type(after_each).unwrap_or(after_each)
}
