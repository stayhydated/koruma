use crate::expand::derive_shared::validator_builder_expr;
use crate::expand::plan::{
    FieldPlan, PlannedElementValidation, PlannedNestedValidation, PlannedNewtypeValidation,
    PlannedRegularValidation, PlannedValidationOperation, PlannedValidator, TargetBorrow,
    ValidationPlan, ValidationTarget,
};
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::Ident;

pub(crate) struct ValidationCheck<'a> {
    pub validator: &'a PlannedValidator,
    pub target: &'a ValidationTarget,
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
    let validation_target_ty = check.target.validate_type();
    let target_expr = check.target_expr;
    let target_ref = match check.target.borrow() {
        TargetBorrow::Reference => quote! { &#target_expr },
        TargetBorrow::AlreadyBorrowed => quote! { #target_expr },
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
        let validator = #koruma::__private::BuildValidator::build_validator(
            #koruma::__private::CaptureValueRef::capture_value_ref(
                #builder_expr,
                #target_ref,
            )
        );
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
    plan.validation_operations()
        .iter()
        .map(|operation| -> Result<TokenStream2, syn::Error> {
            Ok(render_validation_operation(operation, koruma))
        })
        .collect()
}

fn render_validation_operation(
    operation: &PlannedValidationOperation<'_>,
    koruma: &TokenStream2,
) -> TokenStream2 {
    match operation {
        PlannedValidationOperation::NestedRequired(operation) => {
            render_nested_required_validation(operation)
        },
        PlannedValidationOperation::NestedOptional(operation) => {
            render_nested_optional_validation(operation)
        },
        PlannedValidationOperation::NewtypeRequired(operation) => {
            render_newtype_required_validation(operation, koruma)
        },
        PlannedValidationOperation::NewtypeOptional(operation) => {
            render_newtype_optional_validation(operation, koruma)
        },
        PlannedValidationOperation::RegularRequired(operation) => {
            render_regular_required_validation(operation, koruma)
        },
        PlannedValidationOperation::RegularOptional(operation) => {
            render_regular_optional_validation(operation, koruma)
        },
    }
}

fn render_nested_required_validation(operation: &PlannedNestedValidation<'_>) -> TokenStream2 {
    let field_plan = operation.field;
    let field_name = &field_plan.name;
    let field_member = &field_plan.source.member;

    if operation.direct_storage {
        quote! {
            if let Err(nested_err) = self.#field_member.validate() {
                error.#field_name = nested_err;
                has_error = true;
            }
        }
    } else {
        quote! {
            if let Err(nested_err) = self.#field_member.validate() {
                error.#field_name = Some(nested_err);
                has_error = true;
            }
        }
    }
}

fn render_nested_optional_validation(operation: &PlannedNestedValidation<'_>) -> TokenStream2 {
    let field_plan = operation.field;
    let field_name = &field_plan.name;
    let field_member = &field_plan.source.member;

    quote! {
        if let Some(ref __nested_value) = self.#field_member {
            if let Err(nested_err) = __nested_value.validate() {
                error.#field_name = Some(nested_err);
                has_error = true;
            }
        }
    }
}

