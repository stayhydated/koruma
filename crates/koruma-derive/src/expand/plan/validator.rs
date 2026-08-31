use super::*;

#[derive(Clone, Debug)]
pub(crate) struct PlannedValidator {
    pub attr: ValidatorAttr,
    pub label: Option<ValidatorLabel>,
    pub source_span: Span,
    pub doc_name: String,
    pub target: ValidationTarget,
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "planner unit tests assert resolved validator type inference"
        )
    )]
    pub resolved_type_arg: PlannedValidatorTypeArg,
    pub validator_type: PlannedValidatorType,
    pub builder_type: PlannedValidatorType,
    pub setter_calls: Vec<PlannedSetterCall>,
    pub field_ident: Ident,
    pub variant_ident: Ident,
}

pub(crate) fn plan_validators(
    validators: &[ParsedValidatorUse],
    name_plans: &[ValidatorNamePlan],
    field: &FieldInfo,
    target_context: ValidationTargetContext<'_>,
    known_field_names: &[Ident],
) -> Result<Vec<PlannedValidator>, syn::Error> {
    let mut planned = Vec::new();
    let mut errors = ErrorBag::new();

    if validators.len() != name_plans.len() {
        return Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            "internal error: validator name plan count did not match validator count",
        ));
    }

    for (validator, name_plan) in validators.iter().zip(name_plans) {
        if let Some(validator) = errors.push_result(PlannedValidator::build(
            validator,
            name_plan,
            field,
            target_context,
            known_field_names,
        )) {
            planned.push(validator);
        }
    }

    errors.finish()?;
    Ok(planned)
}

impl PlannedValidator {
    pub(crate) fn doc_name(&self) -> String {
        self.doc_name.clone()
    }

    pub(crate) fn build(
        validator_use: &ParsedValidatorUse,
        name_plan: &ValidatorNamePlan,
        field: &FieldInfo,
        target_context: ValidationTargetContext<'_>,
        known_field_names: &[Ident],
    ) -> Result<Self, syn::Error> {
        let validator = validator_use.validator();
        let target = ValidationTarget::for_validator(
            validator,
            validator_use.target(),
            target_context,
            field.name(),
        )?;
        let resolved_explicit_type = target.resolve_explicit_infer_type(validator)?;
        let resolved_type_arg = PlannedValidatorTypeArg::for_validator(
            validator,
            &target,
            resolved_explicit_type.clone(),
        );
        let concrete_type_arg =
            concrete_validator_type_arg(validator, &target, resolved_explicit_type);
        let validator_type =
            PlannedValidatorType::new(validator.path().clone(), concrete_type_arg.as_ref());
        let builder_type_arg =
            builder_validator_type_arg(validator, concrete_type_arg.as_ref().cloned());
        let builder_type =
            PlannedValidatorType::new(validator.path().clone(), builder_type_arg.as_ref());
        let setter_calls = planned_setter_calls(validator.setter_calls(), known_field_names)?;

        Ok(Self {
            attr: validator.clone(),
            label: validator_use.label().cloned(),
            source_span: validator_use.source_span(),
            doc_name: name_plan.doc_name.clone(),
            target,
            resolved_type_arg,
            validator_type,
            builder_type,
            setter_calls,
            field_ident: name_plan.field_ident.clone(),
            variant_ident: name_plan.variant_ident.clone(),
        })
    }
}

pub(crate) fn builder_validator_type_arg(
    validator: &ValidatorAttr,
    concrete_type_arg: Option<Type>,
) -> Option<Type> {
    let uses_infer = validator.uses_type_inference()
        || validator
            .explicit_type()
            .is_some_and(koruma_derive_core::contains_infer_type);

    if uses_infer {
        return concrete_type_arg;
    }

    None
}

pub(crate) fn concrete_validator_type_arg(
    validator: &ValidatorAttr,
    target: &ValidationTarget,
    resolved_explicit_type: Option<Type>,
) -> Option<Type> {
    if let Some(resolved) = resolved_explicit_type {
        return Some(resolved);
    }

    if let Some(explicit) = validator.explicit_type() {
        return Some(explicit.clone());
    }

    if validator.uses_type_inference() {
        return Some(target.validate_type().clone());
    }

    None
}

#[derive(Clone, Debug)]
pub(crate) struct PlannedValidatorType {
    pub validator: Path,
    pub type_arg: Option<Type>,
}

impl PlannedValidatorType {
    pub(crate) fn new(validator: Path, type_arg: Option<&Type>) -> Self {
        Self {
            validator,
            type_arg: type_arg.cloned(),
        }
    }

    pub(crate) fn as_type(&self) -> Type {
        let validator = &self.validator;
        if let Some(type_arg) = &self.type_arg {
            syn::parse_quote! { #validator<#type_arg> }
        } else {
            syn::parse_quote! { #validator }
        }
    }
}
