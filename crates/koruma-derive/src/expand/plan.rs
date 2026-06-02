#![allow(dead_code)]

use koruma_derive_core::{
    BuilderMethodCall, FieldInfo, ParsedValidatorUse, StructOptions, ValidatorAttr, ValidatorLabel,
    ValidatorSetterArg, ValidatorTargetSelector, contains_infer_type, expr_as_simple_ident,
    option_inner_type, parse_struct_options, substitute_infer_type_from_source,
};
use quote::quote;
use syn::{DeriveInput, Fields, Ident, Member, Path, Type, spanned::Spanned};

use super::codegen::{Cardinality, EachCollection, classify_each_collection};
use super::collect_field_infos;
use super::error_bag::ErrorBag;
use super::names::validate_validator_uses;
use super::names::{GeneratedNames, validator_names};

#[derive(Clone, Debug)]
pub(crate) struct ValidationPlan {
    pub struct_options: StructOptions,
    pub struct_plan: StructPlan,
    pub fields: Vec<FieldPlan>,
    pub known_field_names: Vec<Ident>,
}

impl ValidationPlan {
    pub fn build(input: &DeriveInput, derive_name: &str) -> Result<Self, syn::Error> {
        let struct_options = parse_struct_options(&input.attrs)?;
        let fields = struct_fields(input, derive_name)?;
        let field_infos = collect_field_infos(fields, Some(&struct_options))?;
        let total_fields = fields.len();

        if struct_options.try_from() && total_fields != 1 {
            return Err(syn::Error::new_spanned(
                input,
                format!("newtype(try_from) requires exactly one field, found {total_fields}"),
            ));
        }

        if struct_options.is_newtype() && total_fields != 1 {
            return Err(syn::Error::new_spanned(
                input,
                format!("newtype structs must have exactly one field, found {total_fields}"),
            ));
        }

        let known_field_names: Vec<Ident> = fields
            .iter()
            .filter_map(|field| field.ident.clone())
            .collect();

        let mut plan_errors = ErrorBag::new();
        let mut planned_fields = Vec::new();
        for field in &field_infos {
            if let Some(field) =
                plan_errors.push_result(FieldPlan::build(&input.ident, field, &known_field_names))
            {
                planned_fields.push(field);
            }
        }
        plan_errors.finish()?;

        let struct_plan = if struct_options.is_newtype() {
            if planned_fields.is_empty() {
                return Err(syn::Error::new_spanned(
                    input,
                    "newtype structs require a field validation plan",
                ));
            }
            StructPlan::Newtype { field_index: 0 }
        } else {
            match fields {
                Fields::Named(_) => StructPlan::Record,
                Fields::Unnamed(_) => StructPlan::Tuple,
                Fields::Unit => StructPlan::Unit,
            }
        };

        Ok(Self {
            struct_options,
            struct_plan,
            fields: planned_fields,
            known_field_names,
        })
    }

    pub fn field_plan(&self, name: &Ident) -> Option<&FieldPlan> {
        self.fields.iter().find(|field| &field.name == name)
    }

    pub fn struct_newtype(&self) -> Option<&FieldPlan> {
        match &self.struct_plan {
            StructPlan::Newtype { field_index } => self.fields.get(*field_index),
            StructPlan::Record | StructPlan::Tuple | StructPlan::Unit => None,
        }
    }

