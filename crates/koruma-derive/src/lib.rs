use proc_macro::TokenStream;
use proc_macro_error2::{abort, proc_macro_error};
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{
    DeriveInput, Field, Fields, Ident, ItemStruct, Token,
    parse::{Parse, ParseStream},
    parse_macro_input,
};

/// Represents a parsed `#[koruma(ValidatorName(arg = value, ...), value)]` attribute
struct KorumaAttr {
    validator: Ident,
    args: Vec<(Ident, syn::Expr)>,
    include_value: bool,
}

impl Parse for KorumaAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let validator: Ident = input.parse()?;

        // Check for skip early
        if validator == "skip" {
            return Ok(KorumaAttr {
                validator,
                args: Vec::new(),
                include_value: false,
            });
        }

        let args = if input.peek(syn::token::Paren) {
            let content;
            syn::parenthesized!(content in input);

            let mut args = Vec::new();
            while !content.is_empty() {
                let name: Ident = content.parse()?;
                content.parse::<Token![=]>()?;
                let value: syn::Expr = content.parse()?;

                args.push((name, value));

                if content.peek(Token![,]) {
                    content.parse::<Token![,]>()?;
                }
            }
            args
        } else {
            Vec::new()
        };

        // Check for ", value" after the validator
        let include_value = if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            let ident: Ident = input.parse()?;
            if ident != "value" {
                return Err(syn::Error::new_spanned(ident, "expected 'value' keyword"));
            }
            true
        } else {
            false
        };

        Ok(KorumaAttr {
            validator,
            args,
            include_value,
        })
    }
}

/// Field info extracted from the struct
struct FieldInfo {
    name: Ident,
    validator: Option<KorumaAttr>,
}

fn parse_field(field: &Field) -> Option<FieldInfo> {
    let name = field.ident.clone()?;

    for attr in &field.attrs {
        if !attr.path().is_ident("koruma") {
            continue;
        }

        // Parse the attribute content
        let parsed: syn::Result<KorumaAttr> = attr.parse_args();

        match parsed {
            Ok(koruma_attr) => {
                // Check for skip
                if koruma_attr.validator == "skip" {
                    return None;
                }
                return Some(FieldInfo {
                    name,
                    validator: Some(koruma_attr),
                });
            },
            Err(e) => {
                abort!(attr, "Failed to parse koruma attribute: {}", e);
            },
        }
    }

    // Field without koruma attribute - skip it
    None
}

/// Attribute macro for validator structs.
///
/// This automatically adds `#[derive(bon::Builder)]` to the struct, making it
/// easy to construct validators with named arguments.
///
/// # Example
///
/// ```ignore
/// #[koruma_validator]
/// #[derive(Debug, Clone, PartialEq, Eq, Hash, EsFluent)]
/// pub struct NumberRangeValidation {
///     min: i32,
///     max: i32,
///     value: Option<i32>,
/// }
/// ```
#[proc_macro_error]
#[proc_macro_attribute]
pub fn validator(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as ItemStruct);

    // Add #[derive(bon::Builder)] to the existing attributes
    let builder_attr: syn::Attribute = syn::parse_quote!(#[derive(bon::Builder)]);
    input.attrs.insert(0, builder_attr);

    let expanded = quote! {
        #input
    };

    TokenStream::from(expanded)
}

