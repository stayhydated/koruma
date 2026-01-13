use heck::ToUpperCamelCase;
use koruma_derive_core::{FieldInfo, ParseFieldResult, parse_field};
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{DeriveInput, Fields};

/// Core expansion logic for the `#[derive(KorumaAllDisplay)]` derive macro.
///
/// Generates `Display` implementations for the `{Struct}{Field}KorumaValidator` enums
/// returned by the `all()` method. Each variant delegates to its inner validator's Display.
pub fn expand_koruma_all_display(input: DeriveInput) -> Result<TokenStream2, syn::Error> {
    let struct_name = &input.ident;

    let fields = match &input.data {
        syn::Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => {
                return Err(syn::Error::new_spanned(
                    &input,
                    "KorumaAllDisplay only supports structs with named fields",
                ));
            },
        },
        _ => {
            return Err(syn::Error::new_spanned(
                &input,
                "KorumaAllDisplay can only be derived for structs",
            ));
        },
    };

    // Parse all fields and extract validation info
    let mut field_infos: Vec<FieldInfo> = Vec::new();
    for field in fields.iter() {
        match parse_field(field) {
            ParseFieldResult::Valid(info) => field_infos.push(info),
            ParseFieldResult::Skip => {},
            ParseFieldResult::Error(e) => return Err(e),
        }
    }

    // Generate Display impls for each field's validator enum
    let display_impls: Vec<TokenStream2> = field_infos
        .iter()
        .filter(|f| !f.field_validators.is_empty())
        .map(|f| {
            let field_name = &f.name;
            let enum_name = format_ident!(
                "{}{}KorumaValidator",
                struct_name,
                field_name.to_string().to_upper_camel_case()
            );

            let match_arms: Vec<TokenStream2> = f
                .field_validators
                .iter()
                .map(|v| {
                    let variant_name =
                        format_ident!("{}", v.name().to_string().to_upper_camel_case());
                    quote! {
                        #enum_name::#variant_name(v) => ::std::fmt::Display::fmt(v, f)
                    }
                })
                .collect();

            quote! {
                impl ::std::fmt::Display for #enum_name {
                    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                        match self {
                            #(#match_arms),*
                        }
                    }
                }
            }
        })
        .collect();

    // Generate Display impls for element validator enums (if any)
    let element_display_impls: Vec<TokenStream2> = field_infos
        .iter()
        .filter(|f| !f.element_validators.is_empty())
        .map(|f| {
            let field_name = &f.name;
            let enum_name = format_ident!(
                "{}{}ElementKorumaValidator",
                struct_name,
                field_name.to_string().to_upper_camel_case()
            );

            let match_arms: Vec<TokenStream2> = f
                .element_validators
                .iter()
                .map(|v| {
                    let variant_name =
                        format_ident!("{}", v.name().to_string().to_upper_camel_case());
                    quote! {
                        #enum_name::#variant_name(v) => ::std::fmt::Display::fmt(v, f)
                    }
                })
                .collect();

            quote! {
                impl ::std::fmt::Display for #enum_name {
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