    pub(crate) fn validation_operations(&self) -> Vec<PlannedValidationOperation<'_>> {
        self.validation_render_plan().operations
    }

    pub(crate) fn validation_render_plan(&self) -> ValidationRenderPlan<'_> {
        let struct_is_newtype = self.struct_newtype().is_some();

        let operations = self
            .fields
            .iter()
            .map(|field| {
                if field.is_nested() {
                    let operation = PlannedNestedValidation {
                        field,
                        direct_storage: struct_is_newtype,
                    };
                    return if field.field_optional() {
                        PlannedValidationOperation::NestedOptional(operation)
                    } else {
                        PlannedValidationOperation::NestedRequired(operation)
                    };
                }

                if field.is_newtype() {
                    let operation = PlannedNewtypeValidation {
                        field,
                        field_validators: PlannedFieldValidatorGroups::for_field(field),
                    };
                    return if field.field_optional() {
                        PlannedValidationOperation::NewtypeOptional(operation)
                    } else {
                        PlannedValidationOperation::NewtypeRequired(operation)
                    };
                }

                let operation = PlannedRegularValidation {
                    field,
                    field_validators: PlannedFieldValidatorGroups::for_field(field),
                    element_validators: field
                        .has_element_validators()
                        .then(|| PlannedElementValidation::for_field(field)),
                };

                if field.field_optional() {
                    PlannedValidationOperation::RegularOptional(operation)
                } else {
                    PlannedValidationOperation::RegularRequired(operation)
                }
            })
            .collect();

        ValidationRenderPlan { operations }
    }

    pub(crate) fn main_error_render_plan(&self) -> MainErrorRenderPlan<'_> {
        let struct_is_newtype = self.struct_newtype().is_some();
        let fields = self
            .fields
            .iter()
            .map(|field| PlannedMainErrorField::for_field(field, struct_is_newtype))
            .collect();

        MainErrorRenderPlan { fields }
    }

    pub(crate) fn field_error_render_plan(&self) -> FieldErrorRenderPlan<'_> {
        let fields = self
            .fields
            .iter()
            .filter_map(PlannedFieldError::for_field)
            .collect();

        FieldErrorRenderPlan { fields }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ValidationRenderPlan<'a> {
    pub operations: Vec<PlannedValidationOperation<'a>>,
}

fn struct_fields<'a>(input: &'a DeriveInput, derive_name: &str) -> Result<&'a Fields, syn::Error> {
    match &input.data {
        syn::Data::Struct(data) => Ok(&data.fields),
        _ => Err(syn::Error::new_spanned(
            input,
            format!("{derive_name} can only be derived for structs"),
        )),
    }
}

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
    fn for_field(field: &'a FieldPlan, struct_is_newtype: bool) -> Self {
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
pub(crate) struct PlannedFieldError<'a> {
    pub field: &'a FieldPlan,
    pub kind: PlannedFieldErrorKind,
    pub field_validators: Vec<&'a PlannedValidator>,
    pub element_validators: Vec<&'a PlannedValidator>,
}

