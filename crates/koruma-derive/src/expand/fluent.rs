use heck::ToUpperCamelCase;
use koruma_derive_core::{FieldInfo, ParseFieldResult, ValidatorAttr, parse_field};
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{DeriveInput, Fields};

/// Core expansion logic for the `#[derive(KorumaAllFluent)]` derive macro.
///
/// Generates `ToFluentString` implementations for the `{Struct}{Field}KorumaValidator` enums
/// returned by the `all()` method. Each variant delegates to its inner validator's ToFluentString.
#[cfg(feature = "fluent")]
pub fn expand_koruma_all_fluent(input: DeriveInput) -> Result<TokenStream2, syn::Error> {
    let struct_name = &input.ident;

    let fields = match &input.data {
        syn::Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => {
                return Err(syn::Error::new_spanned(
                    &input,
                    "KorumaAllFluent only supports structs with named fields",
                ));
            },
        },
        _ => {
            return Err(syn::Error::new_spanned(
                &input,
                "KorumaAllFluent can only be derived for structs",
            ));
        },
    };

    // Parse all fields and extract validation info
    let mut field_infos: Vec<FieldInfo> = Vec::new();
    for field in fields.iter() {
        match parse_field(field) {
            ParseFieldResult::Valid(info) => field_infos.push(*info),
            ParseFieldResult::Skip => {},
            ParseFieldResult::Error(e) => return Err(e),
        }
    }

    // Generate ToFluentString impls for each field's validator enum
    let fluent_impls: Vec<TokenStream2> = field_infos
        .iter()
        .filter(|f| !f.validation.field_validators.is_empty())
        .map(|f| {
            let field_name = &f.name;
            let enum_name = format_ident!(
                "{}{}KorumaValidator",
                struct_name,
                field_name.to_string().to_upper_camel_case()
            );

            let match_arms: Vec<TokenStream2> = f
                .validation.field_validators
                .iter()
                .map(|v: &ValidatorAttr| {
                    let variant_name =
                        format_ident!("{}", v.name().to_string().to_upper_camel_case());
                    quote! {
                        #enum_name::#variant_name(v) => v.to_fluent_string()
                    }
                })
                .collect();

            quote! {
                impl koruma::es_fluent::ToFluentString for #enum_name {
                    fn to_fluent_string(&self) -> String {
                        use koruma::es_fluent::ToFluentString;
                        match self {
                            #(#match_arms),*
                        }
                    }
                }
            }
        })
        .collect();

    // Generate ToFluentString impls for element validator enums (if any)
    let element_fluent_impls: Vec<TokenStream2> = field_infos
        .iter()
        .filter(|f| !f.validation.element_validators.is_empty())
        .map(|f| {
            let field_name = &f.name;
            let enum_name = format_ident!(
                "{}{}ElementKorumaValidator",
                struct_name,
                field_name.to_string().to_upper_camel_case()
            );

            let match_arms: Vec<TokenStream2> = f
                .validation.element_validators
                .iter()
                .map(|v: &ValidatorAttr| {
                    let variant_name =
                        format_ident!("{}", v.name().to_string().to_upper_camel_case());
                    quote! {
                        #enum_name::#variant_name(v) => v.to_fluent_string()
                    }
                })
                .collect();

            quote! {
                impl koruma::es_fluent::ToFluentString for #enum_name {
                    fn to_fluent_string(&self) -> String {
                        use koruma::es_fluent::ToFluentString;
                        match self {
                            #(#match_arms),*
                        }
                    }
                }
            }
        })
        .collect();

    Ok(quote! {
        #(#fluent_impls)*
        #(#element_fluent_impls)*
    })
}
