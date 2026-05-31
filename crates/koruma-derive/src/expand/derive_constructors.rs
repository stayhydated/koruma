use crate::expand::plan::ValidationPlan;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Fields, Ident};

pub(crate) fn render_try_new_fn(
    plan: &ValidationPlan,
    fields: &Fields,
    struct_name_str: &str,
    main_error_path: &TokenStream2,
) -> TokenStream2 {
    if !plan.struct_options.try_new() {
        return quote! {};
    }

    let all_field_params: Vec<TokenStream2> = fields
        .iter()
        .enumerate()
        .map(|(idx, f)| {
            let name = match &f.ident {
                Some(ident) => quote! { #ident },
                None => {
                    let ident =
                        syn::Ident::new(&format!("_{}", idx), proc_macro2::Span::call_site());
                    quote! { #ident }
                },
            };
            let ty = &f.ty;
            quote! { #name: #ty }
        })
        .collect();

    let struct_init = match fields {
        syn::Fields::Named(_) => {
            let all_field_members: Vec<syn::Member> = fields
                .iter()
                .enumerate()
                .map(|(idx, f)| match &f.ident {
                    Some(ident) => syn::Member::Named(ident.clone()),
                    None => syn::Member::Unnamed(syn::Index::from(idx)),
                })
                .collect();
            quote! {
                let instance = Self {
                    #(#all_field_members),*
                };
            }
        },
        syn::Fields::Unnamed(_) => {
            let all_field_names: Vec<Ident> = fields
                .iter()
                .enumerate()
                .map(|(idx, f)| match &f.ident {
                    Some(ident) => ident.clone(),
                    None => syn::Ident::new(&format!("_{}", idx), proc_macro2::Span::call_site()),
                })
                .collect();
            quote! {
                let instance = Self(#(#all_field_names),*);
            }
        },
        syn::Fields::Unit => {
            quote! {
                let instance = Self;
            }
        },
    };

    quote! {
        #[doc = concat!("Creates a new `", #struct_name_str, "` instance and validates it.\n\nReturns `Ok(instance)` if all validations pass, or `Err(error)` with the validation failures.")]
        pub fn try_new(#(#all_field_params),*) -> Result<Self, #main_error_path> {
            #struct_init
            instance.validate()?;
            Ok(instance)
        }
    }
}

pub(crate) fn render_try_from_impl(
    plan: &ValidationPlan,
    struct_name: &Ident,
    impl_generics: &TokenStream2,
    ty_generics: &TokenStream2,
    where_clause: &TokenStream2,
    main_error_path: &TokenStream2,
) -> TokenStream2 {
    if !plan.struct_options.try_from() {
        return quote! {};
    }

    let Some(field_plan) = plan.struct_newtype() else {
        return quote! {};
    };
    let inner_ty = &field_plan.source.ty;

    let struct_init = match &field_plan.source.member {
        syn::Member::Named(ident) => {
            quote! { Self { #ident: value } }
        },
        syn::Member::Unnamed(_) => {
            quote! { Self(value) }
        },
    };

    quote! {
        impl #impl_generics TryFrom<#inner_ty> for #struct_name #ty_generics #where_clause {
            type Error = #main_error_path;

            fn try_from(value: #inner_ty) -> Result<Self, Self::Error> {
                let instance = #struct_init;
                instance.validate()?;
                Ok(instance)
            }
        }
    }
}
