use crate::expand::codegen::helper_generics_for_usages;
use crate::expand::derive_shared::{field_error_type, field_error_type_path};
use crate::expand::plan::{
    FieldPlan, PlannedErrorDefault, PlannedErrorGetter, PlannedErrorIsEmpty, PlannedMainErrorField,
    PlannedMainErrorStorage, ValidationPlan,
};
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Generics, Ident, Type};

pub(crate) struct MainErrorRender {
    pub definition: TokenStream2,
    pub impl_generics: TokenStream2,
    pub ty_generics: TokenStream2,
    pub where_clause: TokenStream2,
    pub path: TokenStream2,
    pub fields: Vec<TokenStream2>,
    pub getter_methods: Vec<TokenStream2>,
    pub is_empty_body: TokenStream2,
    pub defaults: Vec<TokenStream2>,
}

pub(crate) fn render_main_error(
    plan: &ValidationPlan,
    struct_name: &Ident,
    error_struct_name: &Ident,
    generics: &Generics,
    koruma: &TokenStream2,
) -> MainErrorRender {
    let render_plan = plan.main_error_render_plan();

    let main_error_usages: Vec<Type> = render_plan
        .fields
        .iter()
        .map(|field| main_error_storage_type(field, generics, koruma))
        .collect();
    let helper_generics = helper_generics_for_usages(generics, &main_error_usages);
    let path = helper_generics.type_path(error_struct_name);
    let definition = {
        let definition = &helper_generics.definition;
        quote! { #definition }
    };
    let impl_generics = helper_generics.impl_generics;
    let ty_generics = helper_generics.ty_generics;
    let where_clause = helper_generics.where_clause;

    let fields: Vec<TokenStream2> = render_plan
        .fields
        .iter()
        .map(|field| render_main_error_field_storage(field, generics, koruma))
        .collect();

    let getter_methods: Vec<TokenStream2> = render_plan
        .fields
        .iter()
        .map(|field| render_main_error_getter(field, struct_name, generics, koruma))
        .collect();

    let is_empty_checks: Vec<TokenStream2> = render_plan
        .fields
        .iter()
        .map(render_main_error_is_empty_check)
        .collect();
    let is_empty_body = if is_empty_checks.is_empty() {
        quote! { true }
    } else {
        quote! { #(#is_empty_checks)&&* }
    };

    let defaults: Vec<TokenStream2> = render_plan
        .fields
        .iter()
        .map(|field| render_main_error_default(field, koruma))
        .collect();

    MainErrorRender {
        definition,
        impl_generics,
        ty_generics,
        where_clause,
        path,
        fields,
        getter_methods,
        is_empty_body,
        defaults,
    }
}

fn main_error_storage_type(
    field: &PlannedMainErrorField<'_>,
    generics: &Generics,
    koruma: &TokenStream2,
) -> Type {
    let field_plan = field.field;
    let inner_ty = field_plan.inner_type();
    match field.storage {
        PlannedMainErrorStorage::NestedDirect => {
            syn::parse_quote! { <#inner_ty as #koruma::ValidateExt>::Error }
        },
        PlannedMainErrorStorage::NestedOptional => {
            syn::parse_quote! { Option<<#inner_ty as #koruma::ValidateExt>::Error> }
        },
        PlannedMainErrorStorage::FieldError => field_error_type(generics, field_plan, koruma),
    }
}

fn render_main_error_field_storage(
    field: &PlannedMainErrorField<'_>,
    generics: &Generics,
    koruma: &TokenStream2,
) -> TokenStream2 {
    let field_plan = field.field;
    let field_name = &field_plan.name;
    let inner_ty = field_plan.inner_type();
    match field.storage {
        PlannedMainErrorStorage::NestedDirect => {
            quote! { #field_name: <#inner_ty as #koruma::ValidateExt>::Error }
        },
        PlannedMainErrorStorage::NestedOptional => {
            quote! { #field_name: Option<<#inner_ty as #koruma::ValidateExt>::Error> }
        },
        PlannedMainErrorStorage::FieldError => {
            let field_error_path = field_error_type_path(generics, field_plan, koruma);
            quote! { #field_name: #field_error_path }
        },
    }
}

fn render_main_error_getter(
    field: &PlannedMainErrorField<'_>,
    struct_name: &Ident,
    generics: &Generics,
    koruma: &TokenStream2,
) -> TokenStream2 {
    let field_plan = field.field;
    let field_name = &field_plan.name;
    let field_name_str = field_name.to_string();
    let struct_name_str = struct_name.to_string();
    let inner_ty = field_plan.inner_type();

    match field.getter {
        PlannedErrorGetter::NestedDirect => quote! {
            #[doc = concat!("Returns validation errors for the nested `", #field_name_str, "` field of [`", #struct_name_str, "`].")]
            pub fn #field_name(&self) -> &<#inner_ty as #koruma::ValidateExt>::Error {
                &self.#field_name
            }
        },
        PlannedErrorGetter::NestedOptional => quote! {
            #[doc = concat!("Returns validation errors for the nested `", #field_name_str, "` field of [`", #struct_name_str, "`], if any.")]
            pub fn #field_name(&self) -> Option<&<#inner_ty as #koruma::ValidateExt>::Error> {
                self.#field_name.as_ref()
            }
        },
        PlannedErrorGetter::FieldError => {
            let field_error_path = field_error_type_path(generics, field_plan, koruma);
            quote! {
                #[doc = concat!("Returns validation errors for the `", #field_name_str, "` field of [`", #struct_name_str, "`].")]
                pub fn #field_name(&self) -> &#field_error_path {
                    &self.#field_name
                }
            }
        },
        PlannedErrorGetter::NewtypeInnerDirect => quote! {
            #[doc = concat!("Returns validation errors for the `", #field_name_str, "` field of [`", #struct_name_str, "`].")]
            pub fn #field_name(&self) -> &<#inner_ty as #koruma::ValidateExt>::Error {
                &self.#field_name.inner
            }
        },
        PlannedErrorGetter::NewtypeInnerOptional => quote! {
            #[doc = concat!("Returns validation errors for the `", #field_name_str, "` field of [`", #struct_name_str, "`], if any.")]
            pub fn #field_name(&self) -> Option<&<#inner_ty as #koruma::ValidateExt>::Error> {
                self.#field_name.inner.as_ref()
            }
        },
    }
}

fn render_main_error_is_empty_check(field: &PlannedMainErrorField<'_>) -> TokenStream2 {
    let field_name = &field.field.name;
    match field.is_empty {
        PlannedErrorIsEmpty::NestedDirect => quote! { self.#field_name.is_empty() },
        PlannedErrorIsEmpty::NestedOptional => quote! { self.#field_name.is_none() },
        PlannedErrorIsEmpty::FieldError => quote! { self.#field_name.is_empty() },
    }
}

fn render_main_error_default(
    field: &PlannedMainErrorField<'_>,
    koruma: &TokenStream2,
) -> TokenStream2 {
    let field_plan = field.field;
    let field_name = &field_plan.name;
    let inner_ty = field_plan.inner_type();
    let field_error_struct_name = &field_plan.generated_names.field_error_struct;

    match field.default {
        PlannedErrorDefault::NestedDirect => {
            quote! { #field_name: <#inner_ty as #koruma::ValidateExt>::Error::default() }
        },
        PlannedErrorDefault::None => quote! { #field_name: None },
        PlannedErrorDefault::FieldErrorDefault => {
            quote! { #field_name: #field_error_struct_name::default() }
        },
        PlannedErrorDefault::NewtypeWithValidators { inner_optional } => {
            let field_validator_defaults = field_validator_defaults(field_plan);
            let inner_default = if inner_optional {
                quote! { None }
            } else {
                quote! { <#inner_ty as #koruma::ValidateExt>::Error::default() }
            };

            quote! {
                #field_name: #field_error_struct_name {
                    #(#field_validator_defaults,)*
                    inner: #inner_default
                }
            }
        },
        PlannedErrorDefault::Regular {
            has_field_validators,
            has_element_validators,
        } => {
            let field_validator_defaults = field_validator_defaults(field_plan);
            match (has_field_validators, has_element_validators) {
                (true, true) => quote! {
                    #field_name: #field_error_struct_name {
                        #(#field_validator_defaults),*,
                        element_errors: Vec::new()
                    }
                },
                (true, false) => quote! {
                    #field_name: #field_error_struct_name {
                        #(#field_validator_defaults),*
                    }
                },
                (false, true) => quote! {
                    #field_name: #field_error_struct_name {
                        element_errors: Vec::new()
                    }
                },
                (false, false) => quote! {
                    #field_name: #field_error_struct_name {}
                },
            }
        },
    }
}

fn field_validator_defaults(field_plan: &FieldPlan) -> Vec<TokenStream2> {
    field_plan
        .field_validators()
        .iter()
        .map(|validator| {
            let validator_snake = &validator.field_ident;
            quote! { #validator_snake: None }
        })
        .collect()
}
