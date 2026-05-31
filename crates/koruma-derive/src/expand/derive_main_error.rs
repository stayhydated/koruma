use crate::expand::codegen::helper_generics_for_usages;
use crate::expand::derive_shared::{field_error_type, field_error_type_path};
use crate::expand::plan::ValidationPlan;
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
    let struct_is_newtype = plan.struct_newtype().is_some();
    let field_infos = plan.field_infos();

    let main_error_usages: Vec<Type> = field_infos
        .iter()
        .zip(plan.fields.iter())
        .map(|(_f, field_plan)| {
            if field_plan.is_nested() {
                let inner_ty = field_plan.inner_type();
                if struct_is_newtype && !field_plan.field_optional() {
                    syn::parse_quote! { <#inner_ty as #koruma::ValidateExt>::Error }
                } else {
                    syn::parse_quote! { Option<<#inner_ty as #koruma::ValidateExt>::Error> }
                }
            } else {
                field_error_type(generics, field_plan, koruma)
                    .expect("non-nested fields should have a generated error type")
            }
        })
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

    let fields: Vec<TokenStream2> = field_infos
        .iter()
        .zip(plan.fields.iter())
        .map(|(f, field_plan)| {
            let field_name = &f.name;
            if field_plan.is_nested() {
                let inner_ty = field_plan.inner_type();
                if struct_is_newtype && !field_plan.field_optional() {
                    quote! { #field_name: <#inner_ty as #koruma::ValidateExt>::Error }
                } else {
                    quote! { #field_name: Option<<#inner_ty as #koruma::ValidateExt>::Error> }
                }
            } else {
                let field_error_path = field_error_type_path(generics, field_plan, koruma)
                    .expect("non-nested fields should have a generated error type");
                quote! { #field_name: #field_error_path }
            }
        })
        .collect();

    let getter_methods: Vec<TokenStream2> = field_infos
        .iter()
        .zip(plan.fields.iter())
        .map(|(f, field_plan)| {
            let field_name = &f.name;
            let field_name_str = field_name.to_string();
            let struct_name_str = struct_name.to_string();
            if field_plan.is_nested() {
                let inner_ty = field_plan.inner_type();
                if struct_is_newtype && !field_plan.field_optional() {
                    quote! {
                        #[doc = concat!("Returns validation errors for the nested `", #field_name_str, "` field of [`", #struct_name_str, "`].")]
                        pub fn #field_name(&self) -> &<#inner_ty as #koruma::ValidateExt>::Error {
                            &self.#field_name
                        }
                    }
                } else {
                    quote! {
                        #[doc = concat!("Returns validation errors for the nested `", #field_name_str, "` field of [`", #struct_name_str, "`], if any.")]
                        pub fn #field_name(&self) -> Option<&<#inner_ty as #koruma::ValidateExt>::Error> {
                            self.#field_name.as_ref()
                        }
                    }
                }
            } else if field_plan.is_newtype() {
                let field_error_path = field_error_type_path(generics, field_plan, koruma)
                    .expect("non-nested fields should have a generated error type");
                if field_plan.has_field_validators() {
                    quote! {
                        #[doc = concat!("Returns validation errors for the `", #field_name_str, "` field of [`", #struct_name_str, "`].")]
                        pub fn #field_name(&self) -> &#field_error_path {
                            &self.#field_name
                        }
                    }
                } else {
                    let inner_ty = field_plan.inner_type();
                    if field_plan.field_optional() {
                        quote! {
                            #[doc = concat!("Returns validation errors for the `", #field_name_str, "` field of [`", #struct_name_str, "`], if any.")]
                            pub fn #field_name(&self) -> Option<&<#inner_ty as #koruma::ValidateExt>::Error> {
                                self.#field_name.inner.as_ref()
                            }
                        }
                    } else {
                        quote! {
                            #[doc = concat!("Returns validation errors for the `", #field_name_str, "` field of [`", #struct_name_str, "`].")]
                            pub fn #field_name(&self) -> &<#inner_ty as #koruma::ValidateExt>::Error {
                                &self.#field_name.inner
                            }
                        }
                    }
                }
            } else {
                let field_error_path = field_error_type_path(generics, field_plan, koruma)
                    .expect("non-nested fields should have a generated error type");
                quote! {
                    #[doc = concat!("Returns validation errors for the `", #field_name_str, "` field of [`", #struct_name_str, "`].")]
                    pub fn #field_name(&self) -> &#field_error_path {
                        &self.#field_name
                    }
                }
            }
        })
        .collect();

    let is_empty_checks: Vec<TokenStream2> = field_infos
        .iter()
        .zip(plan.fields.iter())
        .map(|(f, field_plan)| {
            let field_name = &f.name;
            if field_plan.is_nested() {
                if struct_is_newtype && !field_plan.field_optional() {
                    quote! { self.#field_name.is_empty() }
                } else {
                    quote! { self.#field_name.is_none() }
                }
            } else {
                quote! { self.#field_name.is_empty() }
            }
        })
        .collect();
    let is_empty_body = if is_empty_checks.is_empty() {
        quote! { true }
    } else {
        quote! { #(#is_empty_checks)&&* }
    };

    let defaults: Vec<TokenStream2> = field_infos
        .iter()
        .zip(plan.fields.iter())
        .map(|(f, field_plan)| {
            let field_name = &f.name;

            if field_plan.is_nested() {
                let inner_ty = field_plan.inner_type();
                if struct_is_newtype && !field_plan.field_optional() {
                    return quote! {
                        #field_name: <#inner_ty as #koruma::ValidateExt>::Error::default()
                    };
                }
                return quote! { #field_name: None };
            }

            let field_error_struct_name = &field_plan.generated_names.field_error_struct;

            if field_plan.is_newtype() {
                let inner_ty = field_plan.inner_type();
                let field_is_optional = field_plan.field_optional();

                if field_plan.has_field_validators() {
                    let field_validator_defaults: Vec<TokenStream2> = field_plan
                        .field_validators()
                        .iter()
                        .map(|v| {
                            let validator_snake = &v.field_ident;
                            quote! { #validator_snake: None }
                        })
                        .collect();
                    let inner_default = if field_is_optional {
                        quote! { None }
                    } else {
                        quote! { <#inner_ty as #koruma::ValidateExt>::Error::default() }
                    };

                    return quote! {
                        #field_name: #field_error_struct_name {
                            #(#field_validator_defaults,)*
                            inner: #inner_default
                        }
                    };
                }

                return quote! {
                    #field_name: #field_error_struct_name::default()
                };
            }

            let field_validator_defaults: Vec<TokenStream2> = field_plan
                .field_validators()
                .iter()
                .map(|v| {
                    let validator_snake = &v.field_ident;
                    quote! { #validator_snake: None }
                })
                .collect();

            if field_plan.has_element_validators() && !field_plan.has_field_validators() {
                quote! {
                    #field_name: #field_error_struct_name {
                        element_errors: Vec::new()
                    }
                }
            } else if field_plan.has_element_validators() {
                quote! {
                    #field_name: #field_error_struct_name {
                        #(#field_validator_defaults),*,
                        element_errors: Vec::new()
                    }
                }
            } else {
                quote! {
                    #field_name: #field_error_struct_name {
                        #(#field_validator_defaults),*
                    }
                }
            }
        })
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
