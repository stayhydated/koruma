use heck::{ToSnakeCase, ToUpperCamelCase};
use proc_macro::TokenStream;
use proc_macro_error2::{abort, proc_macro_error};
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{
    DeriveInput, Field, Fields, Ident, ItemStruct, Token,
    parse::{Parse, ParseStream},
    parse_macro_input,
};

/// Represents a parsed `#[koruma(ValidatorName(arg = value, ...))]` or
/// `#[koruma(ValidatorName<_>(arg = value, ...))]` attribute
struct KorumaAttr {
    validator: Ident,
    /// Whether the validator uses `<_>` syntax for type inference from field type
    infer_type: bool,
    args: Vec<(Ident, syn::Expr)>,
}

impl Parse for KorumaAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let validator: Ident = input.parse()?;

        // Check for skip early
        if validator == "skip" {
            return Ok(KorumaAttr {
                validator,
                infer_type: false,
                args: Vec::new(),
            });
        }

        // Check for <_> type inference syntax
        let infer_type = if input.peek(Token![<]) {
            input.parse::<Token![<]>()?;
            input.parse::<Token![_]>()?;
            input.parse::<Token![>]>()?;
            true
        } else {
            false
        };

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

        Ok(KorumaAttr {
            validator,
            infer_type,
            args,
        })
    }
}

/// Field info extracted from the struct
struct FieldInfo {
    name: Ident,
    ty: syn::Type,
    validator: Option<KorumaAttr>,
}

