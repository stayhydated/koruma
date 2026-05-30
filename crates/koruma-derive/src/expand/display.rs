use koruma_derive_core::option_inner_type;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

use crate::expand::codegen::helper_generics_for_usages;
use crate::expand::plan::ValidationPlan;
use syn::DeriveInput;

/// Core expansion logic for the `#[derive(KorumaAllDisplay)]` derive macro.
///
/// Generates `Display` implementations for the `{Struct}{Field}KorumaValidator` enums
/// returned by the `all()` method. Each variant delegates to its inner validator's Display.
pub fn expand_koruma_all_display(input: DeriveInput) -> Result<TokenStream2, syn::Error> {
    let generics = &input.generics;

    let plan = ValidationPlan::build(&input, "KorumaAllDisplay")?;

    // Generate Display impls for each field's validator enum
    let display_impls: Vec<TokenStream2> = plan
        .fields
        .iter()
        .zip(plan.field_infos())
        .filter(|(field_plan, _)| !field_plan.field_validators.is_empty())
        .map(|(field_plan, f)| {
            let field_ty = &f.ty;
            let enum_name = &field_plan.generated_names.field_validator_enum;
            let mut helper_usages: Vec<TokenStream2> = field_plan
                .field_validators
                .iter()
                .map(|planned| {
                    let vtype = &planned.validator_type;
                    quote! { #vtype }
                })
                .collect();
            if f.is_newtype() {
                let inner_ty = option_inner_type(field_ty).unwrap_or(field_ty);
                helper_usages.push(quote! { <#inner_ty as koruma::ValidateExt>::Error });
            }
            let helper_generics = helper_generics_for_usages(generics, &helper_usages);
            let helper_impl_generics = &helper_generics.impl_generics;
            let helper_ty_generics = &helper_generics.ty_generics;
            let helper_where_clause = &helper_generics.where_clause;

            let match_arms: Vec<TokenStream2> = field_plan
                .field_validators
                .iter()
                .map(|planned| {
                    let variant_name = &planned.variant_ident;
                    quote! {
                        #enum_name::#variant_name(v) => ::std::fmt::Display::fmt(v, f)
                    }
                })
                .collect();

            // Add Inner variant arm for newtype fields with additional validators
            let inner_arm = if f.is_newtype() {
                Some(quote! {
                    #enum_name::Inner(inner) => ::std::fmt::Display::fmt(inner, f)
                })
            } else {
                None
            };

            quote! {
                impl #helper_impl_generics ::std::fmt::Display for #enum_name #helper_ty_generics #helper_where_clause {
                    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
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
        .filter(|field_plan| !field_plan.element_validators.is_empty())
        .map(|field_plan| {
            let enum_name = &field_plan.generated_names.element_validator_enum;
            let helper_usages: Vec<TokenStream2> = field_plan
                .element_validators
                .iter()
                .map(|planned| {
                    let vtype = &planned.validator_type;
                    quote! { #vtype }
                })
                .collect();
            let helper_generics = helper_generics_for_usages(generics, &helper_usages);
            let helper_impl_generics = &helper_generics.impl_generics;
            let helper_ty_generics = &helper_generics.ty_generics;
            let helper_where_clause = &helper_generics.where_clause;

            let match_arms: Vec<TokenStream2> = field_plan
                .element_validators
                .iter()
                .map(|planned| {
                    let variant_name = &planned.variant_ident;
                    quote! {
                        #enum_name::#variant_name(v) => ::std::fmt::Display::fmt(v, f)
                    }
                })
                .collect();

            quote! {
                impl #helper_impl_generics ::std::fmt::Display for #enum_name #helper_ty_generics #helper_where_clause {
                    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                        match self {
                            #(#match_arms),*
                        }
                    }
                }
            }
        })
        .collect();

    Ok(quote! {
        #(#display_impls)*
        #(#element_display_impls)*
    })
}
