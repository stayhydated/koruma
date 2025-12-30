//! Core expansion logic for koruma derive macros.
//!
//! This module contains the actual TokenStream generation that can be tested.

use heck::{ToSnakeCase, ToUpperCamelCase};
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{DeriveInput, Field, Fields, Ident, ItemStruct, Token, parse::Parse, parse::ParseStream};

/// Represents a single parsed validator: `ValidatorName(arg = value, ...)` or
/// `ValidatorName<_>(arg = value, ...)`
pub(crate) struct ValidatorAttr {
    pub validator: Ident,
    /// Whether the validator uses `<_>` syntax for type inference from field type
    pub infer_type: bool,
    pub args: Vec<(Ident, syn::Expr)>,
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
pub(crate) struct KorumaAttr {
    pub validators: Vec<ValidatorAttr>,
    pub is_skip: bool,
    /// Whether to validate each element in a collection
    pub validate_each: bool,
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
pub(crate) struct FieldInfo {
    pub name: Ident,
    pub ty: syn::Type,
    pub validators: Vec<ValidatorAttr>,
    /// Whether to validate each element in a collection
    pub validate_each: bool,
}

pub(crate) fn parse_field(field: &Field) -> Option<FieldInfo> {
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
            }
            Err(_) => {
                return None;
            }
        }
    }

    // Field without koruma attribute - skip it
    None
}

