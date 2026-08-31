use super::*;

#[derive(Clone, Debug)]
pub(crate) enum ErrorStorage {
    Nested {
        #[cfg_attr(
            not(test),
            expect(
                dead_code,
                reason = "planner unit tests assert nested error cardinality"
            )
        )]
        cardinality: Cardinality,
    },
    NewtypeInner {
        cardinality: Cardinality,
    },
    NewtypeWithValidators {
        cardinality: Cardinality,
    },
    RegularEmpty,
    RegularFieldValidators,
    RegularElementValidators,
    RegularFieldAndElementValidators,
}

impl ErrorStorage {
    pub(crate) fn for_shape(shape: &PlannedField) -> Self {
        match shape {
            PlannedField::Nested(nested) => Self::Nested {
                cardinality: nested.cardinality,
            },
            PlannedField::Newtype(newtype) if newtype.field_validators.is_empty() => {
                Self::NewtypeInner {
                    cardinality: newtype.cardinality,
                }
            },
            PlannedField::Newtype(newtype) => Self::NewtypeWithValidators {
                cardinality: newtype.cardinality,
            },
            PlannedField::Regular(regular) => match (
                regular.field_validators.is_empty(),
                regular.element_validators.is_empty(),
            ) {
                (true, true) => Self::RegularEmpty,
                (false, true) => Self::RegularFieldValidators,
                (true, false) => Self::RegularElementValidators,
                (false, false) => Self::RegularFieldAndElementValidators,
            },
        }
    }
}