fn render_newtype_required_validation(
    operation: &PlannedNewtypeValidation<'_>,
    koruma: &TokenStream2,
) -> TokenStream2 {
    let field_plan = operation.field;
    let field_name = &field_plan.name;
    let field_member = &field_plan.source.member;
    let set_inner_error = quote! { error.#field_name.inner = newtype_err; };

    if operation.field_validators.has_any() {
        let full_type_checks = render_field_validator_checks(
            field_plan,
            &operation.field_validators.full_type_validators,
            quote! { self.#field_member },
            koruma,
        );
        let unwrapped_checks = render_field_validator_checks(
            field_plan,
            &operation.field_validators.unwrapped_validators,
            quote! { __newtype_value },
            koruma,
        );
        let inner_validation = quote! {
            #(#unwrapped_checks)*
            if let Err(newtype_err) = __newtype_value.validate() {
                #set_inner_error
                has_error = true;
            }
        };

        return quote! {
            #(#full_type_checks)*
            let __newtype_value = &self.#field_member;
            #inner_validation
        };
    }

    quote! {
        if let Err(newtype_err) = self.#field_member.validate() {
            #set_inner_error
            has_error = true;
        }
    }
}

fn render_newtype_optional_validation(
    operation: &PlannedNewtypeValidation<'_>,
    koruma: &TokenStream2,
) -> TokenStream2 {
    let field_plan = operation.field;
    let field_name = &field_plan.name;
    let field_member = &field_plan.source.member;
    let set_inner_error = quote! { error.#field_name.inner = Some(newtype_err); };

    if operation.field_validators.has_any() {
        let full_type_checks = render_field_validator_checks(
            field_plan,
            &operation.field_validators.full_type_validators,
            quote! { self.#field_member },
            koruma,
        );
        let unwrapped_checks = render_field_validator_checks(
            field_plan,
            &operation.field_validators.unwrapped_validators,
            quote! { __newtype_value },
            koruma,
        );

        return quote! {
            #(#full_type_checks)*
            if let Some(ref __newtype_value) = self.#field_member {
                #(#unwrapped_checks)*
                if let Err(newtype_err) = __newtype_value.validate() {
                    #set_inner_error
                    has_error = true;
                }
            }
        };
    }

    quote! {
        if let Some(ref __newtype_value) = self.#field_member {
            if let Err(newtype_err) = __newtype_value.validate() {
                #set_inner_error
                has_error = true;
            }
        }
    }
}

fn render_regular_required_validation(
    operation: &PlannedRegularValidation<'_>,
    koruma: &TokenStream2,
) -> TokenStream2 {
    let field_plan = operation.field;
    let field_member = &field_plan.source.member;
    let field_validators = &operation.field_validators;
    let full_type_checks = render_field_validator_checks(
        field_plan,
        &field_validators.full_type_validators,
        quote! { self.#field_member },
        koruma,
    );
    let unwrapped_checks = render_field_validator_checks(
        field_plan,
        &field_validators.unwrapped_validators,
        quote! { __field_value },
        koruma,
    );
    let element_validation = operation
        .element_validators
        .as_ref()
        .map(|element| render_element_validation(field_plan, element, false, koruma))
        .unwrap_or_else(|| quote! {});

    match (
        field_validators.has_full_type_validators(),
        field_validators.has_unwrapped_validators(),
    ) {
        (true, true) => quote! {
            #(#full_type_checks)*
            let __field_value = &self.#field_member;
            #(#unwrapped_checks)*
            #element_validation
        },
        (true, false) => quote! {
            #(#full_type_checks)*
            #element_validation
        },
        (false, true) => quote! {
            let __field_value = &self.#field_member;
            #(#unwrapped_checks)*
            #element_validation
        },
        (false, false) => element_validation,
    }
}

fn render_regular_optional_validation(
    operation: &PlannedRegularValidation<'_>,
    koruma: &TokenStream2,
) -> TokenStream2 {
    let field_plan = operation.field;
    let field_member = &field_plan.source.member;
    let field_validators = &operation.field_validators;
    let full_type_checks = render_field_validator_checks(
        field_plan,
        &field_validators.full_type_validators,
        quote! { self.#field_member },
        koruma,
    );
    let unwrapped_checks = render_field_validator_checks(
        field_plan,
        &field_validators.unwrapped_validators,
        quote! { __field_value },
        koruma,
    );
    let element_validation = operation
        .element_validators
        .as_ref()
        .map(|element| render_element_validation(field_plan, element, true, koruma))
        .unwrap_or_else(|| quote! {});

    match (
        field_validators.has_full_type_validators(),
        field_validators.has_unwrapped_validators(),
    ) {
        (true, true) => quote! {
            #(#full_type_checks)*
            if let Some(ref __field_value) = self.#field_member {
                #(#unwrapped_checks)*
            }
            #element_validation
        },
        (true, false) => quote! {
            #(#full_type_checks)*
            #element_validation
        },
        (false, true) => quote! {
            if let Some(ref __field_value) = self.#field_member {
                #(#unwrapped_checks)*
            }
            #element_validation
        },
        (false, false) => element_validation,
    }
}

fn render_field_validator_checks(
    field_plan: &FieldPlan,
    validators: &[&PlannedValidator],
    target_expr: TokenStream2,
    koruma: &TokenStream2,
) -> Vec<TokenStream2> {
    let field_name = &field_plan.name;
    validators
        .iter()
        .map(|validator| {
            render_validation_check(
                ValidationCheck {
                    validator,
                    target: &validator.target,
                    target_expr: target_expr.clone(),
                    sink: ErrorSink::FieldValidator {
                        field: field_name,
                        slot: &validator.field_ident,
                    },
                },
                koruma,
            )
        })
        .collect()
}

fn render_element_validation(
    field_plan: &FieldPlan,
    element: &PlannedElementValidation<'_>,
    field_optional: bool,
    koruma: &TokenStream2,
) -> TokenStream2 {
    let field_name = &field_plan.name;
    let field_member = &field_plan.source.member;
    let element_error_struct_name = &field_plan.generated_names.element_error_struct;
    let groups = element.groups();
    let full_element_target_expr = match element {
        PlannedElementValidation::RequiredElement(_) => quote! { __item_value },
        PlannedElementValidation::OptionalElement(_) => quote! { item },
    };
    let full_type_element_checks = render_element_validator_checks(
        &groups.full_type_validators,
        full_element_target_expr,
        koruma,
    );
    let unwrapped_element_checks = render_element_validator_checks(
        &groups.unwrapped_validators,
        quote! { __item_value },
        koruma,
    );
    let element_validator_defaults: Vec<TokenStream2> = field_plan
        .element_validators()
        .iter()
        .map(|validator| {
            let validator_snake = &validator.field_ident;
            quote! { #validator_snake: None }
        })
        .collect();

    match (field_optional, element) {
        (true, PlannedElementValidation::OptionalElement(_)) => quote! {
            if let Some(ref __collection_value) = self.#field_member {
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
        },
        (true, PlannedElementValidation::RequiredElement(_)) => quote! {
            if let Some(ref __collection_value) = self.#field_member {
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
        },
        (false, PlannedElementValidation::OptionalElement(_)) => quote! {
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
        },
        (false, PlannedElementValidation::RequiredElement(_)) => quote! {
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
        },
    }
}

fn render_element_validator_checks(
    validators: &[&PlannedValidator],
    target_expr: TokenStream2,
    koruma: &TokenStream2,
) -> Vec<TokenStream2> {
    validators
        .iter()
        .map(|validator| {
            render_validation_check(
                ValidationCheck {
                    validator,
                    target: &validator.target,
                    target_expr: target_expr.clone(),
                    sink: ErrorSink::ElementValidator {
                        slot: &validator.field_ident,
                    },
                },
                koruma,
            )
        })
        .collect()
}
