use super::*;

#[derive(Clone, Debug)]
pub(crate) enum StructPlan {
    Record,
    Tuple,
    Unit,
    Newtype { field_index: usize },
}

#[derive(Clone, Debug)]
pub(crate) struct MainErrorRenderPlan<'a> {
    pub fields: Vec<PlannedMainErrorField<'a>>,
}

#[derive(Clone, Debug)]
pub(crate) struct PlannedMainErrorField<'a> {
    pub field: &'a FieldPlan,
    pub storage: PlannedMainErrorStorage,
    pub getter: PlannedErrorGetter,
    pub default: PlannedErrorDefault,
    pub is_empty: PlannedErrorIsEmpty,
}

impl<'a> PlannedMainErrorField<'a> {
    pub(crate) fn for_field(field: &'a FieldPlan, struct_is_newtype: bool) -> Self {
        let direct_nested = field.is_nested() && struct_is_newtype && !field.field_optional();
        let storage = if direct_nested {
            PlannedMainErrorStorage::NestedDirect
        } else if field.is_nested() {
            PlannedMainErrorStorage::NestedOptional
        } else {
            PlannedMainErrorStorage::FieldError
        };
        let getter = if direct_nested {
            PlannedErrorGetter::NestedDirect
        } else if field.is_nested() {
            PlannedErrorGetter::NestedOptional
        } else if field.is_newtype() && !field.has_field_validators() {
            if field.field_optional() {
                PlannedErrorGetter::NewtypeInnerOptional
            } else {
                PlannedErrorGetter::NewtypeInnerDirect
            }
        } else {
            PlannedErrorGetter::FieldError
        };
        let default = if direct_nested {
            PlannedErrorDefault::NestedDirect
        } else if field.is_nested() {
            PlannedErrorDefault::None
        } else if field.is_newtype() && field.has_field_validators() {
            PlannedErrorDefault::NewtypeWithValidators {
                inner_optional: field.field_optional(),
            }
        } else if field.is_newtype() {
            PlannedErrorDefault::FieldErrorDefault
        } else if field.has_element_validators() {
            PlannedErrorDefault::Regular {
                has_field_validators: field.has_field_validators(),
                has_element_validators: true,
            }
        } else {
            PlannedErrorDefault::Regular {
                has_field_validators: field.has_field_validators(),
                has_element_validators: false,
            }
        };
        let is_empty = if direct_nested {
            PlannedErrorIsEmpty::NestedDirect
        } else if field.is_nested() {
            PlannedErrorIsEmpty::NestedOptional
        } else {
            PlannedErrorIsEmpty::FieldError
        };

        Self {
            field,
            storage,
            getter,
            default,
            is_empty,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PlannedMainErrorStorage {
    NestedDirect,
    NestedOptional,
    FieldError,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PlannedErrorGetter {
    NestedDirect,
    NestedOptional,
    FieldError,
    NewtypeInnerDirect,
    NewtypeInnerOptional,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PlannedErrorDefault {
    NestedDirect,
    None,
    FieldErrorDefault,
    NewtypeWithValidators {
        inner_optional: bool,
    },
    Regular {
        has_field_validators: bool,
        has_element_validators: bool,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PlannedErrorIsEmpty {
    NestedDirect,
    NestedOptional,
    FieldError,
}

#[derive(Clone, Debug)]
pub(crate) struct FieldErrorRenderPlan<'a> {
    pub fields: Vec<PlannedFieldError<'a>>,
}

#[derive(Clone, Debug)]
pub(crate) enum PlannedFieldError<'a> {
    NewtypeInnerRequired {
        field: &'a FieldPlan,
    },
    NewtypeInnerOptional {
        field: &'a FieldPlan,
    },
    NewtypeWithValidatorsRequired {
        field: &'a FieldPlan,
        field_validators: Vec<&'a PlannedValidator>,
    },
    NewtypeWithValidatorsOptional {
        field: &'a FieldPlan,
        field_validators: Vec<&'a PlannedValidator>,
    },
    RegularFieldOnly {
        field: &'a FieldPlan,
        field_validators: Vec<&'a PlannedValidator>,
    },
    RegularElementOnly {
        field: &'a FieldPlan,
        element_validators: Vec<&'a PlannedValidator>,
    },
    RegularFieldAndElement {
        field: &'a FieldPlan,
        field_validators: Vec<&'a PlannedValidator>,
        element_validators: Vec<&'a PlannedValidator>,
    },
}

impl<'a> PlannedFieldError<'a> {
    pub(crate) fn for_field(field: &'a FieldPlan) -> Option<Self> {
        if field.is_nested() {
            return None;
        }

        match field.error_storage() {
            ErrorStorage::NewtypeInner {
                cardinality: Cardinality::Required,
            } => Some(Self::NewtypeInnerRequired { field }),
            ErrorStorage::NewtypeInner {
                cardinality: Cardinality::Optional,
            } => Some(Self::NewtypeInnerOptional { field }),
            ErrorStorage::NewtypeWithValidators {
                cardinality: Cardinality::Required,
            } => Some(Self::NewtypeWithValidatorsRequired {
                field,
                field_validators: field.field_validators().iter().collect(),
            }),
            ErrorStorage::NewtypeWithValidators {
                cardinality: Cardinality::Optional,
            } => Some(Self::NewtypeWithValidatorsOptional {
                field,
                field_validators: field.field_validators().iter().collect(),
            }),
            ErrorStorage::RegularEmpty => None,
            ErrorStorage::RegularFieldValidators => Some(Self::RegularFieldOnly {
                field,
                field_validators: field.field_validators().iter().collect(),
            }),
            ErrorStorage::RegularElementValidators => Some(Self::RegularElementOnly {
                field,
                element_validators: field.element_validators().iter().collect(),
            }),
            ErrorStorage::RegularFieldAndElementValidators => Some(Self::RegularFieldAndElement {
                field,
                field_validators: field.field_validators().iter().collect(),
                element_validators: field.element_validators().iter().collect(),
            }),
            ErrorStorage::Nested { .. } => None,
        }
    }

    pub(crate) fn field(&self) -> &'a FieldPlan {
        match self {
            Self::NewtypeInnerRequired { field }
            | Self::NewtypeInnerOptional { field }
            | Self::NewtypeWithValidatorsRequired { field, .. }
            | Self::NewtypeWithValidatorsOptional { field, .. }
            | Self::RegularFieldOnly { field, .. }
            | Self::RegularElementOnly { field, .. }
            | Self::RegularFieldAndElement { field, .. } => field,
        }
    }

    pub(crate) fn shape(&self) -> FieldErrorShape {
        match self {
            Self::NewtypeInnerRequired { .. } => FieldErrorShape::NewtypeInnerRequired,
            Self::NewtypeInnerOptional { .. } => FieldErrorShape::NewtypeInnerOptional,
            Self::NewtypeWithValidatorsRequired { .. } => {
                FieldErrorShape::NewtypeWithValidatorsRequired
            },
            Self::NewtypeWithValidatorsOptional { .. } => {
                FieldErrorShape::NewtypeWithValidatorsOptional
            },
            Self::RegularFieldOnly { .. } => FieldErrorShape::RegularFieldOnly,
            Self::RegularElementOnly { .. } => FieldErrorShape::RegularElementOnly,
            Self::RegularFieldAndElement { .. } => FieldErrorShape::RegularFieldAndElement,
        }
    }

    pub(crate) fn field_validators(&self) -> &[&'a PlannedValidator] {
        match self {
            Self::NewtypeWithValidatorsRequired {
                field_validators, ..
            }
            | Self::NewtypeWithValidatorsOptional {
                field_validators, ..
            }
            | Self::RegularFieldOnly {
                field_validators, ..
            }
            | Self::RegularFieldAndElement {
                field_validators, ..
            } => field_validators,
            Self::NewtypeInnerRequired { .. }
            | Self::NewtypeInnerOptional { .. }
            | Self::RegularElementOnly { .. } => &[],
        }
    }

    pub(crate) fn element_validators(&self) -> &[&'a PlannedValidator] {
        match self {
            Self::RegularElementOnly {
                element_validators, ..
            }
            | Self::RegularFieldAndElement {
                element_validators, ..
            } => element_validators,
            Self::NewtypeInnerRequired { .. }
            | Self::NewtypeInnerOptional { .. }
            | Self::NewtypeWithValidatorsRequired { .. }
            | Self::NewtypeWithValidatorsOptional { .. }
            | Self::RegularFieldOnly { .. } => &[],
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FieldErrorShape {
    NewtypeInnerRequired,
    NewtypeInnerOptional,
    NewtypeWithValidatorsRequired,
    NewtypeWithValidatorsOptional,
    RegularFieldOnly,
    RegularElementOnly,
    RegularFieldAndElement,
}