impl<'a> PlannedFieldError<'a> {
    fn for_field(field: &'a FieldPlan) -> Option<Self> {
        if field.is_nested() {
            return None;
        }

        let field_validators: Vec<_> = field.field_validators().iter().collect();
        let element_validators: Vec<_> = field.element_validators().iter().collect();
        let kind = match field.error_storage() {
            ErrorStorage::NewtypeInner { cardinality } => PlannedFieldErrorKind::NewtypeInner {
                inner_optional: cardinality == Cardinality::Optional,
                deref: cardinality == Cardinality::Required,
            },
            ErrorStorage::NewtypeWithValidators { cardinality } => {
                PlannedFieldErrorKind::NewtypeWithValidators {
                    inner_optional: cardinality == Cardinality::Optional,
                }
            },
            ErrorStorage::RegularEmpty => {
                PlannedFieldErrorKind::Regular(PlannedRegularFieldErrorKind::Empty)
            },
            ErrorStorage::RegularFieldValidators => {
                PlannedFieldErrorKind::Regular(PlannedRegularFieldErrorKind::FieldValidators)
            },
            ErrorStorage::RegularElementValidators => {
                PlannedFieldErrorKind::Regular(PlannedRegularFieldErrorKind::ElementValidators)
            },
            ErrorStorage::RegularFieldAndElementValidators => PlannedFieldErrorKind::Regular(
                PlannedRegularFieldErrorKind::FieldAndElementValidators,
            ),
            ErrorStorage::Nested { .. } => return None,
        };

        Some(Self {
            field,
            kind,
            field_validators,
            element_validators,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PlannedFieldErrorKind {
    NewtypeInner { inner_optional: bool, deref: bool },
    NewtypeWithValidators { inner_optional: bool },
    Regular(PlannedRegularFieldErrorKind),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PlannedRegularFieldErrorKind {
    Empty,
    FieldValidators,
    ElementValidators,
    FieldAndElementValidators,
}

impl PlannedRegularFieldErrorKind {
    pub(crate) fn has_field_validators(self) -> bool {
        matches!(
            self,
            Self::FieldValidators | Self::FieldAndElementValidators
        )
    }

    pub(crate) fn has_element_validators(self) -> bool {
        matches!(
            self,
            Self::ElementValidators | Self::FieldAndElementValidators
        )
    }
}

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
    fn for_field(field: &'a FieldPlan) -> Self {
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
}

#[derive(Clone, Debug)]
pub(crate) enum PlannedElementValidation<'a> {
    RequiredElement(PlannedElementValidatorGroups<'a>),
    OptionalElement(PlannedElementValidatorGroups<'a>),
}

impl<'a> PlannedElementValidation<'a> {
    fn for_field(field: &'a FieldPlan) -> Self {
        let groups = PlannedElementValidatorGroups {
            full_type_validators: field.full_element_validators().collect(),
            unwrapped_validators: field.unwrapped_element_validators().collect(),
        };

        if field.element_optional() {
            Self::OptionalElement(groups)
        } else {
            Self::RequiredElement(groups)
        }
    }

    pub(crate) fn groups(&self) -> &PlannedElementValidatorGroups<'a> {
        match self {
            Self::RequiredElement(groups) | Self::OptionalElement(groups) => groups,
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
    pub name: Ident,
    pub member: Member,
    pub ty: Type,
    pub index: usize,
}

#[derive(Clone, Debug)]
pub(crate) struct FieldPlan {
    pub name: Ident,
    pub source: FieldSource,
    pub generated_names: GeneratedNames,
    pub shape: PlannedField,
}

impl FieldPlan {
    fn build(
        struct_name: &Ident,
        field: &FieldInfo,
        known_field_names: &[Ident],
    ) -> Result<Self, syn::Error> {
        let mut errors = ErrorBag::new();
        let each_collection = if field.has_element_validators() {
            errors.push_result(classify_each_collection(field.ty()))
        } else {
            None
        };

        errors.push_result(validate_validator_uses(
            field.field_validators(),
            known_field_names,
        ));
        errors.push_result(validate_validator_uses(
            field.element_validators(),
            known_field_names,
        ));
        errors.finish()?;

        let target_context = FieldTargetContext::new(field.ty(), each_collection.as_ref());

        let field_validators = plan_validators(
            field.field_validators(),
            field,
            &target_context,
            ValidationTargetScope::Field,
            known_field_names,
        )?;

        let element_validators = plan_validators(
            field.element_validators(),
            field,
            &target_context,
            ValidationTargetScope::Element,
            known_field_names,
        )?;

        let field_cardinality = target_context.field.cardinality;
        let inner_type = target_context.field.inner_ty.clone();
        let element_type = target_context
            .element
            .as_ref()
            .map(|element| element.element.raw_ty.clone());
        let element_cardinality = target_context
            .element
            .as_ref()
            .map(|element| element.element.cardinality)
            .unwrap_or(Cardinality::Required);
        let generated_names = GeneratedNames::for_field(struct_name, field);

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
            PlannedField::Regular(RegularFieldPlan {
                cardinality: field_cardinality,
                inner_type,
                element_type,
                element_cardinality,
                field_validators,
                element_validators,
            })
        };

        Ok(Self {
            name: field.name().clone(),
            source: FieldSource {
                name: field.name().clone(),
                member: field.member().clone(),
                ty: field.ty().clone(),
                index: field.index(),
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
    Regular(RegularFieldPlan),
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

pub(crate) struct FieldTargetContext<'a> {
    pub field: ValueTargetContext,
    pub element: Option<ElementTargetContext<'a>>,
}

impl<'a> FieldTargetContext<'a> {
    fn new(field_ty: &Type, each_collection: Option<&EachCollection<'a>>) -> Self {
        let field = ValueTargetContext::new(field_ty, each_collection.map(|c| c.outer_cardinality));
        let element = each_collection.map(|collection| ElementTargetContext {
            collection: *collection,
            element: ValueTargetContext::new(
                collection.element_ty,
                Some(collection.element_cardinality),
            ),
        });

        Self { field, element }
    }

    fn for_scope(&self, scope: ValidationTargetScope) -> &ValueTargetContext {
        match scope {
            ValidationTargetScope::Field => &self.field,
            ValidationTargetScope::Element => {
                &self
                    .element
                    .as_ref()
                    .expect("element target context should exist for element validators")
                    .element
            },
        }
    }
}

pub(crate) struct ValueTargetContext {
    pub raw_ty: Type,
    pub inner_ty: Type,
    pub cardinality: Cardinality,
}

impl ValueTargetContext {
    fn new(raw_ty: &Type, cardinality: Option<Cardinality>) -> Self {
        Self {
            raw_ty: raw_ty.clone(),
            inner_ty: option_inner_type(raw_ty).unwrap_or(raw_ty).clone(),
            cardinality: cardinality.unwrap_or_else(|| Cardinality::for_type(raw_ty)),
        }
    }
}

pub(crate) struct ElementTargetContext<'a> {
    pub collection: EachCollection<'a>,
    pub element: ValueTargetContext,
}

#[derive(Clone, Debug)]
pub(crate) struct PlannedValidator {
    pub attr: ValidatorAttr,
    pub label: Option<ValidatorLabel>,
    pub target: ValidationTarget,
    pub resolved_type_arg: PlannedValidatorTypeArg,
    pub validator_type: PlannedValidatorType,
    pub builder_type: PlannedValidatorType,
    pub setter_calls: Vec<PlannedSetterCall>,
    pub field_ident: Ident,
    pub variant_ident: Ident,
}

fn plan_validators(
    validators: &[ParsedValidatorUse],
    field: &FieldInfo,
    target_context: &FieldTargetContext<'_>,
    scope: ValidationTargetScope,
    known_field_names: &[Ident],
) -> Result<Vec<PlannedValidator>, syn::Error> {
    let mut planned = Vec::new();
    let mut errors = ErrorBag::new();

    for validator in validators {
        if let Some(validator) = errors.push_result(PlannedValidator::build(
            validator,
            field,
            target_context,
            scope,
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
        self.label
            .as_ref()
            .map(ToString::to_string)
            .unwrap_or_else(|| self.attr.path_name())
    }

    fn build(
        validator_use: &ParsedValidatorUse,
        field: &FieldInfo,
        target_context: &FieldTargetContext<'_>,
        scope: ValidationTargetScope,
        known_field_names: &[Ident],
    ) -> Result<Self, syn::Error> {
        let validator = validator_use.validator();
        let target = ValidationTarget::for_validator(
            validator,
            validator_use.target(),
            target_context,
            scope,
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
        let siblings = if scope.is_element() {
            field.element_validators()
        } else {
            field.field_validators()
        };
        let generated_names = validator_names(validator_use, siblings, known_field_names)?;

        Ok(Self {
            attr: validator.clone(),
            label: validator_use.label().cloned(),
            target,
            resolved_type_arg,
            validator_type: validator_type.clone(),
            builder_type,
            setter_calls,
            field_ident: generated_names.field_ident,
            variant_ident: generated_names.variant_ident,
        })
    }
}

fn builder_validator_type_arg(
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

fn planned_setter_calls(
    calls: &[BuilderMethodCall],
    known_field_names: &[Ident],
) -> Result<Vec<PlannedSetterCall>, syn::Error> {
    let mut errors = ErrorBag::new();
    let mut planned = Vec::new();

    for call in calls {
        let mut planned_args = Vec::new();
        for arg in call.args() {
            match plan_setter_arg(arg, known_field_names) {
                Ok(arg) => planned_args.push(arg),
                Err(error) => errors.push(error),
            }
        }

        if planned_args.len() == call.args().len() {
            planned.push(PlannedSetterCall {
                method: call.method().clone(),
                args: planned_args,
            });
        }
    }

    errors.finish()?;
    Ok(planned)
}

fn plan_setter_arg(
    arg: &ValidatorSetterArg,
    known_field_names: &[Ident],
) -> Result<PlannedSetterArg, syn::Error> {
    match arg {
        ValidatorSetterArg::Expr(expr) => {
            if let Some(field_ident) = expr_as_simple_ident(expr)
                && known_field_names.iter().any(|name| name == field_ident)
            {
                return Err(syn::Error::new_spanned(
                    expr,
                    format!(
                        "bare field argument `{field_ident}` is ambiguous; use `self.{field_ident}.clone()` explicitly"
                    ),
                ));
            }

            Ok(PlannedSetterArg::Expr(expr.clone()))
        },
    }
}

#[derive(Clone, Debug)]
pub(crate) enum PlannedSetterArg {
    Expr(syn::Expr),
}

#[derive(Clone, Debug)]
pub(crate) struct PlannedSetterCall {
    pub method: Ident,
    pub args: Vec<PlannedSetterArg>,
}

fn concrete_validator_type_arg(
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
    fn new(validator: Path, type_arg: Option<&Type>) -> Self {
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

#[derive(Clone, Debug)]
pub(crate) enum ValidationTarget {
    FieldFull(FullFieldTarget),
    FieldUnwrapped(UnwrappedFieldTarget),
    ElementFull(FullElementTarget),
    ElementUnwrapped(UnwrappedElementTarget),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TargetBorrow {
    Reference,
    AlreadyBorrowed,
}

#[derive(Clone, Debug)]
pub(crate) struct FullFieldTarget {
    pub ty: Type,
    pub cardinality: Cardinality,
    pub borrow: TargetBorrow,
}

#[derive(Clone, Debug)]
pub(crate) struct UnwrappedFieldTarget {
    pub raw_type: Type,
    pub validate_type: Type,
    pub borrow: TargetBorrow,
}

#[derive(Clone, Debug)]
pub(crate) struct FullElementTarget {
    pub ty: Type,
    pub cardinality: Cardinality,
    pub borrow: TargetBorrow,
}

#[derive(Clone, Debug)]
pub(crate) struct UnwrappedElementTarget {
    pub raw_type: Type,
    pub validate_type: Type,
    pub borrow: TargetBorrow,
}

impl ValidationTarget {
    fn for_validator(
        validator: &ValidatorAttr,
        target_selector: &ValidatorTargetSelector,
        target_context: &FieldTargetContext<'_>,
        scope: ValidationTargetScope,
        field_name: &Ident,
    ) -> Result<Self, syn::Error> {
        let value_context = target_context.for_scope(scope);

        Self::reject_option_target_type_arg_on_default_target(
            validator,
            target_selector,
            scope,
            field_name,
        )?;

        if target_selector.is_full() {
            if validator
                .explicit_type()
                .is_some_and(|ty| option_inner_type(ty).is_some())
                && option_inner_type(&value_context.raw_ty).is_none()
            {
                let target_context = if scope.is_element() {
                    format!("element validators on field `{field_name}`")
                } else {
                    format!("field `{field_name}`")
                };
                return Err(syn::Error::new(
                    target_selector
                        .marker_span()
                        .unwrap_or_else(|| validator.path().span()),
                    format!(
                        "explicit `Option<...>` validator type arguments require an optional validation target for {target_context}; `{}` is targeting a non-optional value",
                        validator.path_name()
                    ),
                ));
            }

            return Ok(match scope {
                ValidationTargetScope::Field => Self::FieldFull(FullFieldTarget {
                    ty: value_context.raw_ty.clone(),
                    cardinality: value_context.cardinality,
                    borrow: TargetBorrow::Reference,
                }),
                ValidationTargetScope::Element => Self::ElementFull(FullElementTarget {
                    ty: value_context.raw_ty.clone(),
                    cardinality: value_context.cardinality,
                    borrow: TargetBorrow::AlreadyBorrowed,
                }),
            });
        }

        Ok(match scope {
            ValidationTargetScope::Field => Self::FieldUnwrapped(UnwrappedFieldTarget {
                raw_type: value_context.raw_ty.clone(),
                validate_type: value_context.inner_ty.clone(),
                borrow: TargetBorrow::AlreadyBorrowed,
            }),
            ValidationTargetScope::Element => Self::ElementUnwrapped(UnwrappedElementTarget {
                raw_type: value_context.raw_ty.clone(),
                validate_type: value_context.inner_ty.clone(),
                borrow: TargetBorrow::AlreadyBorrowed,
            }),
        })
    }

    fn reject_option_target_type_arg_on_default_target(
        validator: &ValidatorAttr,
        target_selector: &ValidatorTargetSelector,
        scope: ValidationTargetScope,
        field_name: &Ident,
    ) -> Result<(), syn::Error> {
        let Some(explicit_ty) = validator.explicit_type() else {
            return Ok(());
        };
        if option_inner_type(explicit_ty).is_none() {
            return Ok(());
        }
        if target_selector.is_full() {
            return Ok(());
        }

        let target_context = if scope.is_element() {
            format!("element validators on field `{field_name}`")
        } else {
            format!("field `{field_name}`")
        };
        let validator_name = validator.path_name();

        Err(syn::Error::new_spanned(
            explicit_ty,
            format!(
                "explicit `Option<...>` validator type arguments require `full(...)` target selection for {target_context}; write `full({validator_name}::<Option<_>>)` or use `::<_>` inside `full(...)`"
            ),
        ))
    }

    fn resolve_explicit_infer_type(
        &self,
        validator: &ValidatorAttr,
    ) -> Result<Option<Type>, syn::Error> {
        let Some(explicit_ty) = validator.explicit_type() else {
            return Ok(None);
        };

        if !contains_infer_type(explicit_ty) {
            return Ok(None);
        }

        let validate_type = self.validate_type();
        substitute_infer_type_from_source(explicit_ty, validate_type)
            .map(Some)
            .ok_or_else(|| {
                let rendered_explicit = quote! { #explicit_ty }.to_string();
                let infer_source = validate_type;
                let rendered_source = quote! { #infer_source }.to_string();
                syn::Error::new_spanned(
                    explicit_ty,
                    format!(
                        "cannot infer `_` in `{rendered_explicit}` from `{rendered_source}`; use concrete type arguments or a matching generic shape"
                    ),
                )
            })
    }

    pub(crate) fn raw_type(&self) -> &Type {
        match self {
            Self::FieldFull(target) => &target.ty,
            Self::FieldUnwrapped(target) => &target.raw_type,
            Self::ElementFull(target) => &target.ty,
            Self::ElementUnwrapped(target) => &target.raw_type,
        }
    }

    pub(crate) fn validate_type(&self) -> &Type {
        match self {
            Self::FieldFull(target) => &target.ty,
            Self::FieldUnwrapped(target) => &target.validate_type,
            Self::ElementFull(target) => &target.ty,
            Self::ElementUnwrapped(target) => &target.validate_type,
        }
    }

    pub(crate) fn borrow(&self) -> TargetBorrow {
        match self {
            Self::FieldFull(target) => target.borrow,
            Self::FieldUnwrapped(target) => target.borrow,
            Self::ElementFull(target) => target.borrow,
            Self::ElementUnwrapped(target) => target.borrow,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ValidationTargetScope {
    Field,
    Element,
}

impl ValidationTargetScope {
    pub(crate) fn is_element(self) -> bool {
        self == Self::Element
    }
}

#[derive(Clone, Debug)]
pub(crate) enum PlannedValidatorTypeArg {
    None,
    Resolved(Type),
}

impl PlannedValidatorTypeArg {
    fn as_type(&self) -> Option<&Type> {
        match self {
            PlannedValidatorTypeArg::None => None,
            PlannedValidatorTypeArg::Resolved(ty) => Some(ty),
        }
    }

    fn for_validator(
        validator: &ValidatorAttr,
        target: &ValidationTarget,
        resolved_explicit_type: Option<Type>,
    ) -> Self {
        if let Some(resolved) = resolved_explicit_type {
            return Self::Resolved(resolved);
        }

        if let Some(explicit) = validator.explicit_type() {
            return Self::Resolved(explicit.clone());
        }

        if validator.uses_type_inference() {
            return Self::Resolved(target.validate_type().clone());
        }

        Self::None
    }
}

#[derive(Clone, Debug)]
pub(crate) enum ErrorStorage {
    Nested { cardinality: Cardinality },
    NewtypeInner { cardinality: Cardinality },
    NewtypeWithValidators { cardinality: Cardinality },
    RegularEmpty,
    RegularFieldValidators,
    RegularElementValidators,
    RegularFieldAndElementValidators,
}

impl ErrorStorage {
    fn for_shape(shape: &PlannedField) -> Self {
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
