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

/// Represents a single parsed validator: `ValidatorName(arg = value, ...)` or
/// `ValidatorName<_>(arg = value, ...)`
struct ValidatorAttr {
    validator: Ident,
    /// Whether the validator uses `<_>` syntax for type inference from field type
    infer_type: bool,
    args: Vec<(Ident, syn::Expr)>,
}

impl Parse for ValidatorAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let validator: Ident = input.parse()?;

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

        Ok(ValidatorAttr {
            validator,
            infer_type,
            args,
        })
    }
}

/// Represents a parsed `#[koruma(...)]` attribute which can contain multiple validators
/// separated by commas: `#[koruma(Validator1(a = 1), Validator2(b = 2))]`
/// Can also include `each` modifier for collection validation:
/// `#[koruma(each(Validator1(a = 1)))]`
struct KorumaAttr {
    validators: Vec<ValidatorAttr>,
    is_skip: bool,
    /// Whether to validate each element in a collection
    validate_each: bool,
}

impl Parse for KorumaAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Check for skip
        if input.peek(Ident) {
            let fork = input.fork();
            let ident: Ident = fork.parse()?;
            if ident == "skip" && fork.is_empty() {
                input.parse::<Ident>()?; // consume "skip"
                return Ok(KorumaAttr {
                    validators: Vec::new(),
                    is_skip: true,
                    validate_each: false,
                });
            }
        }

        // Check for each(...) syntax
        let validate_each = if input.peek(Ident) {
            let fork = input.fork();
            let ident: Ident = fork.parse()?;
            if ident == "each" && fork.peek(syn::token::Paren) {
                input.parse::<Ident>()?; // consume "each"
                true
            } else {
                false
            }
        } else {
            false
        };

        // If validate_each, the validators are inside parentheses
        let validators = if validate_each {
            let content;
            syn::parenthesized!(content in input);
            let mut validators = Vec::new();
            while !content.is_empty() {
                validators.push(content.parse::<ValidatorAttr>()?);
                if content.peek(Token![,]) {
                    content.parse::<Token![,]>()?;
                } else {
                    break;
                }
            }
            validators
        } else {
            // Parse comma-separated validators
            let mut validators = Vec::new();
            while !input.is_empty() {
                validators.push(input.parse::<ValidatorAttr>()?);
                if input.peek(Token![,]) {
                    input.parse::<Token![,]>()?;
                } else {
                    break;
                }
            }
            validators
        };

        Ok(KorumaAttr {
            validators,
            is_skip: false,
            validate_each,
        })
    }
}

