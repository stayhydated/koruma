use crate::expand::derive_shared::field_error_type_path;
use crate::expand::derive_validation::render_validation_checks_for_newtype_inner;
use crate::expand::plan::ValidationPlan;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Generics, Ident, Member};

pub(crate) struct NewtypeDerefInputs<'a> {
    pub plan: &'a ValidationPlan,
    pub generics: &'a Generics,
    pub error_struct_name: &'a Ident,
    pub main_error_impl_generics: &'a TokenStream2,
    pub main_error_ty_generics: &'a TokenStream2,
    pub main_error_where_clause: &'a TokenStream2,
    pub koruma: &'a TokenStream2,
}

pub(crate) struct NewtypeValueInputs<'a> {
    pub plan: &'a ValidationPlan,
    pub struct_name: &'a Ident,
    pub error_struct_name: &'a Ident,
    pub impl_generics: &'a TokenStream2,
    pub ty_generics: &'a TokenStream2,
    pub where_clause: &'a TokenStream2,
    pub error_defaults: &'a [TokenStream2],
    pub koruma: &'a TokenStream2,
}

pub(crate) fn render_newtype_marker_impl(
    plan: &ValidationPlan,
    struct_name: &Ident,
    impl_generics: &TokenStream2,
    ty_generics: &TokenStream2,
    where_clause: &TokenStream2,
    koruma: &TokenStream2,
) -> TokenStream2 {
    if plan.struct_newtype().is_none() {
        return quote! {};
    }

    quote! {
        impl #impl_generics #koruma::NewtypeValidation for #struct_name #ty_generics #where_clause {}
    }
}

pub(crate) fn render_newtype_value_impl(
    input: NewtypeValueInputs<'_>,
) -> Result<TokenStream2, syn::Error> {
    let Some(field_plan) = input.plan.struct_newtype() else {
        return Ok(quote! {});
    };

    let inner_ty = &field_plan.source.ty;
    let field_member = &field_plan.source.member;
    let struct_init = match field_member {
        Member::Named(ident) => quote! { Self { #ident: value } },
        Member::Unnamed(_) => quote! { Self(value) },
    };
    let validation_checks = render_validation_checks_for_newtype_inner(input.plan, input.koruma)?;

    let struct_name = input.struct_name;
    let error_struct_name = input.error_struct_name;
    let impl_generics = input.impl_generics;
    let ty_generics = input.ty_generics;
    let where_clause = input.where_clause;
    let error_defaults = input.error_defaults;
    let koruma = input.koruma;

    Ok(quote! {
        impl #impl_generics #koruma::NewtypeValue for #struct_name #ty_generics #where_clause {
            type Inner = #inner_ty;

            fn as_inner(&self) -> &Self::Inner {
                &self.#field_member
            }

            fn into_inner(self) -> Self::Inner
            where
                Self: Sized,
            {
                self.#field_member
            }

            fn validate_inner(
                __koruma_newtype_inner_value: &Self::Inner
            ) -> Result<(), Self::Error> {
                let mut error = #error_struct_name {
                    #(#error_defaults),*
                };
                let mut has_error = false;

                #(#validation_checks)*

                if has_error {
                    Err(error)
                } else {
                    Ok(())
                }
            }
        }

        impl #impl_generics #koruma::NewtypeTryFromInner
            for #struct_name #ty_generics #where_clause
        {
            fn try_from_inner(value: Self::Inner) -> Result<Self, Self::Error>
            where
                Self: Sized,
            {
                <Self as #koruma::NewtypeValue>::validate_inner(&value)?;
                Ok(#struct_init)
            }
        }
    })
}

pub(crate) fn render_newtype_deref_impl(input: NewtypeDerefInputs<'_>) -> TokenStream2 {
    let Some(field_plan) = input.plan.struct_newtype() else {
        return quote! {};
    };

    let field_name = &field_plan.name;

    if field_plan.is_nested() {
        let inner_ty = field_plan.inner_type();

        if field_plan.field_optional() {
            quote! {}
        } else {
            let main_error_impl_generics = input.main_error_impl_generics;
            let main_error_ty_generics = input.main_error_ty_generics;
            let main_error_where_clause = input.main_error_where_clause;
            let error_struct_name = input.error_struct_name;
            let koruma = input.koruma;
            quote! {
                impl #main_error_impl_generics core::ops::Deref for #error_struct_name #main_error_ty_generics #main_error_where_clause {
                    type Target = <#inner_ty as #koruma::ValidateExt>::Error;

                    fn deref(&self) -> &Self::Target {
                        &self.#field_name
                    }
                }
            }
        }
    } else {
        let field_error_path = field_error_type_path(input.generics, field_plan, input.koruma);
        let main_error_impl_generics = input.main_error_impl_generics;
        let main_error_ty_generics = input.main_error_ty_generics;
        let main_error_where_clause = input.main_error_where_clause;
        let error_struct_name = input.error_struct_name;
        quote! {
            impl #main_error_impl_generics core::ops::Deref for #error_struct_name #main_error_ty_generics #main_error_where_clause {
                type Target = #field_error_path;

                fn deref(&self) -> &Self::Target {
                    &self.#field_name
                }
            }
        }
    }
}