fn parse_field(field: &Field) -> Option<FieldInfo> {
    let name = field.ident.clone()?;
    let ty = field.ty.clone();

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
                    ty,
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

/// Find the field marked with #[koruma(value)] and return its name and type
fn find_value_field(input: &ItemStruct) -> Option<(Ident, syn::Type)> {
    if let Fields::Named(ref fields) = input.fields {
        for field in &fields.named {
            for attr in &field.attrs {
                if attr.path().is_ident("koruma") {
                    // Try to parse as just "value"
                    if let Ok(ident) = attr.parse_args::<Ident>() {
                        if ident == "value" {
                            return Some((field.ident.clone().unwrap(), field.ty.clone()));
                        }
                    }
                }
            }
        }
    }
    None
}

/// Attribute macro for validator structs.
///
/// This automatically:
/// - Adds `#[derive(bon::Builder)]` to the struct
/// - Generates a `with_value` method on the builder that delegates to the field
///   marked with `#[koruma(value)]`
/// - For generic validators, generates an `impl_<validator_name>!` macro for easy
///   implementation of the `Validate` trait for multiple types
///
/// # Example (non-generic)
///
/// ```ignore
/// #[koruma::validator]
/// #[derive(Clone, Debug, EsFluent)]
/// pub struct NumberRangeValidation {
///     min: i32,
///     max: i32,
///     #[koruma(value)]
///     actual: Option<i32>,
/// }
/// ```
///
/// # Example (generic)
///
/// ```ignore
/// #[koruma::validator]
/// #[derive(Clone, Debug, EsFluent)]
/// pub struct NumberRangeValidation<T> {
///     min: T,
///     max: T,
///     #[koruma(value)]
///     actual: Option<T>,
/// }
///
/// // Use the generated macro to implement Validate for multiple types:
/// impl_number_range_validation!(i32, i64, u32, u64, f32, f64);
/// ```
#[proc_macro_error]
#[proc_macro_attribute]
pub fn validator(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Ensure no arguments
    if !attr.is_empty() {
        let attr2 = proc_macro2::TokenStream::from(attr);
        abort!(attr2, "koruma::validator does not accept arguments");
    }

    let mut input = parse_macro_input!(item as ItemStruct);
    let struct_name = &input.ident;
    let builder_name = format_ident!("{}Builder", struct_name);

    // Check if the struct has generics
    let has_generics = !input.generics.params.is_empty();

    // Find the field marked with #[koruma(value)]
    let (value_field_name, value_field_type) = match find_value_field(&input) {
        Some(v) => v,
        None => {
            abort!(
                input,
                "koruma::validator requires a field marked with #[koruma(value)].\n\
                 Example:\n\
                 #[koruma(value)]\n\
                 actual: Option<i32>"
            );
        },
    };

    // Extract the inner type from Option<T>
    let inner_type = option_inner_type(&value_field_type).unwrap_or(&value_field_type);

    // Add #[derive(bon::Builder)] to the existing attributes
    let builder_attr: syn::Attribute = syn::parse_quote!(#[derive(koruma::bon::Builder)]);
    input.attrs.insert(0, builder_attr);

    // Remove #[koruma(value)] from the field so bon doesn't see it
    if let Fields::Named(ref mut fields) = input.fields {
        for field in &mut fields.named {
            field.attrs.retain(|attr| {
                if attr.path().is_ident("koruma") {
                    if let Ok(ident) = attr.parse_args::<Ident>() {
                        return ident != "value";
                    }
                }
                true
            });
        }
    }

    // Generate the module name that bon creates (snake_case of struct name + _builder)
    let module_name = format_ident!("{}_builder", struct_name.to_string().to_snake_case());

    // Generate the associated type name (PascalCase of field name) and Set wrapper
    let value_pascal = value_field_name.to_string().to_upper_camel_case();
    let value_assoc_type = format_ident!("{}", value_pascal);
    let set_value_type = format_ident!("Set{}", value_pascal);

    // Generate the impl macro for generic validators
    let impl_macro = if has_generics {
        let macro_name = format_ident!("impl_{}", struct_name.to_string().to_snake_case());
        quote! {
            /// Auto-generated macro for implementing `Validate` for multiple types.
            ///
            /// # Example
            /// ```ignore
            /// #macro_name!(i32, i64, u32, u64);
            /// ```
            #[macro_export]
            macro_rules! #macro_name {
                ($($t:ty),+ $(,)?) => {
                    $(
                        impl koruma::Validate<$t> for #struct_name<$t>
                        where
                            $t: PartialOrd + Clone,
                        {
                            fn validate(&self, value: &$t) -> Result<(), ()> {
                                if *value < self.min || *value > self.max {
                                    Err(())
                                } else {
                                    Ok(())
                                }
                            }
                        }
                    )+
                };
            }
        }
    } else {
        quote! {}
    };

    let with_value_impl = if has_generics {
        // For generic validators, the builder is Builder<T, S> (type param first, then state)
        quote! {
            impl<T, S: #module_name::State> #builder_name<T, S>
            where
                S::#value_assoc_type: koruma::bon::IsUnset,
            {
                /// Sets the value field. This is auto-generated by `#[koruma::validator]`.
                pub fn with_value(self, value: T) -> #builder_name<T, #module_name::#set_value_type<S>> {
                    self.#value_field_name(value)
                }
            }
        }
    } else {
        quote! {
            impl<S: #module_name::State> #builder_name<S>
            where
                S::#value_assoc_type: koruma::bon::IsUnset,
            {
                /// Sets the value field. This is auto-generated by `#[koruma::validator]`.
                pub fn with_value(self, value: #inner_type) -> #builder_name<#module_name::#set_value_type<S>> {
                    self.#value_field_name(value)
                }
            }
        }
    };

    let expanded = quote! {
        #input

        #with_value_impl

        #impl_macro
    };

    TokenStream::from(expanded)
}

/// Extract the inner type T from Option<T>
fn option_inner_type(ty: &syn::Type) -> Option<&syn::Type> {
    let syn::Type::Path(type_path) = ty else {
        return None;
    };
    let segment = type_path.path.segments.last()?;

    if segment.ident != "Option" {
        return None;
    }

    let syn::PathArguments::AngleBracketed(args) = &segment.arguments else {
        return None;
    };

    match args.args.first()? {
        syn::GenericArgument::Type(inner) => Some(inner),
        _ => None,
    }
}

