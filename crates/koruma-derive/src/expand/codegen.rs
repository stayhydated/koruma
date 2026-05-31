use koruma_derive_core::{expr_as_simple_ident, is_option_type, option_inner_type, vec_inner_type};
use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, quote};
use std::collections::BTreeSet;
use syn::visit::{self, Visit};
use syn::{Expr, ExprPath, GenericParam, Generics, Ident, Lifetime, Type, TypePath, parse_quote};

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

    pub fn type_path_type(&self, ident: &Ident) -> Type {
        let ty_generics = &self.ty_generics;
        parse_quote! { #ident #ty_generics }
    }
}

pub(crate) struct RefEnumGenerics {
    pub definition: Generics,
    pub impl_generics: TokenStream2,
    pub ty_generics: TokenStream2,
    pub where_clause: TokenStream2,
    pub return_ty_generics: TokenStream2,
}

impl RefEnumGenerics {
    pub fn return_type_path(&self, ident: &Ident) -> TokenStream2 {
        let ty_generics = &self.return_ty_generics;
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
    usages: &[Type],
) -> HelperGenerics {
    let (type_params, const_params, lifetime_params) = collect_generic_params(source_generics);
    let mut used: BTreeSet<String> = usages
        .iter()
        .flat_map(|ty| {
            collect_matching_generic_names_from_type(
                ty,
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

fn generic_param_type_arg(param: &GenericParam) -> TokenStream2 {
    match param {
        GenericParam::Lifetime(param) => {
            let lifetime = &param.lifetime;
            quote! { #lifetime }
        },
        GenericParam::Type(param) => {
            let ident = &param.ident;
            quote! { #ident }
        },
        GenericParam::Const(param) => {
            let ident = &param.ident;
            quote! { #ident }
        },
    }
}

pub(crate) fn ref_enum_generics_for_usages(
    source_generics: &Generics,
    usages: &[Type],
) -> RefEnumGenerics {
    let helper = helper_generics_for_usages(source_generics, usages);
    let mut definition = helper.definition.clone();
    definition.params.insert(0, parse_quote!('koruma));

    let return_args: Vec<TokenStream2> = helper
        .definition
        .params
        .iter()
        .map(generic_param_type_arg)
        .collect();
    let return_ty_generics = if return_args.is_empty() {
        quote! { <'_> }
    } else {
        quote! { <'_, #(#return_args),*> }
    };

    let definition_for_impl = definition.clone();
    let (impl_generics, ty_generics, where_clause) = definition_for_impl.split_for_impl();
    let where_clause = where_clause
        .map(|clause| quote! { #clause })
        .unwrap_or_default();

    RefEnumGenerics {
        definition,
        impl_generics: quote! { #impl_generics },
        ty_generics: quote! { #ty_generics },
        where_clause,
        return_ty_generics,
    }
}

#[cfg(test)]
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ValidationSite {
    Field,
    Element,
}

#[cfg(test)]
impl ValidationSite {
    pub(crate) fn is_element(self) -> bool {
        self == Self::Element
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FieldCardinality {
    Required,
    Optional,
}

impl FieldCardinality {
    pub(crate) fn for_type(ty: &Type) -> Self {
        if is_option_type(ty) {
            Self::Optional
        } else {
            Self::Required
        }
    }

    pub(crate) fn is_optional(self) -> bool {
        self == Self::Optional
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum EachIterationKind {
    VecLike,
    Slice,
    Array,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct EachCollection<'a> {
    pub collection_ty: &'a Type,
    pub element_ty: &'a Type,
    pub outer_cardinality: FieldCardinality,
    pub element_cardinality: FieldCardinality,
    pub iteration: EachIterationKind,
}

pub(crate) fn classify_each_collection(field_ty: &Type) -> Result<EachCollection<'_>, syn::Error> {
    let outer_cardinality = FieldCardinality::for_type(field_ty);
    let collection_ty = option_inner_type(field_ty).unwrap_or(field_ty);
    let Some((element_ty, iteration)) = classify_each_collection_inner(collection_ty) else {
        return Err(unsupported_each_collection_error(field_ty, collection_ty));
    };

    Ok(EachCollection {
        collection_ty,
        element_ty,
        outer_cardinality,
        element_cardinality: FieldCardinality::for_type(element_ty),
        iteration,
    })
}

fn classify_each_collection_inner(ty: &Type) -> Option<(&Type, EachIterationKind)> {
    match ty {
        Type::Array(array) => Some((&array.elem, EachIterationKind::Array)),
        Type::Group(group) => classify_each_collection_inner(&group.elem),
        Type::Paren(paren) => classify_each_collection_inner(&paren.elem),
        Type::Reference(reference) => classify_each_collection_inner(&reference.elem),
        Type::Slice(slice) => Some((&slice.elem, EachIterationKind::Slice)),
        _ => vec_inner_type(ty).map(|element_ty| (element_ty, EachIterationKind::VecLike)),
    }
}

fn unsupported_each_collection_error(field_ty: &Type, collection_ty: &Type) -> syn::Error {
    let rendered = quote! { #collection_ty }.to_string();
    syn::Error::new_spanned(
        field_ty,
        format!(
            "`each(...)` currently only supports `Vec<T>`, slice fields like `&[T]`, arrays like `[T; N]`, and optional variants of those, found `{rendered}`"
        ),
    )
}

/// Transform a validator arg value for use in generated code.
///
/// Bare identifiers that match struct fields are rejected so cross-field
/// validator arguments stay explicit at the call site.
pub(crate) fn validate_validator_arg_value(
    arg_value: &Expr,
    field_names: &[Ident],
) -> Result<(), syn::Error> {
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
        Ok(())
    }
}

/// Get the effective type for validation (unwrapping Option and Vec as needed)
#[cfg(test)]
pub(crate) fn effective_validation_type(field_ty: &Type, site: ValidationSite) -> &Type {
    // Unwrap outer Option<Collection<T>> first for each validation.
    let after_each = if site.is_element() {
        classify_each_collection(field_ty)
            .expect("test helper requires supported each(...) collection")
            .element_ty
    } else {
        field_ty
    };

    // Unwrap Option<T> for optional field validation
    option_inner_type(after_each).unwrap_or(after_each)
}
