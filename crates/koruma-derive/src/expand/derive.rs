use crate::expand::{
    codegen::{
        effective_validation_type, transform_arg_value, validator_type_for_field,
        validator_wants_full_type,
    },
    parse::{FieldInfo, ParseFieldResult, ValidatorAttr, parse_field, parse_struct_options},
    utils::{
        contains_infer_type, first_generic_arg, is_option_type, option_inner_type,
        substitute_infer_type, vec_inner_type,
    },
};
use heck::{ToSnakeCase, ToUpperCamelCase};
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{DeriveInput, Fields};

/// Core expansion logic for the `#[derive(Koruma)]` derive macro.
///
/// Takes a parsed DeriveInput and returns the expanded TokenStream.
pub fn expand_koruma(input: DeriveInput) -> Result<TokenStream2, syn::Error> {
    let struct_name = &input.ident;
    let error_struct_name = format_ident!("{}KorumaValidationError", struct_name);

    // Parse struct-level options like #[koruma(try_new, const_new)]
    let struct_options = parse_struct_options(&input.attrs)?;

    let fields = match &input.data {
        syn::Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => {
                return Err(syn::Error::new_spanned(
                    &input,
                    "Koruma only supports structs with named fields",
                ));
            },
        },
        _ => {
            return Err(syn::Error::new_spanned(
                &input,
                "Koruma can only be derived for structs",
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

    // Validate newtype option - must have exactly one validated field
    if struct_options.newtype && field_infos.len() != 1 {
        return Err(syn::Error::new_spanned(
            &input,
            format!(
                "newtype structs must have exactly one validated field, found {}",
                field_infos.len()
            ),
        ));
    }

    // Generate per-field error structs and collect info for main error struct
    // For nested fields, we don't generate a per-field error struct - we use the nested type's error directly
    // For newtype fields, we generate a wrapper struct with Deref to the inner error
    let field_error_structs: Vec<TokenStream2> = field_infos
        .iter()
        .filter(|f| !f.is_nested()) // Skip nested fields - they use their own error structs
        .map(|f| {
            let field_name = &f.name;
            let field_ty = &f.ty;
            let field_error_struct_name = format_ident!(
                "{}{}KorumaValidationError",
                struct_name,
                field_name.to_string().to_upper_camel_case()
            );

            // Handle newtype fields - generate a wrapper struct with Deref
            if f.is_newtype() {
                let inner_ty = option_inner_type(field_ty).unwrap_or(field_ty);

                return quote! {
                    /// Per-field validation error struct for a newtype field.
                    /// Derefs to the inner type's error struct for transparent access.
                    #[derive(Clone, Debug)]
                    pub struct #field_error_struct_name {
                        inner: Option<<#inner_ty as koruma::ValidateExt>::Error>,
                    }

                    impl #field_error_struct_name {
                        /// Returns the inner validation error if present.
                        pub fn inner(&self) -> Option<&<#inner_ty as koruma::ValidateExt>::Error> {
                            self.inner.as_ref()
                        }

                        pub fn is_empty(&self) -> bool {
                            self.inner.is_none()
                        }

                        pub fn has_errors(&self) -> bool {
                            !self.is_empty()
                        }
                    }

                    impl core::ops::Deref for #field_error_struct_name {
                        type Target = <#inner_ty as koruma::ValidateExt>::Error;

                        fn deref(&self) -> &Self::Target {
                            self.inner.as_ref().expect("newtype field error should have inner error when accessed via Deref")
                        }
                    }
                };
            }

            // Regular field handling below...
            let has_element_validators = f.has_element_validators();

            // Generate fields for field-level validators
            let field_validator_fields: Vec<TokenStream2> = f
                .field_validators
                .iter()
                .map(|v| {
                    let validator_snake =
                        format_ident!("{}", v.name().to_string().to_snake_case());
                    let vtype = validator_type_for_field(v, field_ty, false);
                    quote! { #validator_snake: Option<#vtype> }
                })
                .collect();

            // Generate getter methods for field-level validators
            let field_validator_getters: Vec<TokenStream2> = f
                .field_validators
                .iter()
                .map(|v| {
                    let validator_snake =
                        format_ident!("{}", v.name().to_string().to_snake_case());
                    let vtype = validator_type_for_field(v, field_ty, false);
                    quote! {
                        pub fn #validator_snake(&self) -> Option<&#vtype> {
                            self.#validator_snake.as_ref()
                        }
                    }
                })
                .collect();

            // Generate is_empty checks for field-level validators
            let field_is_empty_checks: Vec<TokenStream2> = f
                .field_validators
                .iter()
                .map(|v| {
                    let validator_snake =
                        format_ident!("{}", v.name().to_string().to_snake_case());
                    quote! { self.#validator_snake.is_none() }
                })
                .collect();

            // Generate element error struct if we have element validators
            let element_error_struct = if has_element_validators {
                let element_error_struct_name = format_ident!(
                    "{}{}ElementKorumaValidationError",
                    struct_name,
                    field_name.to_string().to_upper_camel_case()
                );

                let element_validator_fields: Vec<TokenStream2> = f
                    .element_validators
                    .iter()
                    .map(|v| {
                        let validator_snake =
                            format_ident!("{}", v.name().to_string().to_snake_case());
                        let vtype = validator_type_for_field(v, field_ty, true);
                        quote! { #validator_snake: Option<#vtype> }
                    })
                    .collect();

                let element_validator_getters: Vec<TokenStream2> = f
                    .element_validators
                    .iter()
                    .map(|v| {
                        let validator_snake =
                            format_ident!("{}", v.name().to_string().to_snake_case());
                        let vtype = validator_type_for_field(v, field_ty, true);
                        quote! {
                            pub fn #validator_snake(&self) -> Option<&#vtype> {
                                self.#validator_snake.as_ref()
                            }
                        }
                    })
                    .collect();

                let element_is_empty_checks: Vec<TokenStream2> = f
                    .element_validators
                    .iter()
                    .map(|v| {
                        let validator_snake =
                            format_ident!("{}", v.name().to_string().to_snake_case());
                        quote! { self.#validator_snake.is_none() }
                    })
                    .collect();

                // Generate enum variants for the element all() method
                let element_enum_name = format_ident!(
                    "{}{}ElementKorumaValidator",
                    struct_name,
                    field_name.to_string().to_upper_camel_case()
                );

                let element_enum_variants: Vec<TokenStream2> = f
                    .element_validators
                    .iter()
                    .map(|v| {
                        let variant_name =
                            format_ident!("{}", v.name().to_string().to_upper_camel_case());
                        let vtype = validator_type_for_field(v, field_ty, true);
                        quote! { #variant_name(#vtype) }
                    })
                    .collect();

                let element_all_pushes: Vec<TokenStream2> = f
                    .element_validators
                    .iter()
                    .map(|v| {
                        let validator_snake =
                            format_ident!("{}", v.name().to_string().to_snake_case());
                        let variant_name =
                            format_ident!("{}", v.name().to_string().to_upper_camel_case());
                        quote! {
                            if let Some(v) = &self.#validator_snake {
                                result.push(#element_enum_name::#variant_name(v.clone()));
                            }
                        }
                    })
                    .collect();

                quote! {
                    /// Enum of all possible element validators for this field.
                    #[derive(Clone, Debug)]
                    #[allow(dead_code)]
                    pub enum #element_enum_name {
                        #(#element_enum_variants),*
                    }

                    /// Per-element validation error struct.
                    #[derive(Clone, Debug)]
                    pub struct #element_error_struct_name {
                        #(#element_validator_fields),*
                    }

                    impl #element_error_struct_name {
                        #(#element_validator_getters)*

                        /// Returns all failed element validators.
                        pub fn all(&self) -> Vec<#element_enum_name> {
                            let mut result = Vec::new();
                            #(#element_all_pushes)*
                            result
                        }

                        pub fn is_empty(&self) -> bool {
                            #(#element_is_empty_checks)&&*
                        }

                        pub fn has_errors(&self) -> bool {
                            !self.is_empty()
                        }
                    }
                }
            } else {
                quote! {}
            };

            // Field for storing element errors (if we have element validators)
            let _element_errors_field = if has_element_validators {
                let element_error_struct_name = format_ident!(
                    "{}{}ElementKorumaValidationError",
                    struct_name,
                    field_name.to_string().to_upper_camel_case()
                );
                quote! { element_errors: Vec<(usize, #element_error_struct_name)> }
            } else {
                quote! {}
            };

            // Getter for element errors
            let element_errors_getter = if has_element_validators {
                let element_error_struct_name = format_ident!(
                    "{}{}ElementKorumaValidationError",
                    struct_name,
                    field_name.to_string().to_upper_camel_case()
                );
                quote! {
                    /// Returns all element validation errors with their indices.
                    pub fn element_errors(&self) -> &[(usize, #element_error_struct_name)] {
                        &self.element_errors
                    }
                }
            } else {
                quote! {}
            };

            // is_empty check for element errors
            let element_is_empty_check = if has_element_validators {
                quote! { && self.element_errors.is_empty() }
            } else {
                quote! {}
            };

            // Generate enum variants for the field all() method
            let enum_name = format_ident!(
                "{}{}KorumaValidator",
                struct_name,
                field_name.to_string().to_upper_camel_case()
            );

            let enum_variants: Vec<TokenStream2> = f
                .field_validators
                .iter()
                .map(|v| {
                    let variant_name =
                        format_ident!("{}", v.name().to_string().to_upper_camel_case());
                    let vtype = validator_type_for_field(v, field_ty, false);
                    quote! { #variant_name(#vtype) }
                })
                .collect();

            // Generate the all() method body
            let all_pushes: Vec<TokenStream2> = f
                .field_validators
                .iter()
                .map(|v| {
                    let validator_snake =
                        format_ident!("{}", v.name().to_string().to_snake_case());
                    let variant_name =
                        format_ident!("{}", v.name().to_string().to_upper_camel_case());
                    quote! {
                        if let Some(v) = &self.#validator_snake {
                            result.push(#enum_name::#variant_name(v.clone()));
                        }
                    }
                })
                .collect();

            // Handle case where there are no field validators (only element validators)
            let enum_and_all = if f.field_validators.is_empty() {
                quote! {}
            } else {
                quote! {
                    /// Enum of all possible validators for this field.
                    #[derive(Clone, Debug)]
                    #[allow(dead_code)]
                    pub enum #enum_name {
                        #(#enum_variants),*
                    }
                }
            };

            let all_method = if f.field_validators.is_empty() {
                quote! {}
            } else {
                quote! {
                    /// Returns all failed field-level validators.
                    pub fn all(&self) -> Vec<#enum_name> {
                        let mut result = Vec::new();
                        #(#all_pushes)*
                        result
                    }
                }
            };

            let is_empty_body = if f.field_validators.is_empty() {
                // Only element validators
                quote! { self.element_errors.is_empty() }
            } else {
                quote! { #(#field_is_empty_checks)&&* #element_is_empty_check }
            };

            // Generate struct fields - need proper comma handling
            let struct_fields = if !f.field_validators.is_empty() && f.has_element_validators() {
                // Both field validators and element errors
                let element_error_struct_name = format_ident!(
                    "{}{}ElementKorumaValidationError",
                    struct_name,
                    field_name.to_string().to_upper_camel_case()
                );
                quote! {
                    #(#field_validator_fields,)*
                    element_errors: Vec<(usize, #element_error_struct_name)>
                }
            } else if f.has_element_validators() {
                // Only element errors
                let element_error_struct_name = format_ident!(
                    "{}{}ElementKorumaValidationError",
                    struct_name,
                    field_name.to_string().to_upper_camel_case()
                );
                quote! {
                    element_errors: Vec<(usize, #element_error_struct_name)>
                }
            } else {
                // Only field validators
                quote! {
                    #(#field_validator_fields),*
                }
            };

            quote! {
                #element_error_struct

                #enum_and_all

                #[derive(Clone, Debug)]
                pub struct #field_error_struct_name {
                    #struct_fields
                }

                impl #field_error_struct_name {
                    #(#field_validator_getters)*

                    #element_errors_getter

                    #all_method

                    pub fn is_empty(&self) -> bool {
                        #is_empty_body
                    }

                    pub fn has_errors(&self) -> bool {
                        !self.is_empty()
                    }
                }
            }
        })
        .collect();

    // Generate main error struct fields (one per validated field)
    // Now all fields just have their field error struct (element errors are nested inside)
    // For nested fields, we use Option<NestedTypeKorumaValidationError> directly
    let error_fields: Vec<TokenStream2> = field_infos
        .iter()
        .map(|f| {
            let field_name = &f.name;
            if f.is_nested() {
                // For nested fields, use Option<NestedTypeKorumaValidationError>
                // We need to derive the error type name from the field type
                let field_ty = &f.ty;
                // Handle Option<T> by extracting T
                let inner_ty = option_inner_type(field_ty).unwrap_or(field_ty);
                quote! { #field_name: Option<<#inner_ty as koruma::ValidateExt>::Error> }
            } else {
                let field_error_struct_name = format_ident!(
                    "{}{}KorumaValidationError",
                    struct_name,
                    field_name.to_string().to_upper_camel_case()
                );
                quote! { #field_name: #field_error_struct_name }
            }
        })
        .collect();

    // Generate getter methods for main error struct
    let getter_methods: Vec<TokenStream2> = field_infos
        .iter()
        .map(|f| {
            let field_name = &f.name;
            if f.is_nested() {
                // For nested fields, return Option<&NestedTypeKorumaValidationError>
                let field_ty = &f.ty;
                let inner_ty = option_inner_type(field_ty).unwrap_or(field_ty);
                quote! {
                    pub fn #field_name(&self) -> Option<&<#inner_ty as koruma::ValidateExt>::Error> {
                        self.#field_name.as_ref()
                    }
                }
            } else {
                let field_error_struct_name = format_ident!(
                    "{}{}KorumaValidationError",
                    struct_name,
                    field_name.to_string().to_upper_camel_case()
                );
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
            if f.is_nested() {
                // For nested fields, check if Option is None
                quote! { self.#field_name.is_none() }
            } else {
                quote! { self.#field_name.is_empty() }
            }
        })
        .collect();

    // Generate is_empty body - either `true` or the checks
    let is_empty_body = if is_empty_checks.is_empty() {
        quote! { true }
    } else {
        quote! { #(#is_empty_checks)&&* }
    };

    // Generate default values for main error struct initialization
    let error_defaults: Vec<TokenStream2> = field_infos
        .iter()
        .map(|f| {
            let field_name = &f.name;

            // For nested fields, default to None
            if f.is_nested() {
                return quote! { #field_name: None };
            }

            let field_error_struct_name = format_ident!(
                "{}{}KorumaValidationError",
                struct_name,
                field_name.to_string().to_upper_camel_case()
            );

            // For newtype fields, default inner to None
            if f.is_newtype() {
                return quote! {
                    #field_name: #field_error_struct_name {
                        inner: None
                    }
                };
            }

            // Generate defaults for field-level validators
            let field_validator_defaults: Vec<TokenStream2> = f
                .field_validators
                .iter()
                .map(|v| {
                    let validator_snake = format_ident!("{}", v.name().to_string().to_snake_case());
                    quote! { #validator_snake: None }
                })
                .collect();

            // Handle different combinations of field/element validators
            if f.has_element_validators() && f.field_validators.is_empty() {
                // Only element validators
                quote! {
                    #field_name: #field_error_struct_name {
                        element_errors: Vec::new()
                    }
                }
            } else if f.has_element_validators() {
                // Both field and element validators
                quote! {
                    #field_name: #field_error_struct_name {
                        #(#field_validator_defaults),*,
                        element_errors: Vec::new()
                    }
                }
            } else {
                // Only field validators
                quote! {
                    #field_name: #field_error_struct_name {
                        #(#field_validator_defaults),*
                    }
                }
            }
        })
        .collect();

    // Generate validation logic - supports both field validators, element validators, and nested structs
    let validation_checks: Vec<TokenStream2> = field_infos
        .iter()
        .map(|f| {
            let field_name = &f.name;
            let field_ty = &f.ty;

            // Handle nested fields - call validate() on the nested struct
            if f.is_nested() {
                let field_is_optional = is_option_type(field_ty);
                if field_is_optional {
                    // For Option<NestedType>, only validate if Some
                    return quote! {
                        if let Some(ref __nested_value) = self.#field_name {
                            if let Err(nested_err) = __nested_value.validate() {
                                error.#field_name = Some(nested_err);
                                has_error = true;
                            }
                        }
                    };
                } else {
                    // For non-optional nested field, always validate
                    return quote! {
                        if let Err(nested_err) = self.#field_name.validate() {
                            error.#field_name = Some(nested_err);
                            has_error = true;
                        }
                    };
                }
            }

            // Handle newtype fields - similar to nested but stores in wrapper struct
            if f.is_newtype() {
                let field_is_optional = is_option_type(field_ty);
                if field_is_optional {
                    // For Option<NewtypeType>, only validate if Some
                    return quote! {
                        if let Some(ref __newtype_value) = self.#field_name {
                            if let Err(newtype_err) = __newtype_value.validate() {
                                error.#field_name.inner = Some(newtype_err);
                                has_error = true;
                            }
                        }
                    };
                } else {
                    // For non-optional newtype field, always validate
                    return quote! {
                        if let Err(newtype_err) = self.#field_name.validate() {
                            error.#field_name.inner = Some(newtype_err);
                            has_error = true;
                        }
                    };
                }
            }

            let has_element_validators = f.has_element_validators();

            // Split field validators into those that want the full type vs those that want
            // the unwrapped type (for Option fields)
            let (full_type_validators, unwrapped_validators): (Vec<_>, Vec<_>) = f
                .field_validators
                .iter()
                .partition(|v| validator_wants_full_type(v));

            // Helper to generate validator check code
            let generate_validator_check =
                |v: &ValidatorAttr, value_expr: TokenStream2, needs_ref: bool| -> TokenStream2 {
                    let validator = &v.validator;
                    let validator_snake = format_ident!("{}", v.name().to_string().to_snake_case());
                    let effective_ty = effective_validation_type(field_ty, false);

                    let builder_calls: Vec<TokenStream2> = v
                        .args
                        .iter()
                        .map(|(arg_name, arg_value)| {
                            let transformed = transform_arg_value(arg_value);
                            quote! { .#arg_name(#transformed) }
                        })
                        .collect();

                    // The reference expression for validate()
                    let ref_expr = if needs_ref {
                        quote! { &#value_expr }
                    } else {
                        quote! { #value_expr }
                    };

                    // Determine the validator type
                    let uses_infer =
                        v.infer_type || v.explicit_type.as_ref().is_some_and(contains_infer_type);

                    if uses_infer {
                        let validator_ty = if let Some(ref explicit_ty) = v.explicit_type {
                            if contains_infer_type(explicit_ty) {
                                // For Option<_>, first_generic_arg gets the inner type
                                let inner_ty = first_generic_arg(field_ty).unwrap_or(field_ty);
                                let substituted = substitute_infer_type(explicit_ty, inner_ty);
                                quote! { #substituted }
                            } else {
                                quote! { #explicit_ty }
                            }
                        } else {
                            quote! { #effective_ty }
                        };
                        let assert_fn = format_ident!(
                            "__koruma_assert_validate_{}_{}_field",
                            field_name,
                            validator_snake
                        );
                        quote! {
                            fn #assert_fn<V: koruma::Validate<T>, T>(v: &V, t: &T) -> bool {
                                v.validate(t)
                            }
                            let validator = #validator::<#validator_ty>::builder()
                                #(#builder_calls)*
                                .with_value(#value_expr.clone())
                                .build();
                            if !#assert_fn(&validator, #ref_expr) {
                                error.#field_name.#validator_snake = Some(validator);
                                has_error = true;
                            }
                        }
                    } else {
                        quote! {
                            let validator = #validator::builder()
                                #(#builder_calls)*
                                .with_value(#value_expr.clone())
                                .build();
                            if !validator.validate(#ref_expr) {
                                error.#field_name.#validator_snake = Some(validator);
                                has_error = true;
                            }
                        }
                    }
                };

            // Generate checks for full-type validators (use field directly, no reference)
            // Note: we pass the field expression without &, the closure adds .clone() for with_value
            // and &... for validate()
            let full_type_checks: Vec<TokenStream2> = full_type_validators
                .iter()
                .map(|v| generate_validator_check(v, quote! { self.#field_name }, true))
                .collect();

            // Generate checks for unwrapped validators (use __field_value which is already a ref)
            let unwrapped_checks: Vec<TokenStream2> = unwrapped_validators
                .iter()
                .map(|v| generate_validator_check(v, quote! { __field_value }, false))
                .collect();

            // Generate element-level validation checks if we have element validators
            let element_validation = if has_element_validators {
                let element_error_struct_name = format_ident!(
                    "{}{}ElementKorumaValidationError",
                    struct_name,
                    field_name.to_string().to_upper_camel_case()
                );

                let element_ty = vec_inner_type(field_ty).unwrap_or(field_ty);
                let element_is_optional = is_option_type(element_ty);
                let effective_element_ty = effective_validation_type(field_ty, true);

                let element_validator_checks: Vec<TokenStream2> = f
                    .element_validators
                    .iter()
                    .map(|v| {
                        let validator = &v.validator;
                        let validator_snake =
                            format_ident!("{}", v.name().to_string().to_snake_case());

                        let builder_calls: Vec<TokenStream2> = v
                            .args
                            .iter()
                            .map(|(arg_name, arg_value)| {
                                let transformed = transform_arg_value(arg_value);
                                quote! { .#arg_name(#transformed) }
                            })
                            .collect();

                        if v.infer_type || v.explicit_type.as_ref().is_some_and(contains_infer_type)
                        {
                            let validator_ty = if let Some(ref explicit_ty) = v.explicit_type {
                                if contains_infer_type(explicit_ty) {
                                    let inner_ty =
                                        first_generic_arg(element_ty).unwrap_or(element_ty);
                                    let substituted = substitute_infer_type(explicit_ty, inner_ty);
                                    quote! { #substituted }
                                } else {
                                    quote! { #explicit_ty }
                                }
                            } else {
                                quote! { #effective_element_ty }
                            };
                            let assert_fn = format_ident!(
                                "__koruma_assert_validate_{}_{}_element",
                                field_name,
                                validator_snake
                            );
                            quote! {
                                fn #assert_fn<V: koruma::Validate<T>, T>(v: &V, t: &T) -> bool {
                                    v.validate(t)
                                }
                                let validator = #validator::<#validator_ty>::builder()
                                    #(#builder_calls)*
                                    .with_value(__item_value.clone())
                                    .build();
                                if !#assert_fn(&validator, __item_value) {
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
                                if !validator.validate(__item_value) {
                                    element_error.#validator_snake = Some(validator);
                                    element_has_error = true;
                                }
                            }
                        }
                    })
                    .collect();

                let element_validator_defaults: Vec<TokenStream2> = f
                    .element_validators
                    .iter()
                    .map(|v| {
                        let validator_snake =
                            format_ident!("{}", v.name().to_string().to_snake_case());
                        quote! { #validator_snake: None }
                    })
                    .collect();

                let inner_element_validation = quote! {
                    let mut element_error = #element_error_struct_name {
                        #(#element_validator_defaults),*
                    };
                    let mut element_has_error = false;

                    #(#element_validator_checks)*

                    if element_has_error {
                        error.#field_name.element_errors.push((idx, element_error));
                        has_error = true;
                    }
                };

                if element_is_optional {
                    // For Vec<Option<T>>, skip None items
                    quote! {
                        for (idx, item) in self.#field_name.iter().enumerate() {
                            if let Some(ref __item_value) = item {
                                #inner_element_validation
                            }
                        }
                    }
                } else {
                    // For Vec<T>, validate each item directly
                    quote! {
                        for (idx, __item_value) in self.#field_name.iter().enumerate() {
                            #inner_element_validation
                        }
                    }
                }
            } else {
                quote! {}
            };

            // Combine field validation and element validation
            let field_is_optional = is_option_type(field_ty);
            let has_full_type_validators = !full_type_validators.is_empty();
            let has_unwrapped_validators = !unwrapped_validators.is_empty();

            // Full-type validators run on the field directly (no Option unwrapping)
            // Unwrapped validators run on the inner value (inside if let Some for Option fields)
            if has_full_type_validators && has_unwrapped_validators && field_is_optional {
                // Both full-type and unwrapped validators, optional field
                quote! {
                    #(#full_type_checks)*
                    if let Some(ref __field_value) = self.#field_name {
                        #(#unwrapped_checks)*
                    }
                    #element_validation
                }
            } else if has_full_type_validators && has_unwrapped_validators {
                // Both types, non-optional field
                quote! {
                    #(#full_type_checks)*
                    let __field_value = &self.#field_name;
                    #(#unwrapped_checks)*
                    #element_validation
                }
            } else if has_full_type_validators {
                // Only full-type validators
                quote! {
                    #(#full_type_checks)*
                    #element_validation
                }
            } else if has_unwrapped_validators && field_is_optional {
                // Only unwrapped validators, optional field
                quote! {
                    if let Some(ref __field_value) = self.#field_name {
                        #(#unwrapped_checks)*
                    }
                    #element_validation
                }
            } else if has_unwrapped_validators {
                // Only unwrapped validators, non-optional field
                quote! {
                    let __field_value = &self.#field_name;
                    #(#unwrapped_checks)*
                    #element_validation
                }
            } else {
                // No field validators, only element validators
                element_validation
            }
        })
        .collect();

    // Generate try_new function if requested
    let try_new_fn = if struct_options.try_new {
        // Collect all struct fields (not just validated ones) for constructor parameters
        let all_field_params: Vec<TokenStream2> = fields
            .iter()
            .map(|f| {
                let name = f.ident.as_ref().unwrap();
                let ty = &f.ty;
                quote! { #name: #ty }
            })
            .collect();

        let all_field_names: Vec<&syn::Ident> =
            fields.iter().map(|f| f.ident.as_ref().unwrap()).collect();

        quote! {
            /// Creates a new instance and validates it.
            ///
            /// Returns `Ok(instance)` if all validations pass, or `Err(error)` where
            /// `error` contains the validation failures for each field.
            pub fn try_new(#(#all_field_params),*) -> Result<Self, #error_struct_name> {
                let instance = Self {
                    #(#all_field_names),*
                };
                instance.validate()?;
                Ok(instance)
            }
        }
    } else {
        quote! {}
    };

    // Generate NewtypeValidation marker trait impl for struct-level newtypes
    let newtype_marker_impl = if struct_options.newtype {
        quote! {
            impl koruma::NewtypeValidation for #struct_name {}
        }
    } else {
        quote! {}
    };

    // Generate Deref impl for newtype error structs
    let newtype_deref_impl = if struct_options.newtype {
        let field_info = &field_infos[0];
        let field_name = &field_info.name;
        let field_ty = &field_info.ty;

        if field_info.is_nested() {
            // For nested newtypes, deref to the inner type's error struct
            let inner_ty = option_inner_type(field_ty).unwrap_or(field_ty);
            let is_optional = is_option_type(field_ty);

            if is_optional {
                // For Option<NestedType>, we can't implement Deref since the error might not exist
                // Instead, we'll just add a convenience method
                quote! {}
            } else {
                // For non-optional nested, we can deref directly
                // But the field is Option in the error struct, so we need to handle that
                // Actually, for newtype we should change the error struct to not use Option
                // Let's add a deref that panics if no error (which shouldn't happen if we have an error struct)
                quote! {
                    impl core::ops::Deref for #error_struct_name {
                        type Target = <#inner_ty as koruma::ValidateExt>::Error;

                        fn deref(&self) -> &Self::Target {
                            self.#field_name.as_ref().expect("newtype error should have inner error")
                        }
                    }
                }
            }
        } else {
            // For newtypes with validators, deref to the per-field error struct
            let field_error_struct_name = format_ident!(
                "{}{}KorumaValidationError",
                struct_name,
                field_name.to_string().to_upper_camel_case()
            );
            quote! {
                impl core::ops::Deref for #error_struct_name {
                    type Target = #field_error_struct_name;

                    fn deref(&self) -> &Self::Target {
                        &self.#field_name
                    }
                }
            }
        }
    } else {
        quote! {}
    };

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

        #newtype_deref_impl

        impl koruma::ValidationError for #error_struct_name {
            fn is_empty(&self) -> bool {
                #is_empty_body
            }
        }

        impl #struct_name {
            #try_new_fn

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

        impl koruma::ValidateExt for #struct_name {
            type Error = #error_struct_name;

            fn validate(&self) -> Result<(), #error_struct_name> {
                #struct_name::validate(self)
            }
        }

        #newtype_marker_impl
    })
}