/// Derive macro for generating validation error structs and validate methods.
///
/// # Example
///
/// ```ignore
/// #[derive(Koruma)]
/// struct Item {
///     #[koruma(NumberRangeValidation(min = 0, max = 100), value)]
///     age: i32,
///
///     #[koruma(StringLengthValidation(min = 1, max = 50))]
///     name: String,
///
///     #[koruma(skip)]
///     internal_id: u64,
/// }
/// ```
///
/// The `value` keyword after the validator will pass the field's value to the
/// validator via `.value(self.field.clone())`, allowing Fluent messages to
/// include the actual value that failed validation.
///
/// This generates:
/// - `ItemValidationError` struct with `Option<ValidatorType>` for each validated field
/// - Getter methods returning `Option<&ValidatorType>` for each field
/// - `validate(&self) -> Result<(), ItemValidationError>` method on `Item`
#[proc_macro_error]
#[proc_macro_derive(Koruma, attributes(koruma))]
pub fn derive_koruma_validation(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = &input.ident;
    let error_struct_name = format_ident!("{}ValidationError", struct_name);

    let fields = match &input.data {
        syn::Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => abort!(input, "Koruma only supports structs with named fields"),
        },
        _ => abort!(input, "Koruma can only be derived for structs"),
    };

    // Parse all fields and extract validation info
    let field_infos: Vec<FieldInfo> = fields.iter().filter_map(parse_field).collect();

    if field_infos.is_empty() {
        abort!(
            input,
            "Koruma requires at least one field with a #[koruma(...)] attribute"
        );
    }

    // Generate error struct fields
    let error_fields: Vec<TokenStream2> = field_infos
        .iter()
        .map(|f| {
            let name = &f.name;
            let validator = &f.validator.as_ref().unwrap().validator;
            quote! {
                #name: Option<#validator>
            }
        })
        .collect();

    // Generate getter methods for error struct
    let getter_methods: Vec<TokenStream2> = field_infos
        .iter()
        .map(|f| {
            let name = &f.name;
            let validator = &f.validator.as_ref().unwrap().validator;
            quote! {
                pub fn #name(&self) -> Option<&#validator> {
                    self.#name.as_ref()
                }
            }
        })
        .collect();

    // Generate is_empty check (all fields are None)
    let is_empty_checks: Vec<TokenStream2> = field_infos
        .iter()
        .map(|f| {
            let name = &f.name;
            quote! { self.#name.is_none() }
        })
        .collect();

    // Generate default values for error struct initialization
    let error_defaults: Vec<TokenStream2> = field_infos
        .iter()
        .map(|f| {
            let name = &f.name;
            quote! { #name: None }
        })
        .collect();

    // Generate validation logic
    let validation_checks: Vec<TokenStream2> = field_infos
        .iter()
        .map(|f| {
            let name = &f.name;
            let koruma_attr = f.validator.as_ref().unwrap();
            let validator = &koruma_attr.validator;
            let args = &koruma_attr.args;
            let include_value = koruma_attr.include_value;

            let builder_calls: Vec<TokenStream2> = args
                .iter()
                .map(|(arg_name, arg_value)| {
                    quote! { .#arg_name(#arg_value) }
                })
                .collect();

            // Optionally add .value(self.field.clone()) if `value` keyword is present
            let value_call = if include_value {
                quote! { .value(self.#name.clone()) }
            } else {
                quote! {}
            };

            quote! {
                let validator = #validator::builder()
                    #(#builder_calls)*
                    #value_call
                    .build();
                if validator.validate(&self.#name).is_err() {
                    error.#name = Some(validator);
                    has_error = true;
                }
            }
        })
        .collect();

    let expanded = quote! {
        /// Auto-generated validation error struct for [`#struct_name`].
        ///
        /// Each field contains `Some(validator)` if validation failed for that field,
        /// or `None` if validation passed. The validator struct can be used to
        /// generate localized error messages via `ToFluentString`.
        #[derive(Debug, Clone)]
        pub struct #error_struct_name {
            #(#error_fields),*
        }

        impl #error_struct_name {
            #(#getter_methods)*
        }

        impl koruma_core::ValidationError for #error_struct_name {
            fn is_empty(&self) -> bool {
                #(#is_empty_checks)&&*
            }
        }

        impl #struct_name {
            /// Validates all fields and returns an error struct containing
            /// all validation failures.
            ///
            /// Returns `Ok(())` if all validations pass, or `Err(error)` where
            /// `error` contains the validation failures for each field.
            pub fn validate(&self) -> Result<(), #error_struct_name> {
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
    };

    TokenStream::from(expanded)
}
