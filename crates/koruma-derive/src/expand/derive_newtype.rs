use crate::expand::derive_shared::field_error_type_path;
use crate::expand::plan::ValidationPlan;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Generics, Ident};

pub(crate) struct NewtypeDerefInputs<'a> {
    pub plan: &'a ValidationPlan,
    pub generics: &'a Generics,
    pub error_struct_name: &'a Ident,
    pub main_error_impl_generics: &'a TokenStream2,
    pub main_error_ty_generics: &'a TokenStream2,
    pub main_error_where_clause: &'a TokenStream2,
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

pub(crate) fn render_newtype_deref_impl(input: NewtypeDerefInputs<'_>) -> TokenStream2 {
    let Some((_field_info, field_plan)) = input.plan.struct_newtype() else {
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
        let Some(field_error_path) =
            field_error_type_path(input.generics, field_plan, input.koruma)
        else {
            return quote! {};
        };
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
