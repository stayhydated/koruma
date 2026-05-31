use crate::expand::derive_shared::validator_builder_expr;
use crate::expand::plan::{PlannedValidator, TargetAccess, ValidationPlan};
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::Ident;

pub(crate) struct ValidationCheck<'a> {
    pub validator: &'a PlannedValidator,
    pub target_expr: TokenStream2,
    pub sink: ErrorSink<'a>,
}

pub(crate) enum ErrorSink<'a> {
    FieldValidator { field: &'a Ident, slot: &'a Ident },
    ElementValidator { slot: &'a Ident },
}

fn render_validation_check(check: ValidationCheck<'_>, koruma: &TokenStream2) -> TokenStream2 {
    let validator = check.validator;
    let builder_expr = validator_builder_expr(validator);
    let validator_ty = &validator.validator_type;
    let validation_target_ty = &validator.target.validate_type;
    let target_expr = check.target_expr;
    let target_ref = match validator.target.access {
        TargetAccess::AlreadyBorrowedLocal => quote! { #target_expr },
        TargetAccess::BorrowField | TargetAccess::BorrowLocal => quote! { &#target_expr },
    };
    let error_assignment = match check.sink {
        ErrorSink::FieldValidator { field, slot } => {
            quote! {
                error.#field.#slot = Some(validator);
                has_error = true;
            }
        },
        ErrorSink::ElementValidator { slot } => {
            quote! {
                element_error.#slot = Some(validator);
                element_has_error = true;
            }
        },
    };

    quote! {
        let validator = #koruma::CaptureValueRef::capture_value_ref(
            #builder_expr,
            #target_ref,
        )
        .build();
        if !<#validator_ty as #koruma::Validate<#validation_target_ty>>::validate(
            &validator,
            #target_ref,
        ) {
            #error_assignment
        }
    }
}

pub(crate) fn render_validation_checks(
    plan: &ValidationPlan,
    koruma: &TokenStream2,
) -> Result<Vec<TokenStream2>, syn::Error> {
    let struct_is_newtype = plan.struct_newtype().is_some();

    plan.fields
        .iter()
        .map(|field_plan| -> Result<TokenStream2, syn::Error> {
            let field_name = &field_plan.name;
            let field_member = &field_plan.source.member;

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
                    if struct_is_newtype {
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

                    let full_type_checks: Vec<TokenStream2> = full_type_validators
                        .iter()
                        .map(|v| {
                            render_validation_check(
                                ValidationCheck {
                                    validator: v,
                                    target_expr: quote! { self.#field_member },
                                    sink: ErrorSink::FieldValidator {
                                        field: field_name,
                                        slot: &v.field_ident,
                                    },
                                },
                                koruma,
                            )
                        })
                        .collect();

                    let unwrapped_checks: Vec<TokenStream2> = unwrapped_validators
                        .iter()
                        .map(|v| {
                            render_validation_check(
                                ValidationCheck {
                                    validator: v,
                                    target_expr: quote! { __newtype_value },
                                    sink: ErrorSink::FieldValidator {
                                        field: field_name,
                                        slot: &v.field_ident,
                                    },
                                },
                                koruma,
                            )
                        })
                        .collect();

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

            let full_type_checks: Vec<TokenStream2> = full_type_validators
                .iter()
                .map(|v| {
                    render_validation_check(
                        ValidationCheck {
                            validator: v,
                            target_expr: quote! { self.#field_member },
                            sink: ErrorSink::FieldValidator {
                                field: field_name,
                                slot: &v.field_ident,
                            },
                        },
                        koruma,
                    )
                })
                .collect();

            let unwrapped_checks: Vec<TokenStream2> = unwrapped_validators
                .iter()
                .map(|v| {
                    render_validation_check(
                        ValidationCheck {
                            validator: v,
                            target_expr: quote! { __field_value },
                            sink: ErrorSink::FieldValidator {
                                field: field_name,
                                slot: &v.field_ident,
                            },
                        },
                        koruma,
                    )
                })
                .collect();

            let element_validation = if has_element_validators {
                let element_error_struct_name = &field_plan.generated_names.element_error_struct;

                let field_is_optional = field_plan.field_optional();
                let element_is_optional = field_plan.element_optional();
                let full_type_element_validators: Vec<_> =
                    field_plan.full_element_validators().collect();
                let unwrapped_element_validators: Vec<_> =
                    field_plan.unwrapped_element_validators().collect();

                let full_type_element_checks: Vec<TokenStream2> = full_type_element_validators
                    .iter()
                    .map(|v| {
                        render_validation_check(
                            ValidationCheck {
                                validator: v,
                                target_expr: quote! { item },
                                sink: ErrorSink::ElementValidator {
                                    slot: &v.field_ident,
                                },
                            },
                            koruma,
                        )
                    })
                    .collect();

                let unwrapped_element_checks: Vec<TokenStream2> = unwrapped_element_validators
                    .iter()
                    .map(|v| {
                        render_validation_check(
                            ValidationCheck {
                                validator: v,
                                target_expr: quote! { __item_value },
                                sink: ErrorSink::ElementValidator {
                                    slot: &v.field_ident,
                                },
                            },
                            koruma,
                        )
                    })
                    .collect();

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
