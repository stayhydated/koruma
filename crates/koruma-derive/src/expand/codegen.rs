use heck::{ToSnakeCase, ToUpperCamelCase};
use koruma_derive_core::{
    ValidatorAttr, contains_infer_type, expr_as_simple_ident, is_option_type, option_inner_type,
    substitute_infer_type_from_source, vec_inner_type,
};
use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, format_ident, quote};
use std::collections::BTreeSet;
use syn::visit::{self, Visit};
use syn::{Error, Expr, ExprPath, GenericParam, Generics, Ident, Lifetime, Type, TypePath};

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

struct GenericUsageVisitor<'a> {
    type_params: &'a BTreeSet<String>,
    const_params: &'a BTreeSet<String>,
    lifetime_params: &'a BTreeSet<String>,
    used: BTreeSet<String>,
}

impl<'a> GenericUsageVisitor<'a> {
    fn new(
        type_params: &'a BTreeSet<String>,
        const_params: &'a BTreeSet<String>,
        lifetime_params: &'a BTreeSet<String>,
    ) -> Self {
        Self {
            type_params,
            const_params,
            lifetime_params,
            used: BTreeSet::new(),
        }
    }

    fn visit_path_arguments(&mut self, path: &syn::Path) {
        for segment in &path.segments {
            visit::visit_path_arguments(self, &segment.arguments);
        }
    }
}

impl<'ast> Visit<'ast> for GenericUsageVisitor<'_> {
    fn visit_type_path(&mut self, type_path: &'ast TypePath) {
        if let Some(qself) = &type_path.qself {
            self.visit_type(&qself.ty);
        } else if type_path.path.leading_colon.is_none()
            && type_path.path.segments.len() == 1
            && let Some(segment) = type_path.path.segments.first()
            && matches!(segment.arguments, syn::PathArguments::None)
            && self.type_params.contains(&segment.ident.to_string())
        {
            self.used.insert(segment.ident.to_string());
        }

        self.visit_path_arguments(&type_path.path);
    }

    fn visit_expr_path(&mut self, expr_path: &'ast ExprPath) {
        if expr_path.qself.is_none()
            && expr_path.path.leading_colon.is_none()
            && expr_path.path.segments.len() == 1
            && let Some(segment) = expr_path.path.segments.first()
            && matches!(segment.arguments, syn::PathArguments::None)
            && self.const_params.contains(&segment.ident.to_string())
        {
            self.used.insert(segment.ident.to_string());
        }

        self.visit_path_arguments(&expr_path.path);
    }

    fn visit_lifetime(&mut self, lifetime: &'ast Lifetime) {
        let key = lifetime.to_token_stream().to_string();
        if self.lifetime_params.contains(&key) {
            self.used.insert(key);
        }
    }
}

fn collect_generic_params(
    source_generics: &Generics,
) -> (BTreeSet<String>, BTreeSet<String>, BTreeSet<String>) {
    let mut type_params = BTreeSet::new();
    let mut const_params = BTreeSet::new();
    let mut lifetime_params = BTreeSet::new();

    for param in &source_generics.params {
        match param {
            GenericParam::Lifetime(param) => {
                lifetime_params.insert(param.lifetime.to_token_stream().to_string());
            },
            GenericParam::Type(param) => {
                type_params.insert(param.ident.to_string());
            },
            GenericParam::Const(param) => {
                const_params.insert(param.ident.to_string());
            },
        }
    }

    (type_params, const_params, lifetime_params)
}

fn collect_matching_generic_names_from_type(
    ty: &Type,
    type_params: &BTreeSet<String>,
    const_params: &BTreeSet<String>,
    lifetime_params: &BTreeSet<String>,
) -> BTreeSet<String> {
    let mut visitor = GenericUsageVisitor::new(type_params, const_params, lifetime_params);
    visitor.visit_type(ty);
    visitor.used
}

fn collect_matching_generic_names_from_predicate(
    predicate: &syn::WherePredicate,
    type_params: &BTreeSet<String>,
    const_params: &BTreeSet<String>,
    lifetime_params: &BTreeSet<String>,
) -> BTreeSet<String> {
    let mut visitor = GenericUsageVisitor::new(type_params, const_params, lifetime_params);
    visitor.visit_where_predicate(predicate);
    visitor.used
}

