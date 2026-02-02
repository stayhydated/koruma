use heck::{ToSnakeCase, ToUpperCamelCase};
use koruma_derive_core::{FieldInfo, ParseFieldResult, ValidatorAttr, parse_field};
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::DeriveInput;

/// Core expansion logic for the `#[derive(KorumaAllFluent)]` derive macro.
///
/// Generates `ToFluentString` implementations for the `{Struct}{Field}KorumaValidator` enums
/// returned by the `all()` method. Each variant delegates to its inner validator's ToFluentString.
#[cfg(feature = "fluent")]
pub fn expand_koruma_all_fluent(input: DeriveInput) -> Result<TokenStream2, syn::Error> {
    let struct_name = &input.ident;

    let fields = match &input.data {
        syn::Data::Struct(data) => &data.fields,
        _ => {
            return Err(syn::Error::new_spanned(
                &input,
                "KorumaAllFluent can only be derived for structs",
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
                .validation
                .field_validators
                .iter()
                .map(|v: &ValidatorAttr| {
                    let variant_name =
                        format_ident!("{}", v.name().to_string().to_upper_camel_case());
                    quote! {
                        #enum_name::#variant_name(v) => v.to_fluent_string()
                    }
                })
                .collect();

            // Add Inner variant arm for newtype fields with additional validators
            let inner_arm = if f.is_newtype() {
                Some(quote! {
                    #enum_name::Inner(inner) => inner.to_fluent_string()
                })
            } else {
                None
            };

            quote! {
                impl ::es_fluent::ToFluentString for #enum_name {
                    fn to_fluent_string(&self) -> String {
                        use ::es_fluent::ToFluentString;
                        match self {
                            #(#match_arms,)*
                            #inner_arm
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
                .validation
                .element_validators
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
                impl ::es_fluent::ToFluentString for #enum_name {
                    fn to_fluent_string(&self) -> String {
                        use ::es_fluent::ToFluentString;
                        match self {
                            #(#match_arms),*
                        }
                    }
                }
            }
        })
        .collect();

    // Generate ToFluentString impls for error structs
    let error_struct_impls: Vec<TokenStream2> = field_infos
        .iter()
        .filter(|f| !f.validation.field_validators.is_empty() || f.is_newtype())
        .map(|f| {
            let field_name = &f.name;
            let error_struct_name = format_ident!(
                "{}{}KorumaValidationError",
                struct_name,
                field_name.to_string().to_upper_camel_case()
            );

            if f.is_newtype() {
                // For newtype fields, delegate to the inner error's to_fluent_string
                quote! {
                    impl ::es_fluent::ToFluentString for #error_struct_name {
                        fn to_fluent_string(&self) -> String {
                            use ::es_fluent::ToFluentString;
                            self.inner().to_fluent_string()
                        }
                    }
                }
            } else {
                // For regular fields, join all validator messages
                let message_pushes: Vec<TokenStream2> = f
                    .validation
                    .field_validators
                    .iter()
                    .map(|v| {
                        let validator_snake =
                            format_ident!("{}", v.name().to_string().to_snake_case());
                        quote! {
                            if let Some(v) = &self.#validator_snake {
                                messages.push(v.to_fluent_string());
                            }
                        }
                    })
                    .collect();

                quote! {
                    impl ::es_fluent::ToFluentString for #error_struct_name {
                        fn to_fluent_string(&self) -> String {
                            use ::es_fluent::ToFluentString;
                            let mut messages = Vec::new();
                            #(#message_pushes)*
                            messages.join("\n")
                        }
                    }
                }
            }
        })
        .collect();

    Ok(quote! {
        #(#fluent_impls)*
        #(#element_fluent_impls)*
        #(#error_struct_impls)*
    })
}
