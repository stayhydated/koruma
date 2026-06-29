use crate::expand::derive_shared::validator_builder_expr;
use crate::expand::plan::{
    FieldPlan, PlannedElementValidation, PlannedFieldBinding, PlannedNestedValidation,
    PlannedNewtypeValidation, PlannedRegularValidation, PlannedValidationOperation,
    PlannedValidator, TargetBorrow, ValidationPlan, ValidationRenderPlan, ValidationTarget,
};
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, quote_spanned};
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

#[derive(Clone, Copy)]
pub(crate) enum ValidationFieldAccess {
    StructField,
    NewtypeInnerValue,
}

impl ValidationFieldAccess {
    fn field_expr(self, field_plan: &FieldPlan) -> TokenStream2 {
        match self {
            Self::StructField => {
                let field_member = &field_plan.source.member;
                quote! { self.#field_member }
            },
            Self::NewtypeInnerValue => quote! { *__koruma_newtype_inner_value },
        }
    }

    fn field_ref_expr(self, field_plan: &FieldPlan) -> TokenStream2 {
        match self {
            Self::StructField => {
                let field_member = &field_plan.source.member;
                quote! { &self.#field_member }
            },
            Self::NewtypeInnerValue => quote! { __koruma_newtype_inner_value },
        }
    }

    fn field_iter_expr(self, field_plan: &FieldPlan) -> TokenStream2 {
        match self {
            Self::StructField => {
                let field_member = &field_plan.source.member;
                quote! { self.#field_member.iter() }
            },
            Self::NewtypeInnerValue => quote! { __koruma_newtype_inner_value.iter() },
        }
    }
}

fn render_validation_check(check: ValidationCheck<'_>, koruma: &TokenStream2) -> TokenStream2 {
    let validator = check.validator;
    let builder_expr = validator_builder_expr(validator);
    let validator_ty = &validator.validator_type;
    let validation_target_ty = check.target.validate_type();
    let target_expr = check.target_expr;
    let completion_probe = if check.validator.attr.has_completion_probe() {
        quote! {
            {
                use #koruma::__private::RustAnalyzerCompletionMarker as _;
                let _ = __koruma_builder.__koruma_ra_completion_marker();
            }
        }
    } else {
        quote! {}
    };
    let target_ref = match check.target.borrow() {
        TargetBorrow::Reference => quote! { &#target_expr },
        TargetBorrow::AlreadyBorrowed => quote! { #target_expr },
    };
    let readiness_assertion = match &check.sink {
        ErrorSink::FieldValidator { .. } => quote! { assert_field_validator_ready },
        ErrorSink::ElementValidator { .. } => quote! { assert_element_validator_ready },
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

    let source_span = validator.source_span;

    quote_spanned! {source_span=>
        {
            let __koruma_builder = #builder_expr;
            #completion_probe
            #koruma::__private::#readiness_assertion::<
                _,
                #validation_target_ty,
                #validator_ty,
            >(&__koruma_builder);

            let validator = #koruma::__private::BuildValidator::build_validator(
                #koruma::__private::CaptureValueRef::capture_value_ref(
                    __koruma_builder,
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
}

pub(crate) fn render_validation_checks(
    plan: &ValidationPlan,
    koruma: &TokenStream2,
) -> Result<Vec<TokenStream2>, syn::Error> {
    let render_plan = plan.validation_render_plan();
    render_validation_render_plan(&render_plan, koruma, ValidationFieldAccess::StructField)
}

pub(crate) fn render_validation_checks_for_newtype_inner(
    plan: &ValidationPlan,
    koruma: &TokenStream2,
) -> Result<Vec<TokenStream2>, syn::Error> {
    let render_plan = plan.validation_render_plan();
    render_validation_render_plan(
        &render_plan,
        koruma,
        ValidationFieldAccess::NewtypeInnerValue,
    )
}

fn render_validation_render_plan(
    render_plan: &ValidationRenderPlan<'_>,
    koruma: &TokenStream2,
    access: ValidationFieldAccess,
) -> Result<Vec<TokenStream2>, syn::Error> {
    render_plan
        .operations
        .iter()
        .map(|operation| -> Result<TokenStream2, syn::Error> {
            Ok(render_validation_operation(operation, koruma, access))
        })
        .collect()
}

fn render_validation_operation(
    operation: &PlannedValidationOperation<'_>,
    koruma: &TokenStream2,
    access: ValidationFieldAccess,
) -> TokenStream2 {
    match operation {
        PlannedValidationOperation::NestedRequired(operation) => {
            render_nested_required_validation(operation, koruma, access)
        },
        PlannedValidationOperation::NestedOptional(operation) => {
            render_nested_optional_validation(operation, koruma, access)
        },
        PlannedValidationOperation::NewtypeRequired(operation) => {
            render_newtype_required_validation(operation, koruma, access)
        },
        PlannedValidationOperation::NewtypeOptional(operation) => {
            render_newtype_optional_validation(operation, koruma, access)
        },
        PlannedValidationOperation::RegularRequired(operation) => {
            render_regular_required_validation(operation, koruma, access)
        },
        PlannedValidationOperation::RegularOptional(operation) => {
            render_regular_optional_validation(operation, koruma, access)
        },
    }
}

fn render_nested_assertion(field_plan: &FieldPlan, koruma: &TokenStream2) -> TokenStream2 {
    let span = field_plan
        .source
        .marker_span
        .unwrap_or_else(|| field_plan.name.span());
    let inner_ty = field_plan.inner_type();
    quote_spanned! {span=>
        {
            #koruma::__private::assert_nested_validation_ready::<#inner_ty>();
        }
    }
}

fn render_newtype_assertion(field_plan: &FieldPlan, koruma: &TokenStream2) -> TokenStream2 {
    let span = field_plan
        .source
        .marker_span
        .unwrap_or_else(|| field_plan.name.span());
    let inner_ty = field_plan.inner_type();
    quote_spanned! {span=>
        {
            #koruma::__private::assert_newtype_field_ready::<#inner_ty>();
        }
    }
}

fn render_each_collection_assertion(field_plan: &FieldPlan, koruma: &TokenStream2) -> TokenStream2 {
    let field_ty = &field_plan.source.ty;
    let Some(element_ty) = field_plan.element_type() else {
        return quote! {};
    };
    let assertion = if field_plan.field_optional() {
        quote! { assert_optional_each_collection_ref }
    } else {
        quote! { assert_each_collection_ref }
    };
    let span = field_plan
        .element_validators()
        .first()
        .map(|validator| validator.source_span)
        .unwrap_or_else(|| field_plan.name.span());

    quote_spanned! {span=>
        {
            #koruma::__private::#assertion::<#field_ty, #element_ty>();
        }
    }
}

fn render_nested_required_validation(
    operation: &PlannedNestedValidation<'_>,
    koruma: &TokenStream2,
    access: ValidationFieldAccess,
) -> TokenStream2 {
    let field_plan = operation.field;
    let field_name = &field_plan.name;
    let nested_assertion = render_nested_assertion(field_plan, koruma);
    let field_ref = access.field_ref_expr(field_plan);

    if operation.direct_storage {
        quote! {
            #nested_assertion
            if let Err(nested_err) = (#field_ref).validate() {
                error.#field_name = nested_err;
                has_error = true;
            }
        }
    } else {
        quote! {
            #nested_assertion
            if let Err(nested_err) = (#field_ref).validate() {
                error.#field_name = Some(nested_err);
                has_error = true;
            }
        }
    }
}

fn render_nested_optional_validation(
    operation: &PlannedNestedValidation<'_>,
    koruma: &TokenStream2,
    access: ValidationFieldAccess,
) -> TokenStream2 {
    let field_plan = operation.field;
    let field_name = &field_plan.name;
    let nested_assertion = render_nested_assertion(field_plan, koruma);
    let field_expr = access.field_expr(field_plan);

    quote! {
        #nested_assertion
        if let Some(ref __nested_value) = #field_expr {
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
    access: ValidationFieldAccess,
) -> TokenStream2 {
    let field_plan = operation.field;
    let field_name = &field_plan.name;
    let set_inner_error = quote! { error.#field_name.inner = newtype_err; };
    let newtype_assertion = render_newtype_assertion(field_plan, koruma);
    let field_expr = access.field_expr(field_plan);
    let field_ref = access.field_ref_expr(field_plan);

    if operation.field_validators.has_any() {
        let full_type_checks = render_field_validator_checks(
            field_plan,
            &operation.field_validators.full_type_validators,
            field_expr,
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
            #newtype_assertion
            #(#full_type_checks)*
            let __newtype_value = #field_ref;
            #inner_validation
        };
    }

    quote! {
        #newtype_assertion
        if let Err(newtype_err) = (#field_ref).validate() {
            #set_inner_error
            has_error = true;
        }
    }
}

fn render_newtype_optional_validation(
    operation: &PlannedNewtypeValidation<'_>,
    koruma: &TokenStream2,
    access: ValidationFieldAccess,
) -> TokenStream2 {
    let field_plan = operation.field;
    let field_name = &field_plan.name;
    let set_inner_error = quote! { error.#field_name.inner = Some(newtype_err); };
    let newtype_assertion = render_newtype_assertion(field_plan, koruma);
    let field_expr = access.field_expr(field_plan);

    if operation.field_validators.has_any() {
        let full_type_checks = render_field_validator_checks(
            field_plan,
            &operation.field_validators.full_type_validators,
            field_expr.clone(),
            koruma,
        );
        let unwrapped_checks = render_field_validator_checks(
            field_plan,
            &operation.field_validators.unwrapped_validators,
            quote! { __newtype_value },
            koruma,
        );

        return quote! {
            #newtype_assertion
            #(#full_type_checks)*
            if let Some(ref __newtype_value) = #field_expr {
                #(#unwrapped_checks)*
                if let Err(newtype_err) = __newtype_value.validate() {
                    #set_inner_error
                    has_error = true;
                }
            }
        };
    }

    quote! {
        #newtype_assertion
        if let Some(ref __newtype_value) = #field_expr {
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
    access: ValidationFieldAccess,
) -> TokenStream2 {
    let field_plan = operation.field;
    let element_validation = operation
        .element_validators
        .as_ref()
        .map(|element| render_element_validation(field_plan, element, koruma, access))
        .unwrap_or_else(|| quote! {});
    let field_expr = access.field_expr(field_plan);
    let field_ref = access.field_ref_expr(field_plan);

    match operation.field_validators.binding() {
        PlannedFieldBinding::FullAndUnwrapped {
            full_type_validators,
            unwrapped_validators,
        } => {
            let full_type_checks = render_field_validator_checks(
                field_plan,
                &full_type_validators,
                field_expr,
                koruma,
            );
            let unwrapped_checks = render_field_validator_checks(
                field_plan,
                &unwrapped_validators,
                quote! { __field_value },
                koruma,
            );
            quote! {
            #(#full_type_checks)*
            let __field_value = #field_ref;
            #(#unwrapped_checks)*
            #element_validation
            }
        },
        PlannedFieldBinding::FullOnly {
            full_type_validators,
        } => {
            let full_type_checks = render_field_validator_checks(
                field_plan,
                &full_type_validators,
                field_expr,
                koruma,
            );
            quote! {
            #(#full_type_checks)*
            #element_validation
            }
        },
        PlannedFieldBinding::UnwrappedOnly {
            unwrapped_validators,
        } => {
            let unwrapped_checks = render_field_validator_checks(
                field_plan,
                &unwrapped_validators,
                quote! { __field_value },
                koruma,
            );
            quote! {
            let __field_value = #field_ref;
            #(#unwrapped_checks)*
            #element_validation
            }
        },
        PlannedFieldBinding::NoValidators => element_validation,
    }
}

fn render_regular_optional_validation(
    operation: &PlannedRegularValidation<'_>,
    koruma: &TokenStream2,
    access: ValidationFieldAccess,
) -> TokenStream2 {
    let field_plan = operation.field;
    let element_validation = operation
        .element_validators
        .as_ref()
        .map(|element| render_element_validation(field_plan, element, koruma, access))
        .unwrap_or_else(|| quote! {});
    let field_expr = access.field_expr(field_plan);

    match operation.field_validators.binding() {
        PlannedFieldBinding::FullAndUnwrapped {
            full_type_validators,
            unwrapped_validators,
        } => {
            let full_type_checks = render_field_validator_checks(
                field_plan,
                &full_type_validators,
                field_expr.clone(),
                koruma,
            );
            let unwrapped_checks = render_field_validator_checks(
                field_plan,
                &unwrapped_validators,
                quote! { __field_value },
                koruma,
            );
            quote! {
            #(#full_type_checks)*
            if let Some(ref __field_value) = #field_expr {
                #(#unwrapped_checks)*
            }
            #element_validation
            }
        },
        PlannedFieldBinding::FullOnly {
            full_type_validators,
        } => {
            let full_type_checks = render_field_validator_checks(
                field_plan,
                &full_type_validators,
                field_expr,
                koruma,
            );
            quote! {
            #(#full_type_checks)*
            #element_validation
            }
        },
        PlannedFieldBinding::UnwrappedOnly {
            unwrapped_validators,
        } => {
            let unwrapped_checks = render_field_validator_checks(
                field_plan,
                &unwrapped_validators,
                quote! { __field_value },
                koruma,
            );
            quote! {
            if let Some(ref __field_value) = #field_expr {
                #(#unwrapped_checks)*
            }
            #element_validation
            }
        },
        PlannedFieldBinding::NoValidators => element_validation,
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
    koruma: &TokenStream2,
    access: ValidationFieldAccess,
) -> TokenStream2 {
    let field_name = &field_plan.name;
    let element_error_struct_name = &field_plan.generated_names.element_error_struct;
    let groups = element.groups();
    let full_element_target_expr = match element {
        PlannedElementValidation::RequiredCollectionRequired(_)
        | PlannedElementValidation::OptionalCollectionRequired(_) => quote! { __item_value },
        PlannedElementValidation::RequiredCollectionOptional(_)
        | PlannedElementValidation::OptionalCollectionOptional(_) => quote! { item },
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
    let collection_assertion = render_each_collection_assertion(field_plan, koruma);
    let element_validator_defaults: Vec<TokenStream2> = field_plan
        .element_validators()
        .iter()
        .map(|validator| {
            let validator_snake = &validator.field_ident;
            quote! { #validator_snake: None }
        })
        .collect();
    let field_expr = access.field_expr(field_plan);
    let field_iter_expr = access.field_iter_expr(field_plan);

    match element {
        PlannedElementValidation::OptionalCollectionOptional(_) => quote! {
            #collection_assertion
            if let Some(ref __collection_value) = #field_expr {
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
        PlannedElementValidation::OptionalCollectionRequired(_) => quote! {
            #collection_assertion
            if let Some(ref __collection_value) = #field_expr {
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
        PlannedElementValidation::RequiredCollectionOptional(_) => quote! {
            #collection_assertion
            for (idx, item) in #field_iter_expr.enumerate() {
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
        PlannedElementValidation::RequiredCollectionRequired(_) => quote! {
            #collection_assertion
            for (idx, __item_value) in #field_iter_expr.enumerate() {
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
