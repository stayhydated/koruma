use super::*;

#[derive(Clone, Debug)]
pub(crate) struct FieldPlan {
    pub name: Ident,
    pub source: FieldSource,
    pub generated_names: GeneratedNames,
    pub shape: PlannedField,
}

impl FieldPlan {
    pub(crate) fn build(
        field: &FieldInfo,
        generated_names: GeneratedNames,
        known_field_names: &[Ident],
    ) -> Result<Self, syn::Error> {
        let mut errors = ErrorBag::new();
        let each_collection = if field.has_element_validators() {
            errors.push_result(classify_each_collection(field.ty()))
        } else {
            None
        };

        let field_name_plans = errors
            .push_result(plan_validator_names(
                field.field_validators(),
                known_field_names,
            ))
            .unwrap_or_default();
        let element_name_plans = errors
            .push_result(plan_validator_names(
                field.element_validators(),
                known_field_names,
            ))
            .unwrap_or_default();
        errors.finish()?;

        let target_context = FieldTargetContext::new(field.ty(), each_collection.as_ref());

        let field_validators = plan_validators(
            field.field_validators(),
            &field_name_plans,
            field,
            ValidationTargetContext::Field(target_context.field()),
            known_field_names,
        )?;

        let element_validators = if let Some(collection) = target_context.collection() {
            plan_validators(
                field.element_validators(),
                &element_name_plans,
                field,
                ValidationTargetContext::Element(collection.element()),
                known_field_names,
            )?
        } else {
            Vec::new()
        };

        let field_cardinality = target_context.field().cardinality;
        let inner_type = target_context.field().validate_ty.clone();
        let element_type = target_context
            .collection()
            .map(|collection| collection.element().raw_ty.clone());
        let element_cardinality = target_context
            .collection()
            .map(|collection| collection.element().cardinality)
            .unwrap_or(Cardinality::Required);
        let shape = if field.is_nested() {
            PlannedField::Nested(NestedFieldPlan {
                cardinality: field_cardinality,
                inner_type,
            })
        } else if field.is_newtype() {
            PlannedField::Newtype(NewtypeFieldPlan {
                cardinality: field_cardinality,
                inner_type,
                field_validators,
            })
        } else {
            PlannedField::Regular(Box::new(RegularFieldPlan {
                cardinality: field_cardinality,
                inner_type,
                element_type,
                element_cardinality,
                field_validators,
                element_validators,
            }))
        };

        Ok(Self {
            name: field.name().clone(),
            source: FieldSource {
                member: field.member().clone(),
                ty: field.ty().clone(),
                index: field.index(),
                marker_span: field.marker_span(),
            },
            generated_names,
            shape,
        })
    }

    pub(crate) fn field_validators(&self) -> &[PlannedValidator] {
        match &self.shape {
            PlannedField::Regular(plan) => &plan.field_validators,
            PlannedField::Nested(_) => &[],
            PlannedField::Newtype(plan) => &plan.field_validators,
        }
    }

    pub(crate) fn element_validators(&self) -> &[PlannedValidator] {
        match &self.shape {
            PlannedField::Regular(plan) => &plan.element_validators,
            PlannedField::Nested(_) | PlannedField::Newtype(_) => &[],
        }
    }

    pub(crate) fn field_optional(&self) -> bool {
        self.field_cardinality().is_optional()
    }

    pub(crate) fn field_cardinality(&self) -> Cardinality {
        match &self.shape {
            PlannedField::Regular(plan) => plan.cardinality,
            PlannedField::Nested(plan) => plan.cardinality,
            PlannedField::Newtype(plan) => plan.cardinality,
        }
    }

    pub(crate) fn inner_type(&self) -> &Type {
        match &self.shape {
            PlannedField::Regular(plan) => &plan.inner_type,
            PlannedField::Nested(plan) => &plan.inner_type,
            PlannedField::Newtype(plan) => &plan.inner_type,
        }
    }

    pub(crate) fn element_type(&self) -> Option<&Type> {
        match &self.shape {
            PlannedField::Regular(plan) => plan.element_type.as_ref(),
            PlannedField::Nested(_) | PlannedField::Newtype(_) => None,
        }
    }

    #[cfg(test)]
    pub(crate) fn element_optional(&self) -> bool {
        self.element_cardinality().is_optional()
    }

    pub(crate) fn element_cardinality(&self) -> Cardinality {
        match &self.shape {
            PlannedField::Regular(plan) => plan.element_cardinality,
            PlannedField::Nested(_) | PlannedField::Newtype(_) => Cardinality::Required,
        }
    }

    pub(crate) fn full_field_validators(&self) -> impl Iterator<Item = &PlannedValidator> {
        self.field_validators()
            .iter()
            .filter(|validator| matches!(validator.target, ValidationTarget::FieldFull(_)))
    }

    pub(crate) fn is_nested(&self) -> bool {
        matches!(self.shape, PlannedField::Nested(_))
    }

    pub(crate) fn is_newtype(&self) -> bool {
        matches!(self.shape, PlannedField::Newtype(_))
    }

    pub(crate) fn has_field_validators(&self) -> bool {
        !self.field_validators().is_empty()
    }

    pub(crate) fn has_element_validators(&self) -> bool {
        !self.element_validators().is_empty()
    }

    pub(crate) fn error_storage(&self) -> ErrorStorage {
        ErrorStorage::for_shape(&self.shape)
    }

    pub(crate) fn unwrapped_field_validators(&self) -> impl Iterator<Item = &PlannedValidator> {
        self.field_validators()
            .iter()
            .filter(|validator| matches!(validator.target, ValidationTarget::FieldUnwrapped(_)))
    }

    pub(crate) fn full_element_validators(&self) -> impl Iterator<Item = &PlannedValidator> {
        self.element_validators()
            .iter()
            .filter(|validator| matches!(validator.target, ValidationTarget::ElementFull(_)))
    }

    pub(crate) fn unwrapped_element_validators(&self) -> impl Iterator<Item = &PlannedValidator> {
        self.element_validators()
            .iter()
            .filter(|validator| matches!(validator.target, ValidationTarget::ElementUnwrapped(_)))
    }
}

#[derive(Clone, Debug)]
pub(crate) enum PlannedField {
    Regular(Box<RegularFieldPlan>),
    Nested(NestedFieldPlan),
    Newtype(NewtypeFieldPlan),
}

#[derive(Clone, Debug)]
pub(crate) struct RegularFieldPlan {
    pub cardinality: Cardinality,
    pub inner_type: Type,
    pub element_type: Option<Type>,
    pub element_cardinality: Cardinality,
    pub field_validators: Vec<PlannedValidator>,
    pub element_validators: Vec<PlannedValidator>,
}

#[derive(Clone, Debug)]
pub(crate) struct NestedFieldPlan {
    pub cardinality: Cardinality,
    pub inner_type: Type,
}

#[derive(Clone, Debug)]
pub(crate) struct NewtypeFieldPlan {
    pub cardinality: Cardinality,
    pub inner_type: Type,
    pub field_validators: Vec<PlannedValidator>,
}
