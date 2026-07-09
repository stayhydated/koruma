use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, quote_spanned};

use crate::expand::codegen::ref_enum_generics_for_usages;
use crate::expand::koruma_crate_path;
use crate::expand::plan::ValidationPlan;
use syn::{DeriveInput, Type};

/// Core expansion logic for the `#[derive(KorumaAllDisplay)]` derive macro.
///
/// Generates `Display` implementations for the borrowed
/// `{Struct}{Field}KorumaValidatorRef` enums returned by the `all()` method.
/// Each variant delegates to its inner validator's Display.
pub fn expand_koruma_all_display(input: DeriveInput) -> Result<TokenStream2, syn::Error> {
    let struct_name = &input.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let koruma = koruma_crate_path();

    let plan = ValidationPlan::build(&input, "KorumaAllDisplay")?;
    let requires_koruma_impl = quote! {
        impl #impl_generics #koruma::__private::KorumaAllDisplayRequiresKoruma
            for #struct_name #ty_generics #where_clause
        {
        }
    };

    // Generate Display impls for each field's validator enum
    let display_impls: Vec<TokenStream2> = plan
        .fields
        .iter()
        .filter(|field_plan| field_plan.has_field_validators())
        .map(|field_plan| {
            let enum_name = &field_plan.generated_names.field_validator_ref_enum;
            let mut helper_usages: Vec<Type> = field_plan
                .field_validators()
                .iter()
                .map(|planned| planned.validator_type.as_type())
                .collect();
            let mut display_assertions: Vec<TokenStream2> = field_plan
                .field_validators()
                .iter()
                .map(|planned| {
                    let span = planned.source_span;
                    let validator_ty = planned.validator_type.as_type();
                    quote_spanned! {span=>
                        #koruma::__private::assert_field_display::<#validator_ty>();
                    }
                })
                .collect();
            if field_plan.is_newtype() {
                let inner_ty = field_plan.inner_type();
                helper_usages
                    .push(syn::parse_quote! { <#inner_ty as #koruma::ValidateExt>::Error });
                let span = field_plan
                    .source
                    .marker_span
                    .unwrap_or_else(|| field_plan.name.span());
                display_assertions.push(quote_spanned! {span=>
                    #koruma::__private::assert_newtype_error_display::<<#inner_ty as #koruma::ValidateExt>::Error>();
                });
            }
            let helper_generics = ref_enum_generics_for_usages(generics, &helper_usages);
            let helper_impl_generics = &helper_generics.impl_generics;
            let helper_ty_generics = &helper_generics.ty_generics;
            let helper_where_clause = &helper_generics.where_clause;

            let match_arms: Vec<TokenStream2> = field_plan
                .field_validators()
                .iter()
                .map(|planned| {
                    let variant_name = &planned.variant_ident;
                    quote! {
                        #enum_name::#variant_name(v) => ::std::fmt::Display::fmt(*v, f)
                    }
                })
                .collect();

            // Add Inner variant arm for newtype fields with additional validators
            let inner_arm = if field_plan.is_newtype() {
                Some(quote! {
                    #enum_name::Inner(inner) => ::std::fmt::Display::fmt(*inner, f)
                })
            } else {
                None
            };

            quote! {
                impl #helper_impl_generics ::std::fmt::Display for #enum_name #helper_ty_generics #helper_where_clause {
                    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                        #(#display_assertions)*

                        match self {
                            #(#match_arms,)*
                            #inner_arm
                        }
                    }
                }
            }
        })
        .collect();

    // Generate Display impls for element validator enums (if any)
    let element_display_impls: Vec<TokenStream2> = plan
        .fields
        .iter()
        .filter(|field_plan| field_plan.has_element_validators())
        .map(|field_plan| {
            let enum_name = &field_plan.generated_names.element_validator_ref_enum;
            let helper_usages: Vec<Type> = field_plan
                .element_validators()
                .iter()
                .map(|planned| planned.validator_type.as_type())
                .collect();
            let display_assertions: Vec<TokenStream2> = field_plan
                .element_validators()
                .iter()
                .map(|planned| {
                    let span = planned.source_span;
                    let validator_ty = planned.validator_type.as_type();
                    quote_spanned! {span=>
                        #koruma::__private::assert_element_display::<#validator_ty>();
                    }
                })
                .collect();
            let helper_generics = ref_enum_generics_for_usages(generics, &helper_usages);
            let helper_impl_generics = &helper_generics.impl_generics;
            let helper_ty_generics = &helper_generics.ty_generics;
            let helper_where_clause = &helper_generics.where_clause;

            let match_arms: Vec<TokenStream2> = field_plan
                .element_validators()
                .iter()
                .map(|planned| {
                    let variant_name = &planned.variant_ident;
                    quote! {
                        #enum_name::#variant_name(v) => ::std::fmt::Display::fmt(*v, f)
                    }
                })
                .collect();

            quote! {
                impl #helper_impl_generics ::std::fmt::Display for #enum_name #helper_ty_generics #helper_where_clause {
                    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                        #(#display_assertions)*

                        match self {
                            #(#match_arms),*
                        }
                    }
                }
            }
        })
        .collect();

    Ok(quote! {
        #requires_koruma_impl
        #(#display_impls)*
        #(#element_display_impls)*
    })
}