/// Find the field marked with #[koruma(value)] and return its name and type
pub(crate) fn find_value_field(input: &ItemStruct) -> Option<(Ident, syn::Type)> {
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

/// Extract the inner type T from Option<T>
pub(crate) fn option_inner_type(ty: &syn::Type) -> Option<&syn::Type> {
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
pub(crate) fn vec_inner_type(ty: &syn::Type) -> Option<&syn::Type> {
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

/// Core expansion logic for the `#[validator]` attribute macro.
///
/// Takes a parsed struct and returns the expanded TokenStream.
pub fn expand_validator(mut input: ItemStruct) -> Result<TokenStream2, syn::Error> {
    let struct_name = &input.ident;
    let builder_name = format_ident!("{}Builder", struct_name);

    // Check if the struct has generics
    let has_generics = !input.generics.params.is_empty();

    // Find the field marked with #[koruma(value)]
    let (value_field_name, value_field_type) = find_value_field(&input).ok_or_else(|| {
        syn::Error::new_spanned(
            &input,
            "koruma::validator requires a field marked with #[koruma(value)].\n\
             Example:\n\
             #[koruma(value)]\n\
             actual: Option<i32>",
        )
    })?;

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

    Ok(quote! {
        #input

        #with_value_impl

        #impl_macro
    })
}

/// Helper to generate the type for a validator
/// 
/// This unwraps Option<T> and Vec<T> to get the effective type being validated.
/// - For `each` validation on `Vec<T>`: uses T
/// - For optional fields `Option<T>`: uses T (validation is skipped if None)
/// - For regular fields: uses the field type directly
fn validator_type_for_field(
    v: &ValidatorAttr,
    field_ty: &syn::Type,
    validate_each: bool,
) -> TokenStream2 {
    let validator = &v.validator;
    
    // Unwrap Vec<T> for each validation
    let after_vec = if validate_each {
        vec_inner_type(field_ty).unwrap_or(field_ty)
    } else {
        field_ty
    };
    
    // Unwrap Option<T> for optional field validation
    let effective_ty = option_inner_type(after_vec).unwrap_or(after_vec);

    if v.infer_type {
        quote! { #validator<#effective_ty> }
    } else {
        quote! { #validator }
    }
}

/// Get the effective type for validation (unwrapping Option and Vec as needed)
fn effective_validation_type<'a>(
    field_ty: &'a syn::Type,
    validate_each: bool,
) -> &'a syn::Type {
    // Unwrap Vec<T> for each validation
    let after_vec = if validate_each {
        vec_inner_type(field_ty).unwrap_or(field_ty)
    } else {
        field_ty
    };
    
    // Unwrap Option<T> for optional field validation
    option_inner_type(after_vec).unwrap_or(after_vec)
}

/// Check if a field type is Option<T>
fn is_option_type(ty: &syn::Type) -> bool {
    option_inner_type(ty).is_some()
}

/// Core expansion logic for the `#[derive(Koruma)]` derive macro.
///
/// Takes a parsed DeriveInput and returns the expanded TokenStream.
pub fn expand_koruma(input: DeriveInput) -> Result<TokenStream2, syn::Error> {
    let struct_name = &input.ident;
    let error_struct_name = format_ident!("{}ValidationError", struct_name);

    let fields = match &input.data {
        syn::Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => {
                return Err(syn::Error::new_spanned(
                    &input,
                    "Koruma only supports structs with named fields",
                ))
            }
        },
        _ => {
            return Err(syn::Error::new_spanned(
                &input,
                "Koruma can only be derived for structs",
            ))
        }
    };

    // Parse all fields and extract validation info
    let field_infos: Vec<FieldInfo> = fields.iter().filter_map(parse_field).collect();

    if field_infos.is_empty() {
        return Err(syn::Error::new_spanned(
            &input,
            "Koruma requires at least one field with a #[koruma(...)] attribute",
        ));
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
                let element_is_optional = is_option_type(element_ty);
                let effective_element_ty = effective_validation_type(field_ty, true);

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
                            let validator = #validator::<#effective_element_ty>::builder()
                                #(#builder_calls)*
                                .with_value(__item_value.clone())
                                .build();
                            if #assert_fn(&validator, __item_value).is_err() {
                                element_error.#validator_snake = Some(validator);
                                element_has_error = true;
                            }
                        }
                    } else {
                        quote! {
                            let validator = #validator::builder()
                                #(#builder_calls)*
                                .with_value(__item_value.clone())
                                .build();
                            if validator.validate(__item_value).is_err() {
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

                let inner_validation = quote! {
                    let mut element_error = #field_error_struct_name {
                        #(#validator_defaults),*
                    };
                    let mut element_has_error = false;

                    #(#validator_checks)*

                    if element_has_error {
                        error.#field_name.push((idx, element_error));
                        has_error = true;
                    }
                };

                if element_is_optional {
                    // For Vec<Option<T>>, skip None items
                    quote! {
                        for (idx, item) in self.#field_name.iter().enumerate() {
                            if let Some(ref __item_value) = item {
                                #inner_validation
                            }
                        }
                    }
                } else {
                    // For Vec<T>, validate each item directly
                    quote! {
                        for (idx, __item_value) in self.#field_name.iter().enumerate() {
                            #inner_validation
                        }
                    }
                }
            } else {
                // Regular field validation
                let field_is_optional = is_option_type(field_ty);
                let effective_ty = effective_validation_type(field_ty, false);
                
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
                            let validator = #validator::<#effective_ty>::builder()
                                #(#builder_calls)*
                                .with_value(__value.clone())
                                .build();
                            if #assert_fn(&validator, __value).is_err() {
                                error.#field_name.#validator_snake = Some(validator);
                                has_error = true;
                            }
                        }
                    } else {
                        quote! {
                            let validator = #validator::builder()
                                #(#builder_calls)*
                                .with_value(__value.clone())
                                .build();
                            if validator.validate(__value).is_err() {
                                error.#field_name.#validator_snake = Some(validator);
                                has_error = true;
                            }
                        }
                    }
                }).collect();

                if field_is_optional {
                    // For Option<T> fields, skip validation if None
                    quote! {
                        if let Some(ref __value) = self.#field_name {
                            #(#validator_checks)*
                        }
                    }
                } else {
                    // For regular fields, validate directly
                    quote! {
                        let __value = &self.#field_name;
                        #(#validator_checks)*
                    }
                }
            }
        })
        .collect();

    Ok(quote! {
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
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_snapshot;

    /// Helper to format TokenStream as pretty-printed Rust code
    fn pretty_print(tokens: TokenStream2) -> String {
        let file = syn::parse_file(&tokens.to_string()).unwrap();
        prettyplease::unparse(&file)
    }

    #[test]
    fn test_validator_expansion_simple() {
        let input: ItemStruct = syn::parse_quote! {
            #[derive(Clone, Debug)]
            pub struct NumberRangeValidation {
                min: i32,
                max: i32,
                #[koruma(value)]
                pub actual: Option<i32>,
            }
        };

        let expanded = expand_validator(input).unwrap();
        assert_snapshot!(pretty_print(expanded));
    }

    #[test]
    fn test_validator_expansion_generic() {
        let input: ItemStruct = syn::parse_quote! {
            #[derive(Clone, Debug)]
            pub struct GenericRangeValidation<T> {
                pub min: T,
                pub max: T,
                #[koruma(value)]
                pub actual: Option<T>,
            }
        };

        let expanded = expand_validator(input).unwrap();
        assert_snapshot!(pretty_print(expanded));
    }

    #[test]
    fn test_koruma_expansion_single_validator() {
        let input: DeriveInput = syn::parse_quote! {
            pub struct Item {
                #[koruma(NumberRangeValidation(min = 0, max = 100))]
                pub age: i32,
            }
        };

        let expanded = expand_koruma(input).unwrap();
        assert_snapshot!(pretty_print(expanded));
    }

    #[test]
    fn test_koruma_expansion_multiple_validators() {
        let input: DeriveInput = syn::parse_quote! {
            pub struct MultiValidatorItem {
                #[koruma(NumberRangeValidation(min = 0, max = 100), EvenNumberValidation)]
                pub value: i32,
            }
        };

        let expanded = expand_koruma(input).unwrap();
        assert_snapshot!(pretty_print(expanded));
    }

    #[test]
    fn test_koruma_expansion_generic_validator() {
        let input: DeriveInput = syn::parse_quote! {
            pub struct GenericItem {
                #[koruma(GenericRangeValidation<_>(min = 0.0, max = 100.0))]
                pub score: f64,
            }
        };

        let expanded = expand_koruma(input).unwrap();
        assert_snapshot!(pretty_print(expanded));
    }

    #[test]
    fn test_koruma_expansion_each() {
        let input: DeriveInput = syn::parse_quote! {
            pub struct Order {
                #[koruma(each(GenericRangeValidation<_>(min = 0.0, max = 100.0)))]
                pub scores: Vec<f64>,
            }
        };

        let expanded = expand_koruma(input).unwrap();
        assert_snapshot!(pretty_print(expanded));
    }

    #[test]
    fn test_koruma_expansion_multiple_fields() {
        let input: DeriveInput = syn::parse_quote! {
            pub struct Item {
                #[koruma(NumberRangeValidation(min = 0, max = 100))]
                pub age: i32,

                #[koruma(StringLengthValidation(min = 1, max = 67))]
                pub name: String,
            }
        };

        let expanded = expand_koruma(input).unwrap();
        assert_snapshot!(pretty_print(expanded));
    }

    // ============================================================================
    // Additional Snapshot Tests - Edge Cases
    // ============================================================================

    #[test]
    fn test_validator_expansion_non_option_value() {
        // Value field that is NOT wrapped in Option
        let input: ItemStruct = syn::parse_quote! {
            #[derive(Clone, Debug)]
            pub struct DirectValueValidation {
                min: i32,
                #[koruma(value)]
                pub actual: i32,
            }
        };

        let expanded = expand_validator(input).unwrap();
        assert_snapshot!(pretty_print(expanded));
    }

    #[test]
    fn test_koruma_expansion_validator_no_args() {
        // Validator with no arguments (like EvenNumberValidation)
        let input: DeriveInput = syn::parse_quote! {
            pub struct Item {
                #[koruma(EvenNumberValidation)]
                pub value: i32,
            }
        };

        let expanded = expand_koruma(input).unwrap();
        assert_snapshot!(pretty_print(expanded));
    }

    #[test]
    fn test_koruma_expansion_each_multiple_validators() {
        // each() with multiple validators
        let input: DeriveInput = syn::parse_quote! {
            pub struct Order {
                #[koruma(each(RangeValidation(min = 0, max = 100), EvenValidation))]
                pub values: Vec<i32>,
            }
        };

        let expanded = expand_koruma(input).unwrap();
        assert_snapshot!(pretty_print(expanded));
    }

    #[test]
    fn test_koruma_expansion_mixed_fields() {
        // Mix of regular field, each field, and multiple validators
        let input: DeriveInput = syn::parse_quote! {
            pub struct ComplexItem {
                #[koruma(RangeValidation(min = 0, max = 100))]
                pub age: i32,

                #[koruma(each(LengthValidation(min = 1, max = 50)))]
                pub tags: Vec<String>,

                #[koruma(RangeValidation(min = 0, max = 10), EvenValidation)]
                pub rating: i32,
            }
        };

        let expanded = expand_koruma(input).unwrap();
        assert_snapshot!(pretty_print(expanded));
    }

    #[test]
    fn test_koruma_expansion_optional_field() {
        // Optional field should generate if-let pattern and skip validation when None
        let input: DeriveInput = syn::parse_quote! {
            pub struct UserProfile {
                #[koruma(StringLengthValidation(min = 1, max = 50))]
                pub username: String,

                #[koruma(StringLengthValidation(min = 1, max = 200))]
                pub bio: Option<String>,
            }
        };

        let expanded = expand_koruma(input).unwrap();
        assert_snapshot!(pretty_print(expanded));
    }

    #[test]
    fn test_koruma_expansion_optional_with_generic() {
        // Optional field with generic validator
        let input: DeriveInput = syn::parse_quote! {
            pub struct Item {
                #[koruma(GenericRange<_>(min = 0, max = 100))]
                pub score: Option<i32>,
            }
        };

        let expanded = expand_koruma(input).unwrap();
        assert_snapshot!(pretty_print(expanded));
    }

    // ============================================================================
    // Unit Tests for Helper Functions
    // ============================================================================

    #[test]
    fn test_option_inner_type_extracts_inner() {
        let ty: syn::Type = syn::parse_quote!(Option<i32>);
        let inner = option_inner_type(&ty);
        assert!(inner.is_some());
        let inner_str = quote!(#inner).to_string();
        assert!(inner_str.contains("i32"), "Expected i32, got: {}", inner_str);
    }

    #[test]
    fn test_option_inner_type_nested() {
        let ty: syn::Type = syn::parse_quote!(Option<Vec<String>>);
        let inner = option_inner_type(&ty);
        assert!(inner.is_some());
        let inner_str = quote!(#inner).to_string();
        assert!(inner_str.contains("Vec"), "Expected Vec<String>, got: {}", inner_str);
    }

    #[test]
    fn test_option_inner_type_returns_none_for_non_option() {
        let ty: syn::Type = syn::parse_quote!(i32);
        assert!(option_inner_type(&ty).is_none());

        let ty: syn::Type = syn::parse_quote!(Vec<i32>);
        assert!(option_inner_type(&ty).is_none());

        let ty: syn::Type = syn::parse_quote!(String);
        assert!(option_inner_type(&ty).is_none());
    }

    #[test]
    fn test_vec_inner_type_extracts_inner() {
        let ty: syn::Type = syn::parse_quote!(Vec<f64>);
        let inner = vec_inner_type(&ty);
        assert!(inner.is_some());
        let inner_str = quote!(#inner).to_string();
        assert!(inner_str.contains("f64"), "Expected f64, got: {}", inner_str);
    }

    #[test]
    fn test_vec_inner_type_complex() {
        let ty: syn::Type = syn::parse_quote!(Vec<Option<String>>);
        let inner = vec_inner_type(&ty);
        assert!(inner.is_some());
        let inner_str = quote!(#inner).to_string();
        assert!(inner_str.contains("Option"), "Expected Option<String>, got: {}", inner_str);
    }

    #[test]
    fn test_vec_inner_type_returns_none_for_non_vec() {
        let ty: syn::Type = syn::parse_quote!(i32);
        assert!(vec_inner_type(&ty).is_none());

        let ty: syn::Type = syn::parse_quote!(Option<i32>);
        assert!(vec_inner_type(&ty).is_none());

        let ty: syn::Type = syn::parse_quote!(HashMap<String, i32>);
        assert!(vec_inner_type(&ty).is_none());
    }

    #[test]
    fn test_find_value_field_finds_marked_field() {
        let input: ItemStruct = syn::parse_quote! {
            pub struct Test {
                min: i32,
                max: i32,
                #[koruma(value)]
                actual: Option<i32>,
            }
        };

        let result = find_value_field(&input);
        assert!(result.is_some());
        let (name, _ty) = result.unwrap();
        assert_eq!(name.to_string(), "actual");
    }

    #[test]
    fn test_find_value_field_returns_none_when_missing() {
        let input: ItemStruct = syn::parse_quote! {
            pub struct Test {
                min: i32,
                max: i32,
                actual: Option<i32>,
            }
        };

        assert!(find_value_field(&input).is_none());
    }

    #[test]
    fn test_parse_field_with_single_validator() {
        let field: syn::Field = syn::parse_quote! {
            #[koruma(RangeValidation(min = 0, max = 100))]
            pub age: i32
        };

        let result = parse_field(&field);
        assert!(result.is_some());
        let info = result.unwrap();
        assert_eq!(info.name.to_string(), "age");
        assert_eq!(info.validators.len(), 1);
        assert_eq!(info.validators[0].validator.to_string(), "RangeValidation");
        assert!(!info.validators[0].infer_type);
        assert_eq!(info.validators[0].args.len(), 2);
    }

    #[test]
    fn test_parse_field_with_generic_validator() {
        let field: syn::Field = syn::parse_quote! {
            #[koruma(GenericRange<_>(min = 0.0, max = 1.0))]
            pub score: f64
        };

        let result = parse_field(&field);
        assert!(result.is_some());
        let info = result.unwrap();
        assert!(info.validators[0].infer_type);
    }

    #[test]
    fn test_parse_field_with_each() {
        let field: syn::Field = syn::parse_quote! {
            #[koruma(each(RangeValidation(min = 0, max = 100)))]
            pub scores: Vec<i32>
        };

        let result = parse_field(&field);
        assert!(result.is_some());
        let info = result.unwrap();
        assert!(info.validate_each);
        assert_eq!(info.validators.len(), 1);
    }

    #[test]
    fn test_parse_field_with_skip_returns_none() {
        let field: syn::Field = syn::parse_quote! {
            #[koruma(skip)]
            pub internal: u64
        };

        assert!(parse_field(&field).is_none());
    }

    #[test]
    fn test_parse_field_without_koruma_returns_none() {
        let field: syn::Field = syn::parse_quote! {
            pub normal_field: String
        };

        assert!(parse_field(&field).is_none());
    }

    // ============================================================================
    // Error Case Tests
    // ============================================================================

    #[test]
    fn test_validator_error_missing_value_field() {
        let input: ItemStruct = syn::parse_quote! {
            pub struct BadValidator {
                min: i32,
                max: i32,
                // Missing #[koruma(value)] field!
            }
        };

        let result = expand_validator(input);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("koruma(value)"));
    }

    #[test]
    fn test_koruma_error_no_validated_fields() {
        let input: DeriveInput = syn::parse_quote! {
            pub struct EmptyStruct {
                // No #[koruma(...)] attributes
                pub normal_field: i32,
            }
        };

        let result = expand_koruma(input);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("koruma(...)"));
    }

    #[test]
    fn test_koruma_error_on_enum() {
        let input: DeriveInput = syn::parse_quote! {
            pub enum NotAStruct {
                VariantA,
                VariantB,
            }
        };

        let result = expand_koruma(input);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("struct"));
    }

    #[test]
    fn test_koruma_error_on_tuple_struct() {
        let input: DeriveInput = syn::parse_quote! {
            pub struct TupleStruct(i32, String);
        };

        let result = expand_koruma(input);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("named fields"));
    }

    // ============================================================================
    // ValidatorAttr Parsing Tests
    // ============================================================================

    #[test]
    fn test_validator_attr_parse_simple() {
        let attr: ValidatorAttr = syn::parse_quote!(RangeValidation);
        assert_eq!(attr.validator.to_string(), "RangeValidation");
        assert!(!attr.infer_type);
        assert!(attr.args.is_empty());
    }

    #[test]
    fn test_validator_attr_parse_with_args() {
        let attr: ValidatorAttr = syn::parse_quote!(RangeValidation(min = 0, max = 100));
        assert_eq!(attr.validator.to_string(), "RangeValidation");
        assert!(!attr.infer_type);
        assert_eq!(attr.args.len(), 2);
        assert_eq!(attr.args[0].0.to_string(), "min");
        assert_eq!(attr.args[1].0.to_string(), "max");
    }

    #[test]
    fn test_validator_attr_parse_generic() {
        let attr: ValidatorAttr = syn::parse_quote!(GenericRange<_>(min = 0.0, max = 1.0));
        assert_eq!(attr.validator.to_string(), "GenericRange");
        assert!(attr.infer_type);
        assert_eq!(attr.args.len(), 2);
    }

    #[test]
    fn test_koruma_attr_parse_skip() {
        let attr: KorumaAttr = syn::parse_quote!(skip);
        assert!(attr.is_skip);
        assert!(!attr.validate_each);
        assert!(attr.validators.is_empty());
    }

    #[test]
    fn test_koruma_attr_parse_each() {
        let attr: KorumaAttr = syn::parse_quote!(each(RangeValidation(min = 0, max = 100)));
        assert!(!attr.is_skip);
        assert!(attr.validate_each);
        assert_eq!(attr.validators.len(), 1);
    }

    #[test]
    fn test_koruma_attr_parse_multiple_validators() {
        let attr: KorumaAttr = syn::parse_quote!(ValidatorA(x = 1), ValidatorB, ValidatorC<_>(y = 2));
        assert!(!attr.is_skip);
        assert!(!attr.validate_each);
        assert_eq!(attr.validators.len(), 3);
        assert!(!attr.validators[0].infer_type);
        assert!(!attr.validators[1].infer_type);
        assert!(attr.validators[2].infer_type);
    }
}