pub(crate) fn helper_generics_for_usages(
    source_generics: &Generics,
    usages: &[TokenStream2],
) -> HelperGenerics {
    let (type_params, const_params, lifetime_params) = collect_generic_params(source_generics);
    let mut used: BTreeSet<String> = usages
        .iter()
        .map(|usage| {
            syn::parse2::<Type>(usage.clone()).unwrap_or_else(|err| {
                panic!("helper generic usage should be a Rust type, got `{usage}`: {err}")
            })
        })
        .flat_map(|ty| {
            collect_matching_generic_names_from_type(
                &ty,
                &type_params,
                &const_params,
                &lifetime_params,
            )
        })
        .collect();

    if let Some(where_clause) = &source_generics.where_clause {
        loop {
            let mut changed = false;
            for predicate in &where_clause.predicates {
                let predicate_names = collect_matching_generic_names_from_predicate(
                    predicate,
                    &type_params,
                    &const_params,
                    &lifetime_params,
                );
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
                !collect_matching_generic_names_from_predicate(
                    predicate,
                    &type_params,
                    &const_params,
                    &lifetime_params,
                )
                .is_disjoint(&used)
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
    v.wants_full_target() || v.explicit_type().is_some_and(is_option_type)
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
    let Some(explicit_ty) = v.explicit_type() else {
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
        v.explicit_type()
            .expect("full-type validators should always have an explicit type"),
        format!(
            "explicit `Option<...>` validator types require an optional validation target, but the {target_context} is `{rendered_target}`"
        ),
    ))
}

/// Transform a validator arg value for use in generated code.
///
/// Bare identifiers that match struct fields are rejected so cross-field
/// validator arguments stay explicit at the call site.
pub(crate) fn transform_arg_value(
    arg_value: &Expr,
    field_names: &[Ident],
) -> Result<TokenStream2, syn::Error> {
    if let Some(field_ident) = expr_as_simple_ident(arg_value)
        && field_names.iter().any(|name| name == field_ident)
    {
        Err(syn::Error::new_spanned(
            arg_value,
            format!(
                "bare field argument `{field_ident}` is ambiguous; use `self.{field_ident}.clone()` explicitly"
            ),
        ))
    } else {
        Ok(quote! { #arg_value })
    }
}

pub(crate) fn validator_builder_expr(
    v: &ValidatorAttr,
    field_ty: &Type,
    validate_each: bool,
    field_names: &[Ident],
) -> Result<TokenStream2, syn::Error> {
    let validator = &v.validator;
    let effective_ty = validator_infer_source_type(v, field_ty, validate_each);

    let uses_infer = v.uses_type_inference() || v.explicit_type().is_some_and(contains_infer_type);
    let validator_path = if uses_infer {
        let validator_ty = if v.has_explicit_type() {
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
        return Ok(quote! { #validator_path::__koruma_builder() });
    };

    let first_method_name = &first_method.method;
    let first_args: Vec<_> = first_method
        .args
        .iter()
        .map(|arg| transform_arg_value(arg, field_names))
        .collect::<Result<_, _>>()?;
    let rest_calls: Vec<TokenStream2> = setter_calls
        .map(|method| {
            let method_name = &method.method;
            let transformed_args: Vec<_> = method
                .args
                .iter()
                .map(|arg| transform_arg_value(arg, field_names))
                .collect::<Result<_, _>>()?;
            Ok(quote! { .#method_name(#(#transformed_args),*) })
        })
        .collect::<Result<_, syn::Error>>()?;

    Ok(quote! {
        #validator_path::#first_method_name(#(#first_args),*)
            #(#rest_calls)*
    })
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
/// - `<_>`: uses the validation target type (unwrapping Option unless `full(...)` is used)
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
    if let Some(explicit_ty) = v.explicit_type() {
        if contains_infer_type(explicit_ty) {
            let substituted = resolve_explicit_infer_type(v, field_ty, validate_each)
                .expect("explicit infer types should be pre-validated")
                .expect("explicit infer types should resolve to a concrete type");
            return quote! { #validator<#substituted> };
        }
        return quote! { #validator<#explicit_ty> };
    }

    if v.uses_type_inference() {
        let effective_ty = validator_infer_source_type(v, field_ty, validate_each);
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
