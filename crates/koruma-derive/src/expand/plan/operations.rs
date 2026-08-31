use super::*;

#[derive(Clone, Debug)]
pub(crate) enum PlannedValidationOperation<'a> {
    NestedRequired(PlannedNestedValidation<'a>),
    NestedOptional(PlannedNestedValidation<'a>),
    NewtypeRequired(PlannedNewtypeValidation<'a>),
    NewtypeOptional(PlannedNewtypeValidation<'a>),
    RegularRequired(PlannedRegularValidation<'a>),
    RegularOptional(PlannedRegularValidation<'a>),
}

#[derive(Clone, Debug)]
pub(crate) struct PlannedNestedValidation<'a> {
    pub field: &'a FieldPlan,
    pub direct_storage: bool,
}

#[derive(Clone, Debug)]
pub(crate) struct PlannedNewtypeValidation<'a> {
    pub field: &'a FieldPlan,
    pub field_validators: PlannedFieldValidatorGroups<'a>,
}

#[derive(Clone, Debug)]
pub(crate) struct PlannedRegularValidation<'a> {
    pub field: &'a FieldPlan,
    pub field_validators: PlannedFieldValidatorGroups<'a>,
    pub element_validators: Option<PlannedElementValidation<'a>>,
}

#[derive(Clone, Debug)]
pub(crate) struct PlannedFieldValidatorGroups<'a> {
    pub full_type_validators: Vec<&'a PlannedValidator>,
    pub unwrapped_validators: Vec<&'a PlannedValidator>,
}

impl<'a> PlannedFieldValidatorGroups<'a> {
    pub(crate) fn for_field(field: &'a FieldPlan) -> Self {
        Self {
            full_type_validators: field.full_field_validators().collect(),
            unwrapped_validators: field.unwrapped_field_validators().collect(),
        }
    }

    pub(crate) fn has_full_type_validators(&self) -> bool {
        !self.full_type_validators.is_empty()
    }

    pub(crate) fn has_unwrapped_validators(&self) -> bool {
        !self.unwrapped_validators.is_empty()
    }

    pub(crate) fn has_any(&self) -> bool {
        self.has_full_type_validators() || self.has_unwrapped_validators()
    }

    pub(crate) fn binding(&self) -> PlannedFieldBinding<'a> {
        match (
            self.full_type_validators.is_empty(),
            self.unwrapped_validators.is_empty(),
        ) {
            (true, true) => PlannedFieldBinding::NoValidators,
            (false, true) => PlannedFieldBinding::FullOnly {
                full_type_validators: self.full_type_validators.clone(),
            },
            (true, false) => PlannedFieldBinding::UnwrappedOnly {
                unwrapped_validators: self.unwrapped_validators.clone(),
            },
            (false, false) => PlannedFieldBinding::FullAndUnwrapped {
                full_type_validators: self.full_type_validators.clone(),
                unwrapped_validators: self.unwrapped_validators.clone(),
            },
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) enum PlannedFieldBinding<'a> {
    NoValidators,
    FullOnly {
        full_type_validators: Vec<&'a PlannedValidator>,
    },
    UnwrappedOnly {
        unwrapped_validators: Vec<&'a PlannedValidator>,
    },
    FullAndUnwrapped {
        full_type_validators: Vec<&'a PlannedValidator>,
        unwrapped_validators: Vec<&'a PlannedValidator>,
    },
}

#[derive(Clone, Debug)]
pub(crate) enum PlannedElementValidation<'a> {
    RequiredCollectionRequired(PlannedElementValidatorGroups<'a>),
    RequiredCollectionOptional(PlannedElementValidatorGroups<'a>),
    OptionalCollectionRequired(PlannedElementValidatorGroups<'a>),
    OptionalCollectionOptional(PlannedElementValidatorGroups<'a>),
}

impl<'a> PlannedElementValidation<'a> {
    pub(crate) fn for_field(field: &'a FieldPlan) -> Self {
        let groups = PlannedElementValidatorGroups {
            full_type_validators: field.full_element_validators().collect(),
            unwrapped_validators: field.unwrapped_element_validators().collect(),
        };

        match (field.field_cardinality(), field.element_cardinality()) {
            (Cardinality::Required, Cardinality::Required) => {
                Self::RequiredCollectionRequired(groups)
            },
            (Cardinality::Required, Cardinality::Optional) => {
                Self::RequiredCollectionOptional(groups)
            },
            (Cardinality::Optional, Cardinality::Required) => {
                Self::OptionalCollectionRequired(groups)
            },
            (Cardinality::Optional, Cardinality::Optional) => {
                Self::OptionalCollectionOptional(groups)
            },
        }
    }

    pub(crate) fn groups(&self) -> &PlannedElementValidatorGroups<'a> {
        match self {
            Self::RequiredCollectionRequired(groups)
            | Self::RequiredCollectionOptional(groups)
            | Self::OptionalCollectionRequired(groups)
            | Self::OptionalCollectionOptional(groups) => groups,
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct PlannedElementValidatorGroups<'a> {
    pub full_type_validators: Vec<&'a PlannedValidator>,
    pub unwrapped_validators: Vec<&'a PlannedValidator>,
}

#[derive(Clone, Debug)]
pub(crate) struct FieldSource {
    pub member: Member,
    pub ty: Type,
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "planner unit tests assert tuple-field source indices"
        )
    )]
    pub index: usize,
    pub marker_span: Option<Span>,
}
