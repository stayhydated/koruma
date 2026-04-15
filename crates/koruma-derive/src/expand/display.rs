use heck::ToUpperCamelCase;
use koruma_derive_core::{
    FieldInfo, ParseFieldResult, ValidatorAttr, option_inner_type, parse_field,
};
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};

use crate::expand::codegen::{helper_generics_for_usages, validator_type_for_field};
use syn::DeriveInput;

/// Core expansion logic for the `#[derive(KorumaAllDisplay)]` derive macro.
///
/// Generates `Display` implementations for the `{Struct}{Field}KorumaValidator` enums
/// returned by the `all()` method. Each variant delegates to its inner validator's Display.
pub fn expand_koruma_all_display(input: DeriveInput) -> Result<TokenStream2, syn::Error> {
    let struct_name = &input.ident;
    let generics = &input.generics;

    let fields = match &input.data {
        syn::Data::Struct(data) => &data.fields,
        _ => {
            return Err(syn::Error::new_spanned(
                &input,
                "KorumaAllDisplay can only be derived for structs",
            ));
        },
    };

    // Parse all fields and extract validation info
    let mut field_infos: Vec<FieldInfo> = Vec::new();
    for (i, field) in fields.iter().enumerate() {
        match parse_field(field, i) {
            ParseFieldResult::Valid(info) => field_infos.push(*info),
            ParseFieldResult::Skip => {},
            ParseFieldResult::Error(e) => return Err(e),
        }
    }

    // Generate Display impls for each field's validator enum
    let display_impls: Vec<TokenStream2> = field_infos
        .iter()
        .filter(|f| !f.validation.field_validators.is_empty())
        .map(|f| {
            let field_name = &f.name;
            let field_ty = &f.ty;
            let enum_name = format_ident!(
                "{}{}KorumaValidator",
                struct_name,
                field_name.to_string().to_upper_camel_case()
            );
            let mut helper_usages: Vec<TokenStream2> = f
                .validation
                .field_validators
                .iter()
                .map(|v| {
                    let vtype = validator_type_for_field(v, field_ty, false);
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

            let match_arms: Vec<TokenStream2> = f
                .validation
                .field_validators
                .iter()
                .map(|v: &ValidatorAttr| {
                    let variant_name =
                        format_ident!("{}", v.name().to_string().to_upper_camel_case());
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
    let element_display_impls: Vec<TokenStream2> = field_infos
        .iter()
        .filter(|f| !f.validation.element_validators.is_empty())
        .map(|f| {
            let field_name = &f.name;
            let field_ty = &f.ty;
            let enum_name = format_ident!(
                "{}{}ElementKorumaValidator",
                struct_name,
                field_name.to_string().to_upper_camel_case()
            );
            let helper_usages: Vec<TokenStream2> = f
                .validation
                .element_validators
                .iter()
                .map(|v| {
                    let vtype = validator_type_for_field(v, field_ty, true);
                    quote! { #vtype }
                })
                .collect();
            let helper_generics = helper_generics_for_usages(generics, &helper_usages);
            let helper_impl_generics = &helper_generics.impl_generics;
            let helper_ty_generics = &helper_generics.ty_generics;
            let helper_where_clause = &helper_generics.where_clause;

            let match_arms: Vec<TokenStream2> = f
                .validation
                .element_validators
                .iter()
                .map(|v: &ValidatorAttr| {
                    let variant_name =
                        format_ident!("{}", v.name().to_string().to_upper_camel_case());
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
