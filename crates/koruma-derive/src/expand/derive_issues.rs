use crate::expand::codegen::helper_generics_for_usages;
use crate::expand::derive_shared::field_error_type;
use crate::expand::plan::{
    PlannedMainErrorField, PlannedMainErrorStorage, PlannedValidator, ValidationPlan,
};
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Generics, Ident, Type};

pub(crate) fn render_validation_issues_impl(
    plan: &ValidationPlan,
    error_struct_name: &Ident,
    generics: &Generics,
    koruma: &TokenStream2,
) -> TokenStream2 {
    let render_plan = plan.main_error_render_plan();
    let main_error_usages = render_plan
        .fields
        .iter()
        .map(|field| {
            let field_plan = field.field;
            let inner_ty = field_plan.inner_type();
            match field.storage {
                PlannedMainErrorStorage::NestedDirect => {
                    syn::parse_quote! { <#inner_ty as #koruma::ValidateExt>::Error }
                },
                PlannedMainErrorStorage::NestedOptional => {
                    syn::parse_quote! { Option<<#inner_ty as #koruma::ValidateExt>::Error> }
                },
                PlannedMainErrorStorage::FieldError => {
                    field_error_type(generics, field_plan, koruma)
                },
            }
        })
        .collect::<Vec<Type>>();
    let helper_generics = helper_generics_for_usages(generics, &main_error_usages);
    let impl_generics = &helper_generics.impl_generics;
    let ty_generics = &helper_generics.ty_generics;
    let where_clause = &helper_generics.where_clause;
    let issue_extractors = render_plan
        .fields
        .iter()
        .map(|field| render_field_issues(field, koruma))
        .collect::<Vec<_>>();

    quote! {
        impl #impl_generics #koruma::ValidationIssues for #error_struct_name #ty_generics #where_clause {
            fn issues(&self) -> ::std::vec::Vec<#koruma::ValidationIssue> {
                let mut __koruma_issues = ::std::vec::Vec::new();
                #(#issue_extractors)*
                __koruma_issues
            }
        }
    }
}

fn render_field_issues(
    main_field: &PlannedMainErrorField<'_>,
    koruma: &TokenStream2,
) -> TokenStream2 {
    let field = main_field.field;
    let field_name = &field.name;
    let field_name_str = field_name.to_string();

    if field.is_nested() {
        return match main_field.storage {
            PlannedMainErrorStorage::NestedDirect => {
                quote! {
                    if !#koruma::ValidationError::is_empty(&self.#field_name) {
                        __koruma_issues.push(#koruma::ValidationIssue::field(
                            #field_name_str,
                            "nested",
                            None,
                            ::std::format!("nested validation failed for `{}`", #field_name_str),
                            ::std::vec::Vec::new(),
                        ));
                    }
                }
            },
            PlannedMainErrorStorage::NestedOptional => {
                quote! {
                    if self.#field_name.as_ref().is_some_and(|__koruma_error| !#koruma::ValidationError::is_empty(__koruma_error)) {
                        __koruma_issues.push(#koruma::ValidationIssue::field(
                            #field_name_str,
                            "nested",
                            None,
                            ::std::format!("nested validation failed for `{}`", #field_name_str),
                            ::std::vec::Vec::new(),
                        ));
                    }
                }
            },
            PlannedMainErrorStorage::FieldError => quote! {},
        };
    }

    let field_validator_issues = field
        .field_validators()
        .iter()
        .map(|validator| render_validator_issue(field_name, &field_name_str, validator, koruma))
        .collect::<Vec<_>>();
    let element_validator_issues = field
        .element_validators()
        .iter()
        .map(|validator| render_element_validator_issue(&field_name_str, validator, koruma))
        .collect::<Vec<_>>();

    let inner_issue = if field.is_newtype() {
        if field.field_optional() {
            quote! {
                if self.#field_name.inner().is_some_and(|__koruma_error| !#koruma::ValidationError::is_empty(__koruma_error)) {
                    __koruma_issues.push(#koruma::ValidationIssue::field(
                        #field_name_str,
                        "newtype",
                        None,
                        ::std::format!("newtype validation failed for `{}`", #field_name_str),
                        ::std::vec::Vec::new(),
                    ));
                }
            }
        } else {
            quote! {
                if !#koruma::ValidationError::is_empty(self.#field_name.inner()) {
                    __koruma_issues.push(#koruma::ValidationIssue::field(
                        #field_name_str,
                        "newtype",
                        None,
                        ::std::format!("newtype validation failed for `{}`", #field_name_str),
                        ::std::vec::Vec::new(),
                    ));
                }
            }
        }
    } else {
        quote! {}
    };

    let element_loop = if element_validator_issues.is_empty() {
        quote! {}
    } else {
        quote! {
            for (__koruma_index, __koruma_element_error) in self.#field_name.element_errors() {
                #(#element_validator_issues)*
            }
        }
    };

    quote! {
        #(#field_validator_issues)*
        #inner_issue
        #element_loop
    }
}

fn render_validator_issue(
    field_name: &Ident,
    field_name_str: &str,
    validator: &PlannedValidator,
    koruma: &TokenStream2,
) -> TokenStream2 {
    let slot = &validator.field_ident;
    let validator_ty = validator.validator_type.as_type();
    let label = option_str_tokens(validator.label.as_ref().map(ToString::to_string));
    let doc_name = validator.doc_name();

    quote! {
        if self.#field_name.#slot().is_some() {
            __koruma_issues.push(#koruma::ValidationIssue::field(
                #field_name_str,
                ::core::any::type_name::<#validator_ty>(),
                #label,
                ::std::format!("validation failed for `{}`", #doc_name),
                ::std::vec::Vec::new(),
            ));
        }
    }
}

fn render_element_validator_issue(
    field_name_str: &str,
    validator: &PlannedValidator,
    koruma: &TokenStream2,
) -> TokenStream2 {
    let slot = &validator.field_ident;
    let validator_ty = validator.validator_type.as_type();
    let label = option_str_tokens(validator.label.as_ref().map(ToString::to_string));
    let doc_name = validator.doc_name();

    quote! {
        if __koruma_element_error.#slot().is_some() {
            __koruma_issues.push(#koruma::ValidationIssue::element(
                #field_name_str,
                *__koruma_index,
                ::core::any::type_name::<#validator_ty>(),
                #label,
                ::std::format!("validation failed for `{}`", #doc_name),
                ::std::vec::Vec::new(),
            ));
        }
    }
}

fn option_str_tokens(value: Option<String>) -> TokenStream2 {
    match value {
        Some(value) => quote! { Some(#value) },
        None => quote! { None },
    }
}