/// Field info extracted from the struct
struct FieldInfo {
    name: Ident,
    ty: syn::Type,
    validators: Vec<ValidatorAttr>,
    /// Whether to validate each element in a collection
    validate_each: bool,
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
                if koruma_attr.is_skip {
                    return None;
                }
                return Some(FieldInfo {
                    name,
                    ty,
                    validators: koruma_attr.validators,
                    validate_each: koruma_attr.validate_each,
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

/// Extract the inner type T from Vec<T>
fn vec_inner_type(ty: &syn::Type) -> Option<&syn::Type> {
    let syn::Type::Path(type_path) = ty else {
        return None;
    };
    let segment = type_path.path.segments.last()?;

    if segment.ident != "Vec" {
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

    // Helper to generate the type for a validator
    // For validate_each, we use the inner type of Vec<T>
    fn validator_type_for_field(
        v: &ValidatorAttr,
        field_ty: &syn::Type,
        validate_each: bool,
    ) -> TokenStream2 {
        let validator = &v.validator;
        let effective_ty = if validate_each {
            vec_inner_type(field_ty).unwrap_or(field_ty)
        } else {
            field_ty
        };

        if v.infer_type {
            quote! { #validator<#effective_ty> }
        } else {
            quote! { #validator }
        }
    }

    // Generate per-field error structs and collect info for main error struct
    let field_error_structs: Vec<TokenStream2> = field_infos
        .iter()
        .map(|f| {
            let field_name = &f.name;
            let field_ty = &f.ty;
            let validate_each = f.validate_each;
            let field_error_struct_name = format_ident!(
                "{}{}Error",
                struct_name,
                field_name.to_string().to_upper_camel_case()
            );

            // Generate fields for each validator
            let validator_fields: Vec<TokenStream2> = f
                .validators
                .iter()
                .map(|v| {
                    let validator_snake =
                        format_ident!("{}", v.validator.to_string().to_snake_case());
                    let vtype = validator_type_for_field(v, field_ty, validate_each);
                    quote! { #validator_snake: Option<#vtype> }
                })
                .collect();

            // Generate getter methods for each validator
            let validator_getters: Vec<TokenStream2> = f
                .validators
                .iter()
                .map(|v| {
                    let validator_snake =
                        format_ident!("{}", v.validator.to_string().to_snake_case());
                    let vtype = validator_type_for_field(v, field_ty, validate_each);
                    quote! {
                        pub fn #validator_snake(&self) -> Option<&#vtype> {
                            self.#validator_snake.as_ref()
                        }
                    }
                })
                .collect();

            // Generate is_empty checks for this field's error struct
            let is_empty_checks: Vec<TokenStream2> = f
                .validators
                .iter()
                .map(|v| {
                    let validator_snake =
                        format_ident!("{}", v.validator.to_string().to_snake_case());
                    quote! { self.#validator_snake.is_none() }
                })
                .collect();

            // Generate enum variants for the all() method
            let enum_name = format_ident!(
                "{}{}Validator",
                struct_name,
                field_name.to_string().to_upper_camel_case()
            );

            let enum_variants: Vec<TokenStream2> = f
                .validators
                .iter()
                .map(|v| {
                    let variant_name =
                        format_ident!("{}", v.validator.to_string().to_upper_camel_case());
                    let vtype = validator_type_for_field(v, field_ty, validate_each);
                    quote! { #variant_name(#vtype) }
                })
                .collect();

            // Generate the all() method body - collect all Some validators into the enum
            let all_pushes: Vec<TokenStream2> = f
                .validators
                .iter()
                .map(|v| {
                    let validator_snake =
                        format_ident!("{}", v.validator.to_string().to_snake_case());
                    let variant_name =
                        format_ident!("{}", v.validator.to_string().to_upper_camel_case());
                    quote! {
                        if let Some(v) = &self.#validator_snake {
                            result.push(#enum_name::#variant_name(v.clone()));
                        }
                    }
                })
                .collect();

            quote! {
                /// Enum of all possible validators for this field.
                #[derive(Clone, Debug)]
                pub enum #enum_name {
                    #(#enum_variants),*
                }

                #[derive(Clone, Debug)]
                pub struct #field_error_struct_name {
                    #(#validator_fields),*
                }

                impl #field_error_struct_name {
                    #(#validator_getters)*

                    /// Returns all failed validators for this field.
                    pub fn all(&self) -> Vec<#enum_name> {
                        let mut result = Vec::new();
                        #(#all_pushes)*
                        result
                    }

                    pub fn is_empty(&self) -> bool {
                        #(#is_empty_checks)&&*
                    }

                    pub fn has_errors(&self) -> bool {
                        !self.is_empty()
                    }
                }
            }
        })
        .collect();

    // Generate main error struct fields (one per validated field)
    let error_fields: Vec<TokenStream2> = field_infos
        .iter()
        .map(|f| {
            let field_name = &f.name;
            let field_error_struct_name = format_ident!(
                "{}{}Error",
                struct_name,
                field_name.to_string().to_upper_camel_case()
            );
            if f.validate_each {
                // For validate_each, store a Vec of (index, error) tuples
                quote! { #field_name: Vec<(usize, #field_error_struct_name)> }
            } else {
                quote! { #field_name: #field_error_struct_name }
            }
        })
        .collect();

    // Generate getter methods for main error struct
    let getter_methods: Vec<TokenStream2> = field_infos
        .iter()
        .map(|f| {
            let field_name = &f.name;
            let field_error_struct_name = format_ident!(
                "{}{}Error",
                struct_name,
                field_name.to_string().to_upper_camel_case()
            );
            if f.validate_each {
                quote! {
                    /// Returns all validation errors for this collection field, with their indices.
                    pub fn #field_name(&self) -> &[(usize, #field_error_struct_name)] {
                        &self.#field_name
                    }
                }
            } else {
                quote! {
                    pub fn #field_name(&self) -> &#field_error_struct_name {
                        &self.#field_name
                    }
                }
            }
        })
        .collect();

    // Generate is_empty check (all field error structs are empty)
    let is_empty_checks: Vec<TokenStream2> = field_infos
        .iter()
        .map(|f| {
            let field_name = &f.name;
            if f.validate_each {
                quote! { self.#field_name.is_empty() }
            } else {
                quote! { self.#field_name.is_empty() }
            }
        })
        .collect();

    // Generate default values for main error struct initialization
    let error_defaults: Vec<TokenStream2> = field_infos
        .iter()
        .map(|f| {
            let field_name = &f.name;
            let field_error_struct_name = format_ident!(
                "{}{}Error",
                struct_name,
                field_name.to_string().to_upper_camel_case()
            );

            if f.validate_each {
                // For validate_each, start with an empty Vec
                quote! { #field_name: Vec::new() }
            } else {
                let validator_defaults: Vec<TokenStream2> = f
                    .validators
                    .iter()
                    .map(|v| {
                        let validator_snake =
                            format_ident!("{}", v.validator.to_string().to_snake_case());
                        quote! { #validator_snake: None }
                    })
                    .collect();

                quote! {
                    #field_name: #field_error_struct_name {
                        #(#validator_defaults),*
                    }
                }
            }
        })
        .collect();

    // Generate validation logic - always use .with_value()
    let validation_checks: Vec<TokenStream2> = field_infos
        .iter()
        .map(|f| {
            let field_name = &f.name;
            let field_ty = &f.ty;
            let validate_each = f.validate_each;

            let field_error_struct_name = format_ident!(
                "{}{}Error",
                struct_name,
                field_name.to_string().to_upper_camel_case()
            );

            if validate_each {
                // For validate_each, iterate over the collection
                let element_ty = vec_inner_type(field_ty).unwrap_or(field_ty);

                let validator_checks: Vec<TokenStream2> = f.validators.iter().map(|v| {
                    let validator = &v.validator;
                    let validator_snake = format_ident!("{}", validator.to_string().to_snake_case());

                    let builder_calls: Vec<TokenStream2> = v
                        .args
                        .iter()
                        .map(|(arg_name, arg_value)| {
                            quote! { .#arg_name(#arg_value) }
                        })
                        .collect();

                    if v.infer_type {
                        let assert_fn = format_ident!(
                            "__koruma_assert_validate_{}_{}",
                            field_name,
                            validator_snake
                        );
                        quote! {
                            fn #assert_fn<V: koruma::Validate<T>, T>(v: &V, t: &T) -> Result<(), ()> {
                                v.validate(t)
                            }
                            let validator = #validator::<#element_ty>::builder()
                                #(#builder_calls)*
                                .with_value(item.clone())
                                .build();
                            if #assert_fn(&validator, item).is_err() {
                                element_error.#validator_snake = Some(validator);
                                element_has_error = true;
                            }
                        }
                    } else {
                        quote! {
                            let validator = #validator::builder()
                                #(#builder_calls)*
                                .with_value(item.clone())
                                .build();
                            if validator.validate(item).is_err() {
                                element_error.#validator_snake = Some(validator);
                                element_has_error = true;
                            }
                        }
                    }
                }).collect();

                let validator_defaults: Vec<TokenStream2> = f
                    .validators
                    .iter()
                    .map(|v| {
                        let validator_snake =
                            format_ident!("{}", v.validator.to_string().to_snake_case());
                        quote! { #validator_snake: None }
                    })
                    .collect();

                quote! {
                    for (idx, item) in self.#field_name.iter().enumerate() {
                        let mut element_error = #field_error_struct_name {
                            #(#validator_defaults),*
                        };
                        let mut element_has_error = false;

                        #(#validator_checks)*

                        if element_has_error {
                            error.#field_name.push((idx, element_error));
                            has_error = true;
                        }
                    }
                }
            } else {
                // Regular field validation
                let validator_checks: Vec<TokenStream2> = f.validators.iter().map(|v| {
                    let validator = &v.validator;
                    let validator_snake = format_ident!("{}", validator.to_string().to_snake_case());

                    let builder_calls: Vec<TokenStream2> = v
                        .args
                        .iter()
                        .map(|(arg_name, arg_value)| {
                            quote! { .#arg_name(#arg_value) }
                        })
                        .collect();

                    if v.infer_type {
                        let assert_fn = format_ident!(
                            "__koruma_assert_validate_{}_{}",
                            field_name,
                            validator_snake
                        );
                        quote! {
                            fn #assert_fn<V: koruma::Validate<T>, T>(v: &V, t: &T) -> Result<(), ()> {
                                v.validate(t)
                            }
                            let validator = #validator::<#field_ty>::builder()
                                #(#builder_calls)*
                                .with_value(self.#field_name.clone())
                                .build();
                            if #assert_fn(&validator, &self.#field_name).is_err() {
                                error.#field_name.#validator_snake = Some(validator);
                                has_error = true;
                            }
                        }
                    } else {
                        quote! {
                            let validator = #validator::builder()
                                #(#builder_calls)*
                                .with_value(self.#field_name.clone())
                                .build();
                            if validator.validate(&self.#field_name).is_err() {
                                error.#field_name.#validator_snake = Some(validator);
                                has_error = true;
                            }
                        }
                    }
                }).collect();

                quote! {
                    #(#validator_checks)*
                }
            }
        })
        .collect();

    let expanded = quote! {
        // Per-field error structs
        #(#field_error_structs)*

        /// Auto-generated validation error struct for [`#struct_name`].
        ///
        /// Each field contains a nested error struct with `Option<Validator>` for each
        /// validator. Access errors via chained calls like `error.field().validator()`.
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
