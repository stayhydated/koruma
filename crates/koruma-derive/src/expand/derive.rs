use crate::expand::codegen::{
    each_element_type, helper_generics_for_usages, validator_builder_expr, validator_field_ident,
    validator_type_for_field, validator_variant_ident, validator_wants_full_type,
};
use crate::expand::plan::ValidationPlan;
use heck::ToUpperCamelCase;
use koruma_derive_core::{
    FieldInfo, ValidatorAttr, contains_infer_type, is_option_type, option_inner_type,
};
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::DeriveInput;

/// Core expansion logic for the `#[derive(Koruma)]` derive macro.
///
/// Takes a parsed DeriveInput and returns the expanded TokenStream.
pub fn expand_koruma(input: DeriveInput) -> Result<TokenStream2, syn::Error> {
    let struct_name = &input.ident;
    let error_struct_name = format_ident!("{}KorumaValidationError", struct_name);
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let plan = ValidationPlan::build(&input, "Koruma")?;
    let struct_options = &plan.struct_options;
    let field_infos = plan.field_infos();
    let struct_newtype_field_info = plan.struct_newtype_field_info.clone();
    let known_field_names = &plan.known_field_names;
    let fields = match &input.data {
        syn::Data::Struct(data) => &data.fields,
        _ => unreachable!("ValidationPlan already rejects non-struct inputs"),
    };

    let field_error_type_path = |f: &FieldInfo| -> Option<TokenStream2> {
        if f.is_nested() {
            return None;
        }

        let field_name = &f.name;
        let field_ty = &f.ty;
        let field_error_struct_name = format_ident!(
            "{}{}KorumaValidationError",
            struct_name,
            field_name.to_string().to_upper_camel_case()
        );

        let mut usages: Vec<TokenStream2> = if f.is_newtype() {
            let inner_ty = option_inner_type(field_ty).unwrap_or(field_ty);
            let mut usages: Vec<TokenStream2> = f
                .validation
                .field_validators
                .iter()
                .map(|v| {
                    let vtype = validator_type_for_field(v, field_ty, false);
                    quote! { #vtype }
                })
                .collect();
            usages.push(quote! { <#inner_ty as koruma::ValidateExt>::Error });
            usages
        } else {
            f.validation
                .field_validators
                .iter()
                .map(|v| {
                    let vtype = validator_type_for_field(v, field_ty, false);
                    quote! { #vtype }
                })
                .collect()
        };

        if !f.is_newtype() && f.has_element_validators() {
            let element_error_struct_name = format_ident!(
                "{}{}ElementKorumaValidationError",
                struct_name,
                field_name.to_string().to_upper_camel_case()
            );
            let element_usages: Vec<TokenStream2> = f
                .validation
                .element_validators
                .iter()
                .map(|v| {
                    let vtype = validator_type_for_field(v, field_ty, true);
                    quote! { #vtype }
                })
                .collect();
            let element_helper = helper_generics_for_usages(generics, &element_usages);
            let element_error_path = element_helper.type_path(&element_error_struct_name);
            usages.push(quote! { Vec<(usize, #element_error_path)> });
        }

        let helper = helper_generics_for_usages(generics, &usages);
        Some(helper.type_path(&field_error_struct_name))
    };

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
                let has_field_validators = !f.validation.field_validators.is_empty();
                let field_name_str = field_name.to_string();
                let struct_name_str = struct_name.to_string();

                // If the newtype field has no validators, generate simple wrapper
                if !has_field_validators {
                    let field_is_optional = is_option_type(field_ty);
                    let helper_generics =
                        helper_generics_for_usages(generics, &[quote! { <#inner_ty as koruma::ValidateExt>::Error }]);
                    let helper_definition = &helper_generics.definition;
                    let helper_impl_generics = &helper_generics.impl_generics;
                    let helper_ty_generics = &helper_generics.ty_generics;
                    let helper_where_clause = &helper_generics.where_clause;
                    let inner_field_ty = if field_is_optional {
                        quote! { Option<<#inner_ty as koruma::ValidateExt>::Error> }
                    } else {
                        quote! { <#inner_ty as koruma::ValidateExt>::Error }
                    };
                    let inner_getter = if field_is_optional {
                        quote! {
                            #[doc = concat!("Returns the inner validation error for `", #field_name_str, "`, if any.")]
                            pub fn inner(&self) -> Option<&<#inner_ty as koruma::ValidateExt>::Error> {
                                self.inner.as_ref()
                            }
                        }
                    } else {
                        quote! {
                            #[doc = concat!("Returns the inner validation error for `", #field_name_str, "`.")]
                            pub fn inner(&self) -> &<#inner_ty as koruma::ValidateExt>::Error {
                                &self.inner
                            }
                        }
                    };
                    let is_empty_check = if field_is_optional {
                        quote! {
                            self.inner.as_ref().map_or(true, |inner| inner.is_empty())
                        }
                    } else {
                        quote! { self.inner.is_empty() }
                    };
                    let deref_impl = if field_is_optional {
                        quote! {}
                    } else {
                        quote! {
                            impl #helper_impl_generics std::ops::Deref for #field_error_struct_name #helper_ty_generics #helper_where_clause {
                                type Target = <#inner_ty as koruma::ValidateExt>::Error;

                                fn deref(&self) -> &Self::Target {
                                    &self.inner
                                }
                            }
                        }
                    };

                    return quote! {
                        #[doc = concat!("Validation errors for the `", #field_name_str, "` newtype field of [`", #struct_name_str, "`].")]
                        #[derive(Clone, Debug, Default)]
                        pub struct #field_error_struct_name #helper_definition {
                            inner: #inner_field_ty,
                        }

                        impl #helper_impl_generics #field_error_struct_name #helper_ty_generics #helper_where_clause {
                            #inner_getter

                            pub fn is_empty(&self) -> bool {
                                #is_empty_check
                            }

                            pub fn has_errors(&self) -> bool {
                                !self.is_empty()
                            }
                        }

                        #deref_impl
                    };
                }

                // Newtype field with validators - generate full error struct like regular fields
                // but also include the inner newtype error
                let field_validator_names: Vec<String> = f
                    .validation
                    .field_validators
                    .iter()
                    .map(|v| v.path_name())
                    .collect();
                let validators_list = field_validator_names.join("`], `[");
                let mut helper_usages: Vec<TokenStream2> = f
                    .validation
                    .field_validators
                    .iter()
                    .map(|v| {
                        let vtype = validator_type_for_field(v, field_ty, false);
                        quote! { #vtype }
                    })
                    .collect();
                helper_usages.push(quote! { <#inner_ty as koruma::ValidateExt>::Error });
                let helper_generics = helper_generics_for_usages(generics, &helper_usages);
                let helper_definition = &helper_generics.definition;
                let helper_impl_generics = &helper_generics.impl_generics;
                let helper_ty_generics = &helper_generics.ty_generics;
                let helper_where_clause = &helper_generics.where_clause;
                let field_is_optional = is_option_type(field_ty);

                let field_validator_fields: Vec<TokenStream2> = f
                    .validation
                    .field_validators
                    .iter()
                    .map(|v| {
                        let validator_snake =
                            validator_field_ident(v, &f.validation.field_validators);
                        let vtype = validator_type_for_field(v, field_ty, false);
                        quote! { #validator_snake: Option<#vtype> }
                    })
                    .collect();

                let field_validator_getters: Vec<TokenStream2> = f
                    .validation
                    .field_validators
                    .iter()
                    .map(|v| {
                        let validator_snake =
                            validator_field_ident(v, &f.validation.field_validators);
                        let validator_name = v.path_name();
                        let vtype = validator_type_for_field(v, field_ty, false);
                        quote! {
                            #[doc = concat!("Returns the failed `", #validator_name, "` validator, if any.")]
                            pub fn #validator_snake(&self) -> Option<&#vtype> {
                                self.#validator_snake.as_ref()
                            }
                        }
                    })
                    .collect();

                let field_is_empty_checks: Vec<TokenStream2> = f
                    .validation
                    .field_validators
                    .iter()
                    .map(|v| {
                        let validator_snake =
                            validator_field_ident(v, &f.validation.field_validators);
                        quote! { self.#validator_snake.is_none() }
                    })
                    .collect();

                // Generate enum variants for the all() method
                let enum_name = format_ident!(
                    "{}{}KorumaValidator",
                    struct_name,
                    field_name.to_string().to_upper_camel_case()
                );
                let enum_path = helper_generics.type_path(&enum_name);

                let enum_variants: Vec<TokenStream2> = f
                    .validation
                    .field_validators
                    .iter()
                    .map(|v| {
                        let variant_name =
                            validator_variant_ident(v, &f.validation.field_validators);
                        let vtype = validator_type_for_field(v, field_ty, false);
                        quote! { #variant_name(#vtype) }
                    })
                    .collect();

                // Add Inner variant for wrapping inner newtype validation errors
                let inner_error_ty = quote! { <#inner_ty as koruma::ValidateExt>::Error };
                let inner_variant = quote! { Inner(#inner_error_ty) };

                let all_pushes: Vec<TokenStream2> = f
                    .validation
                    .field_validators
                    .iter()
                    .map(|v| {
                        let validator_snake =
                            validator_field_ident(v, &f.validation.field_validators);
                        let variant_name =
                            validator_variant_ident(v, &f.validation.field_validators);
                        quote! {
                            if let Some(v) = &self.#validator_snake {
                                result.push(#enum_name::#variant_name(v.clone()));
                            }
                        }
                    })
                    .collect();

                // Add push for inner error when it's not empty
                let inner_push = quote! {
                    if let Some(inner) = &self.inner {
                        if !inner.is_empty() {
                            result.push(#enum_name::Inner(inner.clone()));
                        }
                    }
                };
                let inner_push = if field_is_optional {
                    inner_push
                } else {
                    quote! {
                        if !self.inner.is_empty() {
                            result.push(#enum_name::Inner(self.inner.clone()));
                        }
                    }
                };
                let inner_field_ty = if field_is_optional {
                    quote! { Option<<#inner_ty as koruma::ValidateExt>::Error> }
                } else {
                    quote! { <#inner_ty as koruma::ValidateExt>::Error }
                };
                let inner_getter = if field_is_optional {
                    quote! {
                        #[doc = concat!("Returns the inner validation error for `", #field_name_str, "`, if any.")]
                        pub fn inner(&self) -> Option<&<#inner_ty as koruma::ValidateExt>::Error> {
                            self.inner.as_ref()
                        }
                    }
                } else {
                    quote! {
                        #[doc = concat!("Returns the inner validation error for `", #field_name_str, "`.")]
                        pub fn inner(&self) -> &<#inner_ty as koruma::ValidateExt>::Error {
                            &self.inner
                        }
                    }
                };
                let inner_is_empty_check = if field_is_optional {
                    quote! { self.inner.as_ref().map_or(true, |inner| inner.is_empty()) }
                } else {
                    quote! { self.inner.is_empty() }
                };

                return quote! {
                    #[doc = concat!("Validators for the `", #field_name_str, "` field of [`", #struct_name_str, "`]: [`", #validators_list, "`] (plus inner validation).")]
                    #[derive(Clone, Debug)]
                    #[allow(dead_code)]
                    pub enum #enum_name #helper_definition {
                        #(#enum_variants,)*
                        #inner_variant
                    }

                    #[doc = concat!("Validation errors for the `", #field_name_str, "` field of [`", #struct_name_str, "`].\n\nValidators: [`", #validators_list, "`].")]
                    #[derive(Clone, Debug, Default)]
                    pub struct #field_error_struct_name #helper_definition {
                        #(#field_validator_fields,)*
                        inner: #inner_field_ty,
                    }

                    impl #helper_impl_generics #field_error_struct_name #helper_ty_generics #helper_where_clause {
                        #(#field_validator_getters)*

                        #inner_getter

                        #[doc = concat!("Returns all failed validators for `", #field_name_str, "` including inner newtype validation errors.")]
                        pub fn all(&self) -> Vec<#enum_path> {
                            let mut result = Vec::new();
                            #(#all_pushes)*
                            #inner_push
                            result
                        }

                        pub fn is_empty(&self) -> bool {
                            #(#field_is_empty_checks)&&* && #inner_is_empty_check
                        }

                        pub fn has_errors(&self) -> bool {
                            !self.is_empty()
                        }
                    }
                };
            }

            // Regular field handling below...
            let has_element_validators = f.has_element_validators();

            // Generate fields for field-level validators
            let field_validator_fields: Vec<TokenStream2> = f
                .validation.field_validators
                .iter()
                .map(|v| {
                    let validator_snake =
                        validator_field_ident(v, &f.validation.field_validators);
                    let vtype = validator_type_for_field(v, field_ty, false);
                    quote! { #validator_snake: Option<#vtype> }
                })
                .collect();

            // Generate getter methods for field-level validators
            let field_validator_getters: Vec<TokenStream2> = f
                .validation.field_validators
                .iter()
                .map(|v| {
                    let validator_snake =
                        validator_field_ident(v, &f.validation.field_validators);
                    let validator_name = v.path_name();
                    let vtype = validator_type_for_field(v, field_ty, false);
                    quote! {
                        #[doc = concat!("Returns the failed [`", #validator_name, "`] validator, if any.")]
                        pub fn #validator_snake(&self) -> Option<&#vtype> {
                            self.#validator_snake.as_ref()
                        }
                    }
                })
                .collect();

            // Generate is_empty checks for field-level validators
            let field_is_empty_checks: Vec<TokenStream2> = f
                .validation.field_validators
                .iter()
                .map(|v| {
                    let validator_snake =
                        validator_field_ident(v, &f.validation.field_validators);
                    quote! { self.#validator_snake.is_none() }
                })
                .collect();
            let field_validator_usages: Vec<TokenStream2> = f
                .validation
                .field_validators
                .iter()
                .map(|v| {
                    let vtype = validator_type_for_field(v, field_ty, false);
                    quote! { #vtype }
                })
                .collect();
            let element_error_struct_name = format_ident!(
                "{}{}ElementKorumaValidationError",
                struct_name,
                field_name.to_string().to_upper_camel_case()
            );
            let element_enum_name = format_ident!(
                "{}{}ElementKorumaValidator",
                struct_name,
                field_name.to_string().to_upper_camel_case()
            );
            let element_validator_usages: Vec<TokenStream2> = f
                .validation
                .element_validators
                .iter()
                .map(|v| {
                    let vtype = validator_type_for_field(v, field_ty, true);
                    quote! { #vtype }
                })
                .collect();
            let element_helper_generics = has_element_validators
                .then(|| helper_generics_for_usages(generics, &element_validator_usages));
            let element_error_path = element_helper_generics
                .as_ref()
                .map(|helper| helper.type_path(&element_error_struct_name));
            let mut field_error_usages = field_validator_usages.clone();
            if let Some(element_error_path) = &element_error_path {
                field_error_usages.push(quote! { Vec<(usize, #element_error_path)> });
            }
            let field_error_helper_generics =
                helper_generics_for_usages(generics, &field_error_usages);
            let field_error_definition = &field_error_helper_generics.definition;
            let field_error_impl_generics = &field_error_helper_generics.impl_generics;
            let field_error_ty_generics = &field_error_helper_generics.ty_generics;
            let field_error_where_clause = &field_error_helper_generics.where_clause;

            // Generate element error struct if we have element validators
            let element_error_struct = if has_element_validators {
                let element_helper_generics =
                    element_helper_generics.as_ref().expect("element validators checked");
                let element_definition = &element_helper_generics.definition;
                let element_impl_generics = &element_helper_generics.impl_generics;
                let element_ty_generics = &element_helper_generics.ty_generics;
                let element_where_clause = &element_helper_generics.where_clause;
                let element_enum_path = element_helper_generics.type_path(&element_enum_name);

                let element_validator_names: Vec<String> = f
                    .validation.element_validators
                    .iter()
                    .map(|v| v.path_name())
                    .collect();
                let element_validators_list = element_validator_names.join("`], `[");
                let field_name_str = field_name.to_string();
                let struct_name_str = struct_name.to_string();

                let element_validator_fields: Vec<TokenStream2> = f
                    .validation.element_validators
                    .iter()
                    .map(|v| {
                        let validator_snake =
                            validator_field_ident(v, &f.validation.element_validators);
                        let vtype = validator_type_for_field(v, field_ty, true);
                        quote! { #validator_snake: Option<#vtype> }
                    })
                    .collect();

                let element_validator_getters: Vec<TokenStream2> = f
                    .validation.element_validators
                    .iter()
                    .map(|v| {
                        let validator_snake =
                            validator_field_ident(v, &f.validation.element_validators);
                        let validator_name = v.path_name();
                        let vtype = validator_type_for_field(v, field_ty, true);
                        quote! {
                            #[doc = concat!("Returns the failed [`", #validator_name, "`] validator, if any.")]
                            pub fn #validator_snake(&self) -> Option<&#vtype> {
                                self.#validator_snake.as_ref()
                            }
                        }
                    })
                    .collect();

                let element_is_empty_checks: Vec<TokenStream2> = f
                    .validation.element_validators
                    .iter()
                    .map(|v| {
                        let validator_snake =
                            validator_field_ident(v, &f.validation.element_validators);
                        quote! { self.#validator_snake.is_none() }
                    })
                    .collect();

                let element_enum_variants: Vec<TokenStream2> = f
                    .validation.element_validators
                    .iter()
                    .map(|v| {
                        let variant_name =
                            validator_variant_ident(v, &f.validation.element_validators);
                        let vtype = validator_type_for_field(v, field_ty, true);
                        quote! { #variant_name(#vtype) }
                    })
                    .collect();

                let element_all_pushes: Vec<TokenStream2> = f
                    .validation.element_validators
                    .iter()
                    .map(|v| {
                        let validator_snake =
                            validator_field_ident(v, &f.validation.element_validators);
                        let variant_name =
                            validator_variant_ident(v, &f.validation.element_validators);
                        quote! {
                            if let Some(v) = &self.#validator_snake {
                                result.push(#element_enum_name::#variant_name(v.clone()));
                            }
                        }
                    })
                    .collect();

                quote! {
                    #[doc = concat!("Element validators for the `", #field_name_str, "` field of [`", #struct_name_str, "`]: [`", #element_validators_list, "`].")]
                    #[derive(Clone, Debug)]
                    #[allow(dead_code)]
                    pub enum #element_enum_name #element_definition {
                        #(#element_enum_variants),*
                    }

                    #[doc = concat!("Per-element validation errors for the `", #field_name_str, "` field of [`", #struct_name_str, "`].")]
                    #[derive(Clone, Debug, Default)]
                    pub struct #element_error_struct_name #element_definition {
                        #(#element_validator_fields),*
                    }

                    impl #element_impl_generics #element_error_struct_name #element_ty_generics #element_where_clause {
                        #(#element_validator_getters)*

                        #[doc = concat!("Returns all failed element validators for `", #field_name_str, "`.")]
                        pub fn all(&self) -> Vec<#element_enum_path> {
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
                let element_error_path = element_error_path
                    .as_ref()
                    .expect("element validators should have an error path");
                quote! { element_errors: Vec<(usize, #element_error_path)> }
            } else {
                quote! {}
            };

            // Getter for element errors
            let element_errors_getter = if has_element_validators {
                let element_error_path = element_error_path
                    .as_ref()
                    .expect("element validators should have an error path");
                let field_name_str = field_name.to_string();
                let struct_name_str = struct_name.to_string();
                quote! {
                    #[doc = concat!("Returns all element validation errors for `", #field_name_str, "` of [`", #struct_name_str, "`] with their indices.")]
                    pub fn element_errors(&self) -> &[(usize, #element_error_path)] {
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
            let field_enum_helper_generics =
                helper_generics_for_usages(generics, &field_validator_usages);
            let field_enum_definition = &field_enum_helper_generics.definition;
            let field_enum_path = field_enum_helper_generics.type_path(&enum_name);

            let enum_variants: Vec<TokenStream2> = f
                .validation.field_validators
                .iter()
                .map(|v| {
                    let variant_name =
                        validator_variant_ident(v, &f.validation.field_validators);
                    let vtype = validator_type_for_field(v, field_ty, false);
                    quote! { #variant_name(#vtype) }
                })
                .collect();

            // Generate the all() method body
            let all_pushes: Vec<TokenStream2> = f
                .validation.field_validators
                .iter()
                .map(|v| {
                    let validator_snake =
                        validator_field_ident(v, &f.validation.field_validators);
                    let variant_name =
                        validator_variant_ident(v, &f.validation.field_validators);
                    quote! {
                        if let Some(v) = &self.#validator_snake {
                            result.push(#enum_name::#variant_name(v.clone()));
                        }
                    }
                })
                .collect();

            // Handle case where there are no field validators (only element validators)
            let enum_and_all = if f.validation.field_validators.is_empty() {
                quote! {}
            } else {
                let validator_names: Vec<String> = f
                    .validation.field_validators
                    .iter()
                    .map(|v| v.path_name())
                    .collect();
                let validators_list = validator_names.join("`], `[");
                let field_name_str = field_name.to_string();
                let struct_name_str = struct_name.to_string();
                quote! {
                    #[doc = concat!("Validators for the `", #field_name_str, "` field of [`", #struct_name_str, "`]: [`", #validators_list, "`].")]
                    #[derive(Clone, Debug)]
                    #[allow(dead_code)]
                    pub enum #enum_name #field_enum_definition {
                        #(#enum_variants),*
                    }
                }
            };

            let all_method = if f.validation.field_validators.is_empty() {
                quote! {}
            } else {
                let field_name_str = field_name.to_string();
                quote! {
                    #[doc = concat!("Returns all failed validators for `", #field_name_str, "`.")]
                    pub fn all(&self) -> Vec<#field_enum_path> {
                        let mut result = Vec::new();
                        #(#all_pushes)*
                        result
                    }
                }
            };

            let is_empty_body = if f.validation.field_validators.is_empty() {
                // Only element validators
                quote! { self.element_errors.is_empty() }
            } else {
                quote! { #(#field_is_empty_checks)&&* #element_is_empty_check }
            };

            // Build doc comment for field error struct
            let field_error_struct_doc = {
                let field_name_str = field_name.to_string();
                let struct_name_str = struct_name.to_string();
                if has_element_validators && !f.validation.field_validators.is_empty() {
                    let field_validator_names: Vec<String> = f
                        .validation.field_validators
                        .iter()
                        .map(|v| v.path_name())
                        .collect();
                    let field_validators = field_validator_names.join("`], `[");
                    let element_validator_names: Vec<String> = f
                        .validation.element_validators
                        .iter()
                        .map(|v| v.path_name())
                        .collect();
                    let element_validators = element_validator_names.join("`], `[");
                    quote! {
                        #[doc = concat!("Validation errors for the `", #field_name_str, "` field of [`", #struct_name_str, "`].\n\nField validators: [`", #field_validators, "`]. Element validators: [`", #element_validators, "`].")]
                    }
                } else if has_element_validators {
                    quote! {
                        #[doc = concat!("Validation errors for the `", #field_name_str, "` field of [`", #struct_name_str, "`] (element validators only).")]
                    }
                } else if !f.validation.field_validators.is_empty() {
                    let field_validator_names: Vec<String> = f
                        .validation.field_validators
                        .iter()
                        .map(|v| v.path_name())
                        .collect();
                    let field_validators = field_validator_names.join("`], `[");
                    quote! {
                        #[doc = concat!("Validation errors for the `", #field_name_str, "` field of [`", #struct_name_str, "`].\n\nValidators: [`", #field_validators, "`].")]
                    }
                } else {
                    quote! {}
                }
            };

            // Generate struct fields - need proper comma handling
            let struct_fields = if !f.validation.field_validators.is_empty() && f.has_element_validators() {
                // Both field validators and element errors
                let element_error_path = element_error_path
                    .as_ref()
                    .expect("element validators should have an error path");
                quote! {
                    #(#field_validator_fields,)*
                    element_errors: Vec<(usize, #element_error_path)>
                }
            } else if f.has_element_validators() {
                // Only element errors
                let element_error_path = element_error_path
                    .as_ref()
                    .expect("element validators should have an error path");
                quote! {
                    element_errors: Vec<(usize, #element_error_path)>
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

                #field_error_struct_doc
                #[derive(Clone, Debug, Default)]
                pub struct #field_error_struct_name #field_error_definition {
                    #struct_fields
                }

                impl #field_error_impl_generics #field_error_struct_name #field_error_ty_generics #field_error_where_clause {
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

    let main_error_usages: Vec<TokenStream2> = field_infos
        .iter()
        .map(|f| {
            if f.is_nested() {
                let field_ty = &f.ty;
                let inner_ty = option_inner_type(field_ty).unwrap_or(field_ty);
                if struct_options.is_newtype() && !is_option_type(field_ty) {
                    quote! { <#inner_ty as koruma::ValidateExt>::Error }
                } else {
                    quote! { Option<<#inner_ty as koruma::ValidateExt>::Error> }
                }
            } else {
                field_error_type_path(f)
                    .expect("non-nested fields should have a generated error type")
            }
        })
        .collect();
    let main_error_helper_generics = helper_generics_for_usages(generics, &main_error_usages);
    let main_error_definition = &main_error_helper_generics.definition;
    let main_error_impl_generics = &main_error_helper_generics.impl_generics;
    let main_error_ty_generics = &main_error_helper_generics.ty_generics;
    let main_error_where_clause = &main_error_helper_generics.where_clause;
    let main_error_path = main_error_helper_generics.type_path(&error_struct_name);

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
                if struct_options.is_newtype() && !is_option_type(field_ty) {
                    quote! { #field_name: <#inner_ty as koruma::ValidateExt>::Error }
                } else {
                    quote! { #field_name: Option<<#inner_ty as koruma::ValidateExt>::Error> }
                }
            } else {
                let field_error_path = field_error_type_path(f)
                    .expect("non-nested fields should have a generated error type");
                quote! { #field_name: #field_error_path }
            }
        })
        .collect();

    // Generate getter methods for main error struct
    let getter_methods: Vec<TokenStream2> = field_infos
        .iter()
        .map(|f| {
            let field_name = &f.name;
            let field_name_str = field_name.to_string();
            let struct_name_str = struct_name.to_string();
            if f.is_nested() {
                // For nested fields, return Option<&NestedTypeKorumaValidationError>
                let field_ty = &f.ty;
                let inner_ty = option_inner_type(field_ty).unwrap_or(field_ty);
                if struct_options.is_newtype() && !is_option_type(field_ty) {
                    quote! {
                        #[doc = concat!("Returns validation errors for the nested `", #field_name_str, "` field of [`", #struct_name_str, "`].")]
                        pub fn #field_name(&self) -> &<#inner_ty as koruma::ValidateExt>::Error {
                            &self.#field_name
                        }
                    }
                } else {
                    quote! {
                        #[doc = concat!("Returns validation errors for the nested `", #field_name_str, "` field of [`", #struct_name_str, "`], if any.")]
                        pub fn #field_name(&self) -> Option<&<#inner_ty as koruma::ValidateExt>::Error> {
                            self.#field_name.as_ref()
                        }
                    }
                }
            } else if f.is_newtype() {
                let field_error_path = field_error_type_path(f)
                    .expect("non-nested fields should have a generated error type");
                let has_field_validators = !f.validation.field_validators.is_empty();

                if has_field_validators {
                    // Newtype field with validators - return the wrapper struct
                    // This provides access to both field validators and inner error
                    quote! {
                        #[doc = concat!("Returns validation errors for the `", #field_name_str, "` field of [`", #struct_name_str, "`].")]
                        pub fn #field_name(&self) -> &#field_error_path {
                            &self.#field_name
                        }
                    }
                } else {
                    // For newtype fields without validators, return &InnerError directly for friction-free access
                    // This allows `e.field().all()` directly without needing `?`
                    let field_ty = &f.ty;
                    let inner_ty = option_inner_type(field_ty).unwrap_or(field_ty);
                    if is_option_type(field_ty) {
                        quote! {
                            #[doc = concat!("Returns validation errors for the `", #field_name_str, "` field of [`", #struct_name_str, "`], if any.")]
                            pub fn #field_name(&self) -> Option<&<#inner_ty as koruma::ValidateExt>::Error> {
                                self.#field_name.inner.as_ref()
                            }
                        }
                    } else {
                        quote! {
                            #[doc = concat!("Returns validation errors for the `", #field_name_str, "` field of [`", #struct_name_str, "`].")]
                            pub fn #field_name(&self) -> &<#inner_ty as koruma::ValidateExt>::Error {
                                &self.#field_name.inner
                            }
                        }
                    }
                }
            } else {
                let field_error_path = field_error_type_path(f)
                    .expect("non-nested fields should have a generated error type");
                quote! {
                    #[doc = concat!("Returns validation errors for the `", #field_name_str, "` field of [`", #struct_name_str, "`].")]
                    pub fn #field_name(&self) -> &#field_error_path {
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
                if struct_options.is_newtype() && !is_option_type(&f.ty) {
                    quote! { self.#field_name.is_empty() }
                } else {
                    quote! { self.#field_name.is_none() }
                }
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
                let field_ty = &f.ty;
                let inner_ty = option_inner_type(field_ty).unwrap_or(field_ty);
                if struct_options.is_newtype() && !is_option_type(field_ty) {
                    return quote! {
                        #field_name: <#inner_ty as koruma::ValidateExt>::Error::default()
                    };
                }
                return quote! { #field_name: None };
            }

            let field_error_struct_name = format_ident!(
                "{}{}KorumaValidationError",
                struct_name,
                field_name.to_string().to_upper_camel_case()
            );

            // For newtype fields, generate appropriate defaults
            if f.is_newtype() {
                let has_field_validators = !f.validation.field_validators.is_empty();
                let field_ty = &f.ty;
                let inner_ty = option_inner_type(field_ty).unwrap_or(field_ty);
                let field_is_optional = is_option_type(field_ty);

                if has_field_validators {
                    // Newtype field with validators - generate explicit defaults for field validators
                    let field_validator_defaults: Vec<TokenStream2> = f
                        .validation
                        .field_validators
                        .iter()
                        .map(|v| {
                            let validator_snake =
                                validator_field_ident(v, &f.validation.field_validators);
                            quote! { #validator_snake: None }
                        })
                        .collect();
                    let inner_default = if field_is_optional {
                        quote! { None }
                    } else {
                        quote! { <#inner_ty as koruma::ValidateExt>::Error::default() }
                    };

                    return quote! {
                        #field_name: #field_error_struct_name {
                            #(#field_validator_defaults,)*
                            inner: #inner_default
                        }
                    };
                }

                // Simple newtype field without validators - use Default
                return quote! {
                    #field_name: #field_error_struct_name::default()
                };
            }

            // Generate defaults for field-level validators
            let field_validator_defaults: Vec<TokenStream2> = f
                .validation
                .field_validators
                .iter()
                .map(|v| {
                    let validator_snake = validator_field_ident(v, &f.validation.field_validators);
                    quote! { #validator_snake: None }
                })
                .collect();

            // Handle different combinations of field/element validators
            if f.has_element_validators() && f.validation.field_validators.is_empty() {
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
        .map(|f| -> Result<TokenStream2, syn::Error> {
            let field_name = &f.name;
            let field_member = &f.member;
            let field_ty = &f.ty;

            // Handle nested fields - call validate() on the nested struct
            if f.is_nested() {
                let field_is_optional = is_option_type(field_ty);
                if field_is_optional {
                    // For Option<NestedType>, only validate if Some
                    return Ok(quote! {
                        if let Some(ref __nested_value) = self.#field_member {
                            if let Err(nested_err) = __nested_value.validate() {
                                error.#field_name = Some(nested_err);
                                has_error = true;
                            }
                        }
                    });
                } else {
                    // For non-optional nested field, always validate
                    if struct_options.is_newtype() {
                        return Ok(quote! {
                            if let Err(nested_err) = self.#field_member.validate() {
                                error.#field_name = nested_err;
                                has_error = true;
                            }
                        });
                    }
                    return Ok(quote! {
                        if let Err(nested_err) = self.#field_member.validate() {
                            error.#field_name = Some(nested_err);
                            has_error = true;
                        }
                    });
                }
            }

            // Handle newtype fields - similar to nested but stores in wrapper struct
            if f.is_newtype() {
                let field_is_optional = is_option_type(field_ty);
                let has_field_validators = !f.validation.field_validators.is_empty();
                let set_inner_error = if field_is_optional {
                    quote! { error.#field_name.inner = Some(newtype_err); }
                } else {
                    quote! { error.#field_name.inner = newtype_err; }
                };

                if has_field_validators {
                    // Newtype field with validators - use two-phase validation
                    // First run field validators, then if required check passes, validate inner

                    // Split field validators into full-type and unwrapped
                    let (full_type_validators, unwrapped_validators): (Vec<_>, Vec<_>) = f
                        .validation
                        .field_validators
                        .iter()
                        .partition(|v| validator_wants_full_type(v));

                    // Generate validator check code for newtype fields
                    let generate_newtype_validator_check =
                        |v: &ValidatorAttr,
                         value_expr: TokenStream2,
                         needs_ref: bool|
                         -> Result<TokenStream2, syn::Error> {
                            let validator_snake =
                                validator_field_ident(v, &f.validation.field_validators);
                            let builder_expr =
                                validator_builder_expr(v, field_ty, false, known_field_names)?;

                            let ref_expr = if needs_ref {
                                quote! { &#value_expr }
                            } else {
                                quote! { #value_expr }
                            };

                            if v.uses_type_inference()
                                || v.explicit_type().is_some_and(contains_infer_type)
                            {
                                let assert_fn = format_ident!(
                                    "__koruma_assert_validate_{}_{}_newtype_field",
                                    field_name,
                                    validator_snake
                                );
                                Ok(quote! {
                                    fn #assert_fn<V: koruma::Validate<T>, T>(v: &V, t: &T) -> bool {
                                        v.validate(t)
                                    }
                                    let validator = koruma::BuilderWithValueRef::with_value_ref(
                                        #builder_expr,
                                        #ref_expr,
                                    )
                                    .build();
                                    if !#assert_fn(&validator, #ref_expr) {
                                        error.#field_name.#validator_snake = Some(validator);
                                        has_error = true;
                                    }
                                })
                            } else {
                                Ok(quote! {
                                    let validator = koruma::BuilderWithValueRef::with_value_ref(
                                        #builder_expr,
                                        #ref_expr,
                                    )
                                    .build();
                                    if !validator.validate(#ref_expr) {
                                        error.#field_name.#validator_snake = Some(validator);
                                        has_error = true;
                                    }
                                })
                            }
                        };

                    // Generate checks for full-type validators
                    let full_type_checks: Vec<TokenStream2> = full_type_validators
                        .iter()
                        .map(|v| {
                            generate_newtype_validator_check(v, quote! { self.#field_member }, true)
                        })
                        .collect::<Result<_, _>>()?;

                    // Generate checks for unwrapped validators
                    let unwrapped_checks: Vec<TokenStream2> = unwrapped_validators
                        .iter()
                        .map(|v| {
                            generate_newtype_validator_check(v, quote! { __newtype_value }, false)
                        })
                        .collect::<Result<_, _>>()?;

                    // Build the inner validation logic
                    let inner_validation = if unwrapped_validators.is_empty() {
                        quote! {
                            if let Err(newtype_err) = __newtype_value.validate() {
                                #set_inner_error
                                has_error = true;
                            }
                        }
                    } else {
                        quote! {
                            #(#unwrapped_checks)*
                            if let Err(newtype_err) = __newtype_value.validate() {
                                #set_inner_error
                                has_error = true;
                            }
                        }
                    };

                    if field_is_optional {
                        return Ok(quote! {
                            #(#full_type_checks)*
                            if let Some(ref __newtype_value) = self.#field_member {
                                #inner_validation
                            }
                        });
                    }

                    return Ok(quote! {
                        #(#full_type_checks)*
                        let __newtype_value = &self.#field_member;
                        #inner_validation
                    });
                }

                // Newtype field without validators - simple case
                if field_is_optional {
                    // For Option<NewtypeType>, only validate if Some
                    return Ok(quote! {
                        if let Some(ref __newtype_value) = self.#field_member {
                            if let Err(newtype_err) = __newtype_value.validate() {
                                #set_inner_error
                                has_error = true;
                            }
                        }
                    });
                } else {
                    // For non-optional newtype field, always validate
                    return Ok(quote! {
                        if let Err(newtype_err) = self.#field_member.validate() {
                            #set_inner_error
                            has_error = true;
                        }
                    });
                }
            }

            let has_element_validators = f.has_element_validators();

            // Split field validators into those that want the full type vs those that want
            // the unwrapped type (for Option fields)
            let (full_type_validators, unwrapped_validators): (Vec<_>, Vec<_>) = f
                .validation
                .field_validators
                .iter()
                .partition(|v| validator_wants_full_type(v));

            // Helper to generate validator check code
            let generate_validator_check = |v: &ValidatorAttr,
                                            value_expr: TokenStream2,
                                            needs_ref: bool|
             -> Result<TokenStream2, syn::Error> {
                let validator_snake = validator_field_ident(v, &f.validation.field_validators);
                let builder_expr = validator_builder_expr(v, field_ty, false, known_field_names)?;

                // The reference expression for validate()
                let ref_expr = if needs_ref {
                    quote! { &#value_expr }
                } else {
                    quote! { #value_expr }
                };

                // Determine the validator type
                if v.uses_type_inference() || v.explicit_type().is_some_and(contains_infer_type) {
                    let assert_fn = format_ident!(
                        "__koruma_assert_validate_{}_{}_field",
                        field_name,
                        validator_snake
                    );
                    Ok(quote! {
                        fn #assert_fn<V: koruma::Validate<T>, T>(v: &V, t: &T) -> bool {
                            v.validate(t)
                        }
                        let validator = koruma::BuilderWithValueRef::with_value_ref(
                            #builder_expr,
                            #ref_expr,
                        )
                        .build();
                        if !#assert_fn(&validator, #ref_expr) {
                            error.#field_name.#validator_snake = Some(validator);
                            has_error = true;
                        }
                    })
                } else {
                    Ok(quote! {
                        let validator = koruma::BuilderWithValueRef::with_value_ref(
                            #builder_expr,
                            #ref_expr,
                        )
                        .build();
                        if !validator.validate(#ref_expr) {
                            error.#field_name.#validator_snake = Some(validator);
                            has_error = true;
                        }
                    })
                }
            };

            // Generate checks for full-type validators (use field directly, no reference).
            // The helper turns this into a borrowed value for both builder capture and validate().
            let full_type_checks: Vec<TokenStream2> = full_type_validators
                .iter()
                .map(|v| generate_validator_check(v, quote! { self.#field_member }, true))
                .collect::<Result<_, _>>()?;

            // Generate checks for unwrapped validators (use __field_value which is already a ref)
            let unwrapped_checks: Vec<TokenStream2> = unwrapped_validators
                .iter()
                .map(|v| generate_validator_check(v, quote! { __field_value }, false))
                .collect::<Result<_, _>>()?;

            // Generate element-level validation checks if we have element validators
            let element_validation = if has_element_validators {
                let element_error_struct_name = format_ident!(
                    "{}{}ElementKorumaValidationError",
                    struct_name,
                    field_name.to_string().to_upper_camel_case()
                );

                let field_is_optional = is_option_type(field_ty);
                let element_ty = each_element_type(field_ty);
                let element_is_optional = is_option_type(element_ty);
                let (full_type_element_validators, unwrapped_element_validators): (Vec<_>, Vec<_>) =
                    f.validation
                        .element_validators
                        .iter()
                        .partition(|v| validator_wants_full_type(v));

                let generate_element_validator_check =
                    |v: &ValidatorAttr,
                     value_expr: TokenStream2|
                     -> Result<TokenStream2, syn::Error> {
                        let validator_snake =
                            validator_field_ident(v, &f.validation.element_validators);
                        let builder_expr =
                            validator_builder_expr(v, field_ty, true, known_field_names)?;

                        if v.uses_type_inference()
                            || v.explicit_type().is_some_and(contains_infer_type)
                        {
                            let assert_fn = format_ident!(
                                "__koruma_assert_validate_{}_{}_element",
                                field_name,
                                validator_snake
                            );
                            Ok(quote! {
                                fn #assert_fn<V: koruma::Validate<T>, T>(v: &V, t: &T) -> bool {
                                    v.validate(t)
                                }
                                let validator = koruma::BuilderWithValueRef::with_value_ref(
                                    #builder_expr,
                                    #value_expr,
                                )
                                .build();
                                if !#assert_fn(&validator, #value_expr) {
                                    element_error.#validator_snake = Some(validator);
                                    element_has_error = true;
                                }
                            })
                        } else {
                            Ok(quote! {
                                let validator = koruma::BuilderWithValueRef::with_value_ref(
                                    #builder_expr,
                                    #value_expr,
                                )
                                .build();
                                if !validator.validate(#value_expr) {
                                    element_error.#validator_snake = Some(validator);
                                    element_has_error = true;
                                }
                            })
                        }
                    };

                let full_type_element_checks: Vec<TokenStream2> = full_type_element_validators
                    .iter()
                    .map(|v| generate_element_validator_check(v, quote! { item }))
                    .collect::<Result<_, _>>()?;

                let unwrapped_element_checks: Vec<TokenStream2> = unwrapped_element_validators
                    .iter()
                    .map(|v| generate_element_validator_check(v, quote! { __item_value }))
                    .collect::<Result<_, _>>()?;

                let element_validator_defaults: Vec<TokenStream2> = f
                    .validation
                    .element_validators
                    .iter()
                    .map(|v| {
                        let validator_snake =
                            validator_field_ident(v, &f.validation.element_validators);
                        quote! { #validator_snake: None }
                    })
                    .collect();

                if field_is_optional {
                    let item_iteration = if element_is_optional {
                        // For collections of Option<T>, full-type validators see every element
                        // while unwrapped validators only inspect Some(..) items.
                        quote! {
                            for (idx, item) in __collection_value.iter().enumerate() {
                                let mut element_error = #element_error_struct_name {
                                    #(#element_validator_defaults),*
                                };
                                let mut element_has_error = false;

                                #(#full_type_element_checks)*

                                if let Some(__item_value) = item {
                                    #(#unwrapped_element_checks)*
                                }

                                if element_has_error {
                                    error.#field_name.element_errors.push((idx, element_error));
                                    has_error = true;
                                }
                            }
                        }
                    } else {
                        // For collections of T, validate each item directly.
                        quote! {
                            for (idx, __item_value) in __collection_value.iter().enumerate() {
                                let mut element_error = #element_error_struct_name {
                                    #(#element_validator_defaults),*
                                };
                                let mut element_has_error = false;

                                #(#full_type_element_checks)*
                                #(#unwrapped_element_checks)*

                                if element_has_error {
                                    error.#field_name.element_errors.push((idx, element_error));
                                    has_error = true;
                                }
                            }
                        }
                    };

                    quote! {
                        if let Some(ref __collection_value) = self.#field_member {
                            #item_iteration
                        }
                    }
                } else {
                    if element_is_optional {
                        quote! {
                            for (idx, item) in self.#field_member.iter().enumerate() {
                                let mut element_error = #element_error_struct_name {
                                    #(#element_validator_defaults),*
                                };
                                let mut element_has_error = false;

                                #(#full_type_element_checks)*

                                if let Some(__item_value) = item {
                                    #(#unwrapped_element_checks)*
                                }

                                if element_has_error {
                                    error.#field_name.element_errors.push((idx, element_error));
                                    has_error = true;
                                }
                            }
                        }
                    } else {
                        quote! {
                            for (idx, __item_value) in self.#field_member.iter().enumerate() {
                                let mut element_error = #element_error_struct_name {
                                    #(#element_validator_defaults),*
                                };
                                let mut element_has_error = false;

                                #(#full_type_element_checks)*
                                #(#unwrapped_element_checks)*

                                if element_has_error {
                                    error.#field_name.element_errors.push((idx, element_error));
                                    has_error = true;
                                }
                            }
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
                Ok(quote! {
                    #(#full_type_checks)*
                    if let Some(ref __field_value) = self.#field_member {
                        #(#unwrapped_checks)*
                    }
                    #element_validation
                })
            } else if has_full_type_validators && has_unwrapped_validators {
                // Both types, non-optional field
                Ok(quote! {
                    #(#full_type_checks)*
                    let __field_value = &self.#field_member;
                    #(#unwrapped_checks)*
                    #element_validation
                })
            } else if has_full_type_validators {
                // Only full-type validators
                Ok(quote! {
                    #(#full_type_checks)*
                    #element_validation
                })
            } else if has_unwrapped_validators && field_is_optional {
                // Only unwrapped validators, optional field
                Ok(quote! {
                    if let Some(ref __field_value) = self.#field_member {
                        #(#unwrapped_checks)*
                    }
                    #element_validation
                })
            } else if has_unwrapped_validators {
                // Only unwrapped validators, non-optional field
                Ok(quote! {
                    let __field_value = &self.#field_member;
                    #(#unwrapped_checks)*
                    #element_validation
                })
            } else {
                // No field validators, only element validators
                Ok(element_validation)
            }
        })
        .collect::<Result<_, _>>()?;

    let struct_name_str = struct_name.to_string();

    // Generate try_new function if requested
    let try_new_fn = if struct_options.try_new {
        // Collect all struct fields (not just validated ones) for constructor parameters
        // For named fields, use the ident; for unnamed fields, generate param names like _0, _1
        let all_field_params: Vec<TokenStream2> = fields
            .iter()
            .enumerate()
            .map(|(idx, f)| {
                let name = match &f.ident {
                    Some(ident) => quote::quote! { #ident },
                    None => {
                        let ident =
                            syn::Ident::new(&format!("_{}", idx), proc_macro2::Span::call_site());
                        quote::quote! { #ident }
                    },
                };
                let ty = &f.ty;
                quote! { #name: #ty }
            })
            .collect();

        // Generate appropriate struct initialization based on whether it's a tuple struct
        let struct_init = match fields {
            syn::Fields::Named(_) => {
                // Named fields: Self { field1, field2, ... }
                let all_field_members: Vec<syn::Member> = fields
                    .iter()
                    .enumerate()
                    .map(|(idx, f)| match &f.ident {
                        Some(ident) => syn::Member::Named(ident.clone()),
                        None => syn::Member::Unnamed(syn::Index::from(idx)),
                    })
                    .collect();
                quote! {
                    let instance = Self {
                        #(#all_field_members),*
                    };
                }
            },
            syn::Fields::Unnamed(_) => {
                // Tuple struct: Self(field1, field2, ...)
                let all_field_names: Vec<syn::Ident> = fields
                    .iter()
                    .enumerate()
                    .map(|(idx, f)| match &f.ident {
                        Some(ident) => ident.clone(),
                        None => {
                            syn::Ident::new(&format!("_{}", idx), proc_macro2::Span::call_site())
                        },
                    })
                    .collect();
                quote! {
                    let instance = Self(#(#all_field_names),*);
                }
            },
            syn::Fields::Unit => {
                quote! {
                    let instance = Self;
                }
            },
        };

        quote! {
            #[doc = concat!("Creates a new `", #struct_name_str, "` instance and validates it.\n\nReturns `Ok(instance)` if all validations pass, or `Err(error)` with the validation failures.")]
            pub fn try_new(#(#all_field_params),*) -> Result<Self, #main_error_path> {
                #struct_init
                instance.validate()?;
                Ok(instance)
            }
        }
    } else {
        quote! {}
    };

    // Generate NewtypeValidation marker trait impl for struct-level newtypes
    let newtype_marker_impl = if struct_options.is_newtype() {
        quote! {
            impl #impl_generics koruma::NewtypeValidation for #struct_name #ty_generics #where_clause {}
        }
    } else {
        quote! {}
    };

    // Generate TryFrom<Inner> impl for newtype structs with try_from
    let try_from_impl = if struct_options.try_from() {
        let field_info = struct_newtype_field_info
            .as_ref()
            .expect("newtype(try_from) implies a struct-level newtype field");
        let inner_ty = &field_info.ty;

        // Construct the struct instance based on field type (named vs unnamed)
        let struct_init = match &field_info.member {
            syn::Member::Named(ident) => {
                quote! { Self { #ident: value } }
            },
            syn::Member::Unnamed(_) => {
                quote! { Self(value) }
            },
        };

        quote! {
            impl #impl_generics TryFrom<#inner_ty> for #struct_name #ty_generics #where_clause {
                type Error = #main_error_path;

                fn try_from(value: #inner_ty) -> Result<Self, Self::Error> {
                    let instance = #struct_init;
                    instance.validate()?;
                    Ok(instance)
                }
            }
        }
    } else {
        quote! {}
    };

    // Generate Deref impl for newtype error structs
    let newtype_deref_impl = if struct_options.is_newtype() {
        let field_info = struct_newtype_field_info
            .as_ref()
            .expect("struct-level newtypes should expose one participating field");
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
                // For non-optional nested newtypes, the error struct stores the inner error directly.
                quote! {
                    impl #main_error_impl_generics core::ops::Deref for #error_struct_name #main_error_ty_generics #main_error_where_clause {
                        type Target = <#inner_ty as koruma::ValidateExt>::Error;

                        fn deref(&self) -> &Self::Target {
                            &self.#field_name
                        }
                    }
                }
            }
        } else {
            // For newtypes with validators, deref to the per-field error struct
            let field_error_path = field_error_type_path(field_info)
                .expect("newtype field should have a generated error type");
            quote! {
                impl #main_error_impl_generics core::ops::Deref for #error_struct_name #main_error_ty_generics #main_error_where_clause {
                    type Target = #field_error_path;

                    fn deref(&self) -> &Self::Target {
                        &self.#field_name
                    }
                }
            }
        }
    } else {
        quote! {}
    };

    let field_names: Vec<String> = field_infos.iter().map(|f| f.name.to_string()).collect();
    let fields_doc = if field_names.is_empty() {
        String::new()
    } else if field_names.len() == 1 {
        format!("field: `{}`", field_names[0])
    } else {
        format!("fields: `{}`", field_names.join("`, `"))
    };

    Ok(quote! {
        // Per-field error structs
        #(#field_error_structs)*

        #[doc = concat!("Validation errors for [`", #struct_name_str, "`].\n\nContains per-field error structs for ", #fields_doc, ".")]
        #[derive(Clone, Debug, Default)]
        pub struct #error_struct_name #main_error_definition {
            #(#error_fields),*
        }

        impl #main_error_impl_generics #error_struct_name #main_error_ty_generics #main_error_where_clause {
            #(#getter_methods)*
        }

        #newtype_deref_impl

        impl #main_error_impl_generics koruma::ValidationError for #error_struct_name #main_error_ty_generics #main_error_where_clause {
            fn is_empty(&self) -> bool {
                #is_empty_body
            }
        }

        impl #impl_generics #struct_name #ty_generics #where_clause {
            #try_new_fn

            #[doc = concat!("Validates all fields of `", #struct_name_str, "` and returns an error struct containing all validation failures.")]
            pub fn validate(&self) -> Result<(), #main_error_path> {
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

        impl #impl_generics koruma::ValidateExt for #struct_name #ty_generics #where_clause {
            type Error = #main_error_path;

            fn validate(&self) -> Result<(), #main_error_path> {
                Self::validate(self)
            }
        }

        #newtype_marker_impl

        #try_from_impl
    })
}
