use heck::{ToSnakeCase, ToUpperCamelCase};
use koruma_derive_core::{
    ValidatorAttr, contains_infer_type, expr_as_simple_ident, is_option_type, option_inner_type,
    substitute_infer_type_from_source, vec_inner_type,
};
use proc_macro2::{TokenStream as TokenStream2, TokenTree};
use quote::{ToTokens, format_ident, quote};
use std::collections::BTreeSet;
use syn::{Error, Expr, GenericParam, Generics, Ident, Type};

pub(crate) struct HelperGenerics {
    pub definition: Generics,
    pub impl_generics: TokenStream2,
    pub ty_generics: TokenStream2,
    pub where_clause: TokenStream2,
}

impl HelperGenerics {
    pub fn type_path(&self, ident: &Ident) -> TokenStream2 {
        let ty_generics = &self.ty_generics;
        quote! { #ident #ty_generics }
    }
}

fn generic_param_key(param: &GenericParam) -> String {
    match param {
        GenericParam::Lifetime(param) => param.lifetime.to_token_stream().to_string(),
        GenericParam::Type(param) => param.ident.to_string(),
        GenericParam::Const(param) => param.ident.to_string(),
    }
}

fn collect_matching_generic_names(
    tokens: &TokenStream2,
    param_names: &BTreeSet<String>,
) -> BTreeSet<String> {
    fn walk(tokens: TokenStream2, param_names: &BTreeSet<String>, used: &mut BTreeSet<String>) {
        let mut iter = tokens.into_iter().peekable();
        while let Some(token) = iter.next() {
            match token {
                TokenTree::Ident(ident) => {
                    let key = ident.to_string();
                    if param_names.contains(&key) {
                        used.insert(key);
                    }
                },
                TokenTree::Punct(punct) if punct.as_char() == '\'' => {
                    if let Some(TokenTree::Ident(ident)) = iter.peek() {
                        let key = format!("'{}", ident);
                        if param_names.contains(&key) {
                            used.insert(key);
                        }
                    }
                },
                TokenTree::Group(group) => walk(group.stream(), param_names, used),
                _ => {},
            }
        }
    }

    let mut used = BTreeSet::new();
    walk(tokens.clone(), param_names, &mut used);
    used
}

pub(crate) fn helper_generics_for_usages(
    source_generics: &Generics,
    usages: &[TokenStream2],
) -> HelperGenerics {
    let param_names: BTreeSet<String> = source_generics
        .params
        .iter()
        .map(generic_param_key)
        .collect();
    let mut used: BTreeSet<String> = usages
        .iter()
        .flat_map(|usage| collect_matching_generic_names(usage, &param_names))
        .collect();

    if let Some(where_clause) = &source_generics.where_clause {
        loop {
            let mut changed = false;
            for predicate in &where_clause.predicates {
                let predicate_tokens = quote! { #predicate };
                let predicate_names =
                    collect_matching_generic_names(&predicate_tokens, &param_names);
                if !predicate_names.is_empty() && !predicate_names.is_disjoint(&used) {
                    for name in predicate_names {
                        changed |= used.insert(name);
                    }
                }
            }
            if !changed {
                break;
            }
        }
    }

    let params = source_generics
        .params
        .iter()
        .filter(|param| used.contains(&generic_param_key(param)))
        .cloned()
        .collect();

    let mut definition_where_clause = None;
    if let Some(where_clause) = &source_generics.where_clause {
        let predicates: syn::punctuated::Punctuated<_, syn::token::Comma> = where_clause
            .predicates
            .iter()
            .filter(|predicate| {
                let predicate_tokens = quote! { #predicate };
                !collect_matching_generic_names(&predicate_tokens, &param_names).is_disjoint(&used)
            })
            .cloned()
            .collect();

        if !predicates.is_empty() {
            definition_where_clause = Some(syn::WhereClause {
                where_token: where_clause.where_token,
                predicates,
            });
        }
    }

    let definition = Generics {
        params,
        where_clause: definition_where_clause,
        ..Generics::default()
    };

    let definition_for_impl = definition.clone();
    let (impl_generics, ty_generics, where_clause) = definition_for_impl.split_for_impl();
    let where_clause = where_clause
        .map(|clause| quote! { #clause })
        .unwrap_or_default();

    HelperGenerics {
        definition,
        impl_generics: quote! { #impl_generics },
        ty_generics: quote! { #ty_generics },
        where_clause,
    }
}

/// Check if a validator wants the full field type (not unwrapped from Option).
///
/// Any explicit `Option<...>` validator type takes the full-type path so derived
/// validation passes `&Option<T>` instead of unwrapping to `&T`.
pub(crate) fn validator_wants_full_type(v: &ValidatorAttr) -> bool {
    v.explicit_type.as_ref().is_some_and(is_option_type)
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

pub(crate) fn validate_full_type_option_target(
    v: &ValidatorAttr,
    field_ty: &Type,
    validate_each: bool,
    field_name: &Ident,
) -> Result<(), Error> {
    if !validator_wants_full_type(v) {
        return Ok(());
    }

    let target_ty = validator_infer_source_type(v, field_ty, validate_each);
    if is_option_type(target_ty) {
        return Ok(());
    }

    let rendered_target = quote! { #target_ty }.to_string();
    let target_context = if validate_each {
        format!("element type of field `{field_name}`")
    } else {
        format!("field `{field_name}`")
    };

    Err(Error::new_spanned(
        v.explicit_type
            .as_ref()
            .expect("full-type validators should always have an explicit type"),
        format!(
            "explicit `Option<...>` validator types require an optional validation target, but the {target_context} is `{rendered_target}`"
        ),
    ))
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

pub(crate) fn validator_builder_expr(
    v: &ValidatorAttr,
    field_ty: &Type,
    validate_each: bool,
    field_names: &[Ident],
) -> TokenStream2 {
    let validator = &v.validator;
    let effective_ty = effective_validation_type(field_ty, validate_each);

    let uses_infer = v.infer_type || v.explicit_type.as_ref().is_some_and(contains_infer_type);
    let validator_path = if uses_infer {
        let validator_ty = if v.explicit_type.is_some() {
            let substituted = resolve_explicit_infer_type(v, field_ty, validate_each)
                .expect("explicit infer types should be pre-validated")
                .expect("explicit infer types should resolve to a concrete type");
            quote! { #substituted }
        } else {
            quote! { #effective_ty }
        };

        quote! { #validator::<#validator_ty> }
    } else {
        quote! { #validator }
    };

    let mut setter_calls = v.setter_calls().iter();
    let Some(first_method) = setter_calls.next() else {
        return quote! { #validator_path::__koruma_builder() };
    };

    let first_method_name = &first_method.method;
    let first_args: Vec<_> = first_method
        .args
        .iter()
        .map(|arg| transform_arg_value(arg, field_names))
        .collect();
    let rest_calls: Vec<TokenStream2> = setter_calls
        .map(|method| {
            let method_name = &method.method;
            let transformed_args: Vec<_> = method
                .args
                .iter()
                .map(|arg| transform_arg_value(arg, field_names))
                .collect();
            quote! { .#method_name(#(#transformed_args),*) }
        })
        .collect();

    quote! {
        #validator_path::#first_method_name(#(#first_args),*)
            #(#rest_calls)*
    }
}

fn stable_hash_hex(input: &str) -> String {
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in input.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{:08x}", (hash & 0xffff_ffff) as u32)
}

fn has_name_collision(
    target_name: &str,
    siblings: &[ValidatorAttr],
    name_fn: impl Fn(&ValidatorAttr) -> String,
) -> bool {
    siblings
        .iter()
        .filter(|sibling| name_fn(sibling) == target_name)
        .nth(1)
        .is_some()
}

pub(crate) fn validator_field_ident(v: &ValidatorAttr, siblings: &[ValidatorAttr]) -> Ident {
    let simple = v.name().to_string().to_snake_case();
    if !has_name_collision(&simple, siblings, |sibling| {
        sibling.name().to_string().to_snake_case()
    }) {
        return format_ident!("{}", simple);
    }

    let fallback = v.codegen_snake_name();
    let resolved =
        if has_name_collision(&fallback, siblings, |sibling| sibling.codegen_snake_name()) {
            format!("{}_{}", fallback, stable_hash_hex(&v.path_name()))
        } else {
            fallback
        };

    format_ident!("{}", resolved)
}

pub(crate) fn validator_variant_ident(v: &ValidatorAttr, siblings: &[ValidatorAttr]) -> Ident {
    let simple = v.name().to_string().to_upper_camel_case();
    if !has_name_collision(&simple, siblings, |sibling| {
        sibling.name().to_string().to_upper_camel_case()
    }) {
        return format_ident!("{}", simple);
    }

    let fallback = v.codegen_upper_camel_name();
    let resolved = if has_name_collision(&fallback, siblings, |sibling| {
        sibling.codegen_upper_camel_name()
    }) {
        format!(
            "{}H{}",
            fallback,
            stable_hash_hex(&v.path_name()).to_ascii_uppercase()
        )
    } else {
        fallback
    };

    format_ident!("{}", resolved)
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