/// Derive macro for generating validation error structs and validate methods.
///
/// # Example
///
/// ```ignore
/// #[derive(Koruma)]
/// struct Item {
///     #[koruma(NumberRangeValidation(min = 0, max = 100))]
///     age: i32,
///
///     #[koruma(StringLengthValidation(min = 1, max = 50))]
///     name: String,
///
///     // No #[koruma(...)] attribute means field is not validated
///     internal_id: u64,
/// }
/// ```
///
/// This generates:
/// - `ItemValidationError` struct with `Option<ValidatorType>` for each validated field
/// - Getter methods returning `Option<&ValidatorType>` for each field
/// - `validate(&self) -> Result<(), ItemValidationError>` method on `Item`
///
/// The macro always generates `.with_value(self.field.clone())` for validators.
#[proc_macro_error]
#[proc_macro_derive(Koruma, attributes(koruma))]
pub fn derive_koruma(input: TokenStream) -> TokenStream {
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
    // For generic validators (marked with <_>), we use the field type as the type parameter
    let error_fields: Vec<TokenStream2> = field_infos
        .iter()
        .map(|f| {
            let name = &f.name;
            let koruma_attr = f.validator.as_ref().unwrap();
            let validator = &koruma_attr.validator;
            let field_ty = &f.ty;

            if koruma_attr.infer_type {
                // Generic validator: ValidatorName<FieldType>
                quote! {
                    #name: Option<#validator<#field_ty>>
                }
            } else {
                // Non-generic validator: ValidatorName
                quote! {
                    #name: Option<#validator>
                }
            }
        })
        .collect();

    // Generate getter methods for error struct
    let getter_methods: Vec<TokenStream2> = field_infos
        .iter()
        .map(|f| {
            let name = &f.name;
            let koruma_attr = f.validator.as_ref().unwrap();
            let validator = &koruma_attr.validator;
            let field_ty = &f.ty;

            if koruma_attr.infer_type {
                quote! {
                    pub fn #name(&self) -> Option<&#validator<#field_ty>> {
                        self.#name.as_ref()
                    }
                }
            } else {
                quote! {
                    pub fn #name(&self) -> Option<&#validator> {
                        self.#name.as_ref()
                    }
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

    // Generate validation logic - always use .with_value()
    let validation_checks: Vec<TokenStream2> = field_infos
        .iter()
        .map(|f| {
            let name = &f.name;
            let field_ty = &f.ty;
            let koruma_attr = f.validator.as_ref().unwrap();
            let validator = &koruma_attr.validator;
            let args = &koruma_attr.args;

            let builder_calls: Vec<TokenStream2> = args
                .iter()
                .map(|(arg_name, arg_value)| {
                    quote! { .#arg_name(#arg_value) }
                })
                .collect();

            if koruma_attr.infer_type {
                // Generic validator: ValidatorName::<FieldType>::builder()
                // We use a helper function to ensure Validate<T> is implemented,
                // giving a clearer error message if not
                let assert_fn = format_ident!("__koruma_assert_validate_{}", name);
                quote! {
                    fn #assert_fn<V: koruma::Validate<T>, T>(v: &V, t: &T) -> Result<(), ()> {
                        v.validate(t)
                    }
                    let validator = #validator::<#field_ty>::builder()
                        #(#builder_calls)*
                        .with_value(self.#name.clone())
                        .build();
                    if #assert_fn(&validator, &self.#name).is_err() {
                        error.#name = Some(validator);
                        has_error = true;
                    }
                }
            } else {
                // Non-generic validator: ValidatorName::builder()
                quote! {
                    let validator = #validator::builder()
                        #(#builder_calls)*
                        .with_value(self.#name.clone())
                        .build();
                    if validator.validate(&self.#name).is_err() {
                        error.#name = Some(validator);
                        has_error = true;
                    }
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
        #[derive(Clone, Debug)]
        pub struct #error_struct_name {
            #(#error_fields),*
        }

        impl #error_struct_name {
            #(#getter_methods)*
        }

        impl koruma::ValidationError for #error_struct_name {
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
