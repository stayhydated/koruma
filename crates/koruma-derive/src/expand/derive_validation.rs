use crate::expand::derive_shared::validator_builder_expr;
use crate::expand::plan::{PlannedValidator, ValidationPlan};
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

pub(crate) fn render_validation_checks(
    plan: &ValidationPlan,
    koruma: &TokenStream2,
) -> Result<Vec<TokenStream2>, syn::Error> {
    let struct_options = &plan.struct_options;

    plan.field_infos()
        .iter()
        .zip(plan.fields.iter())
        .map(|(f, field_plan)| -> Result<TokenStream2, syn::Error> {
            let field_name = &f.name;
            let field_member = &f.member;

            if field_plan.is_nested() {
                let field_is_optional = field_plan.field_optional();
                if field_is_optional {
                    return Ok(quote! {
                        if let Some(ref __nested_value) = self.#field_member {
                            if let Err(nested_err) = __nested_value.validate() {
                                error.#field_name = Some(nested_err);
                                has_error = true;
                            }
                        }
                    });
                } else {
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

            if field_plan.is_newtype() {
                let field_is_optional = field_plan.field_optional();
                let has_field_validators = field_plan.has_field_validators();
                let set_inner_error = if field_is_optional {
                    quote! { error.#field_name.inner = Some(newtype_err); }
                } else {
                    quote! { error.#field_name.inner = newtype_err; }
                };

                if has_field_validators {
                    let full_type_validators: Vec<_> = field_plan.full_field_validators().collect();
                    let unwrapped_validators: Vec<_> =
                        field_plan.unwrapped_field_validators().collect();

                    let generate_newtype_validator_check =
                        |v: &PlannedValidator,
                         value_expr: TokenStream2,
                         needs_ref: bool|
                         -> Result<TokenStream2, syn::Error> {
                            let validator_snake = &v.field_ident;
                            let builder_expr = validator_builder_expr(v);
                            let validator_ty = &v.validator_type;
                            let validation_target_ty = &v.validation_target_type;

                            let ref_expr = if needs_ref {
                                quote! { &#value_expr }
                            } else {
                                quote! { #value_expr }
                            };

                            Ok(quote! {
                                let validator = #koruma::BuilderWithValueRef::with_value_ref(
                                    #builder_expr,
                                    #ref_expr,
                                )
                                .build();
                                if !<#validator_ty as #koruma::Validate<#validation_target_ty>>::validate(
                                    &validator,
                                    #ref_expr,
                                ) {
                                    error.#field_name.#validator_snake = Some(validator);
                                    has_error = true;
                                }
                            })
                        };

                    let full_type_checks: Vec<TokenStream2> = full_type_validators
                        .iter()
                        .map(|v| {
                            generate_newtype_validator_check(v, quote! { self.#field_member }, true)
                        })
                        .collect::<Result<_, _>>()?;

                    let unwrapped_checks: Vec<TokenStream2> = unwrapped_validators
                        .iter()
                        .map(|v| {
                            generate_newtype_validator_check(v, quote! { __newtype_value }, false)
                        })
                        .collect::<Result<_, _>>()?;

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

                if field_is_optional {
                    return Ok(quote! {
                        if let Some(ref __newtype_value) = self.#field_member {
                            if let Err(newtype_err) = __newtype_value.validate() {
                                #set_inner_error
                                has_error = true;
                            }
                        }
                    });
                } else {
                    return Ok(quote! {
                        if let Err(newtype_err) = self.#field_member.validate() {
                            #set_inner_error
                            has_error = true;
                        }
                    });
                }
            }

            let has_element_validators = field_plan.has_element_validators();
            let full_type_validators: Vec<_> = field_plan.full_field_validators().collect();
            let unwrapped_validators: Vec<_> = field_plan.unwrapped_field_validators().collect();

            let generate_validator_check = |v: &PlannedValidator,
                                            value_expr: TokenStream2,
                                            needs_ref: bool|
             -> Result<TokenStream2, syn::Error> {
                let validator_snake = &v.field_ident;
                let builder_expr = validator_builder_expr(v);
                let validator_ty = &v.validator_type;
                let validation_target_ty = &v.validation_target_type;

                let ref_expr = if needs_ref {
                    quote! { &#value_expr }
                } else {
                    quote! { #value_expr }
                };

                Ok(quote! {
                    let validator = #koruma::BuilderWithValueRef::with_value_ref(
                        #builder_expr,
                        #ref_expr,
                    )
                    .build();
                    if !<#validator_ty as #koruma::Validate<#validation_target_ty>>::validate(
                        &validator,
                        #ref_expr,
                    ) {
                        error.#field_name.#validator_snake = Some(validator);
                        has_error = true;
                    }
                })
            };

            let full_type_checks: Vec<TokenStream2> = full_type_validators
                .iter()
                .map(|v| generate_validator_check(v, quote! { self.#field_member }, true))
                .collect::<Result<_, _>>()?;

            let unwrapped_checks: Vec<TokenStream2> = unwrapped_validators
                .iter()
                .map(|v| generate_validator_check(v, quote! { __field_value }, false))
                .collect::<Result<_, _>>()?;

            let element_validation = if has_element_validators {
                let element_error_struct_name = &field_plan.generated_names.element_error_struct;

                let field_is_optional = field_plan.field_optional();
                let element_is_optional = field_plan.element_optional();
                let full_type_element_validators: Vec<_> =
                    field_plan.full_element_validators().collect();
                let unwrapped_element_validators: Vec<_> =
                    field_plan.unwrapped_element_validators().collect();

                let generate_element_validator_check =
                    |v: &PlannedValidator,
                     value_expr: TokenStream2|
                     -> Result<TokenStream2, syn::Error> {
                        let validator_snake = &v.field_ident;
                        let builder_expr = validator_builder_expr(v);
                        let validator_ty = &v.validator_type;
                        let validation_target_ty = &v.validation_target_type;

                        Ok(quote! {
                            let validator = #koruma::BuilderWithValueRef::with_value_ref(
                                #builder_expr,
                                #value_expr,
                            )
                            .build();
                            if !<#validator_ty as #koruma::Validate<#validation_target_ty>>::validate(
                                &validator,
                                #value_expr,
                            ) {
                                element_error.#validator_snake = Some(validator);
                                element_has_error = true;
                            }
                        })
                    };

                let full_type_element_checks: Vec<TokenStream2> = full_type_element_validators
                    .iter()
                    .map(|v| generate_element_validator_check(v, quote! { item }))
                    .collect::<Result<_, _>>()?;

                let unwrapped_element_checks: Vec<TokenStream2> = unwrapped_element_validators
                    .iter()
                    .map(|v| generate_element_validator_check(v, quote! { __item_value }))
                    .collect::<Result<_, _>>()?;

                let element_validator_defaults: Vec<TokenStream2> = field_plan
                    .element_validators()
                    .iter()
                    .map(|v| {
                        let validator_snake = &v.field_ident;
                        quote! { #validator_snake: None }
                    })
                    .collect();

                if field_is_optional {
                    let item_iteration = if element_is_optional {
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
                } else if element_is_optional {
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
            } else {
                quote! {}
            };

            let field_is_optional = field_plan.field_optional();
            let has_full_type_validators = !full_type_validators.is_empty();
            let has_unwrapped_validators = !unwrapped_validators.is_empty();

            if has_full_type_validators && has_unwrapped_validators && field_is_optional {
                Ok(quote! {
                    #(#full_type_checks)*
                    if let Some(ref __field_value) = self.#field_member {
                        #(#unwrapped_checks)*
                    }
                    #element_validation
                })
            } else if has_full_type_validators && has_unwrapped_validators {
                Ok(quote! {
                    #(#full_type_checks)*
                    let __field_value = &self.#field_member;
                    #(#unwrapped_checks)*
                    #element_validation
                })
            } else if has_full_type_validators {
                Ok(quote! {
                    #(#full_type_checks)*
                    #element_validation
                })
            } else if has_unwrapped_validators && field_is_optional {
                Ok(quote! {
                    if let Some(ref __field_value) = self.#field_member {
                        #(#unwrapped_checks)*
                    }
                    #element_validation
                })
            } else if has_unwrapped_validators {
                Ok(quote! {
                    let __field_value = &self.#field_member;
                    #(#unwrapped_checks)*
                    #element_validation
                })
            } else {
                Ok(element_validation)
            }
        })
        .collect()
}
