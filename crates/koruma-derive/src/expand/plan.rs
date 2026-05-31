#![allow(dead_code)]

use koruma_derive_core::{
    BuilderMethodCall, FieldInfo, ParsedValidatorUse, StructOptions, ValidatorAttr,
    contains_infer_type, option_inner_type, parse_struct_options,
    substitute_infer_type_from_source,
};
use quote::quote;
use syn::{DeriveInput, Expr, Fields, Ident, Member, Path, Type};

use super::codegen::{
    EachIterationKind, FieldCardinality, classify_each_collection, validate_validator_arg_value,
};
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
    pub error_storage: ErrorStorage,
    pub generates_all_enum: bool,
}

impl FieldPlan {
    fn build(
        struct_name: &Ident,
        field: &FieldInfo,
        known_field_names: &[Ident],
    ) -> Result<Self, syn::Error> {
        let mut errors = ErrorBag::new();
        let each_collection = if field.has_element_validators() {
            errors.push_result(classify_each_collection(&field.ty))
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

        let field_validators = plan_validators(
            field.field_validators(),
            field,
            TargetScope::Field,
            known_field_names,
        )?;

        let element_validators = plan_validators(
            field.element_validators(),
            field,
            TargetScope::Element,
            known_field_names,
        )?;

        let field_cardinality = each_collection
            .as_ref()
            .map(|collection| collection.outer_cardinality)
            .unwrap_or_else(|| FieldCardinality::for_type(&field.ty));
        let inner_type = option_inner_type(&field.ty).unwrap_or(&field.ty).clone();
        let collection_type = each_collection
            .as_ref()
            .map(|collection| collection.collection_ty.clone());
        let element_type = each_collection
            .as_ref()
            .map(|collection| collection.element_ty.clone());
        let element_cardinality = each_collection
            .as_ref()
            .map(|collection| collection.element_cardinality)
            .unwrap_or(FieldCardinality::Required);
        let each_iteration = each_collection
            .as_ref()
            .map(|collection| collection.iteration);
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
                collection_type,
                element_type,
                element_cardinality,
                each_iteration,
                field_validators,
                element_validators,
            })
        };

        let error_storage = ErrorStorage::for_shape(&shape);
        let generates_all_enum = error_storage.generates_all_enum();

        Ok(Self {
            name: field.name.clone(),
            source: FieldSource {
                name: field.name.clone(),
                member: field.member.clone(),
                ty: field.ty.clone(),
                index: field.index,
            },
            generated_names,
            shape,
            error_storage,
            generates_all_enum,
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

    pub(crate) fn field_cardinality(&self) -> FieldCardinality {
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

    pub(crate) fn element_cardinality(&self) -> FieldCardinality {
        match &self.shape {
            PlannedField::Regular(plan) => plan.element_cardinality,
            PlannedField::Nested(_) | PlannedField::Newtype(_) => FieldCardinality::Required,
        }
    }

    pub(crate) fn full_field_validators(&self) -> impl Iterator<Item = &PlannedValidator> {
        self.field_validators()
            .iter()
            .filter(|validator| validator.target.is_field_full())
    }

    pub(crate) fn is_nested(&self) -> bool {
        self.error_storage.is_nested()
    }

    pub(crate) fn is_newtype(&self) -> bool {
        self.error_storage.is_newtype()
    }

    pub(crate) fn has_field_validators(&self) -> bool {
        self.error_storage.has_field_validator_slots()
    }

    pub(crate) fn has_element_validators(&self) -> bool {
        self.error_storage.has_element_error_slots()
    }

    pub(crate) fn unwrapped_field_validators(&self) -> impl Iterator<Item = &PlannedValidator> {
        self.field_validators()
            .iter()
            .filter(|validator| validator.target.is_field_unwrapped())
    }

    pub(crate) fn full_element_validators(&self) -> impl Iterator<Item = &PlannedValidator> {
        self.element_validators()
            .iter()
            .filter(|validator| validator.target.is_element_full())
    }

    pub(crate) fn unwrapped_element_validators(&self) -> impl Iterator<Item = &PlannedValidator> {
        self.element_validators()
            .iter()
            .filter(|validator| validator.target.is_element_unwrapped())
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
    pub cardinality: FieldCardinality,
    pub inner_type: Type,
    pub collection_type: Option<Type>,
    pub element_type: Option<Type>,
    pub element_cardinality: FieldCardinality,
    pub each_iteration: Option<EachIterationKind>,
    pub field_validators: Vec<PlannedValidator>,
    pub element_validators: Vec<PlannedValidator>,
}

#[derive(Clone, Debug)]
pub(crate) struct NestedFieldPlan {
    pub cardinality: FieldCardinality,
    pub inner_type: Type,
}

#[derive(Clone, Debug)]
pub(crate) struct NewtypeFieldPlan {
    pub cardinality: FieldCardinality,
    pub inner_type: Type,
    pub field_validators: Vec<PlannedValidator>,
}

#[derive(Clone, Debug)]
pub(crate) struct PlannedValidator {
    pub attr: ValidatorAttr,
    pub label: Option<Ident>,
    pub target: TargetPlan,
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
    scope: TargetScope,
    known_field_names: &[Ident],
) -> Result<Vec<PlannedValidator>, syn::Error> {
    let mut planned = Vec::new();
    let mut errors = ErrorBag::new();

    for validator in validators {
        if let Some(validator) = errors.push_result(PlannedValidator::build(
            validator,
            field,
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
        scope: TargetScope,
        known_field_names: &[Ident],
    ) -> Result<Self, syn::Error> {
        let validator = &validator_use.validator;
        let target = TargetPlan::for_validator(validator, &field.ty, scope, &field.name)?;
        let resolved_explicit_type = target.resolve_explicit_infer_type(validator)?;
        let resolved_type_arg = PlannedValidatorTypeArg::for_validator(
            validator,
            &target,
            resolved_explicit_type.clone(),
        );
        let concrete_type_arg =
            concrete_validator_type_arg(validator, &target, resolved_explicit_type);
        let validator_type =
            PlannedValidatorType::new(validator.validator.clone(), concrete_type_arg.as_ref());
        let builder_type_arg =
            builder_validator_type_arg(validator, concrete_type_arg.as_ref().cloned());
        let builder_type =
            PlannedValidatorType::new(validator.validator.clone(), builder_type_arg.as_ref());
        let setter_calls = planned_setter_calls(validator.setter_calls(), known_field_names)?;
        let siblings = if scope.is_element() {
            field.element_validators()
        } else {
            field.field_validators()
        };
        let generated_names = validator_names(validator_use, siblings, known_field_names)?;

        Ok(Self {
            attr: validator.clone(),
            label: validator_use.label.clone(),
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
        let mut call_valid = true;
        for arg in &call.args {
            if let Err(error) = validate_validator_arg_value(arg, known_field_names) {
                errors.push(error);
                call_valid = false;
            }
        }

        if call_valid {
            planned.push(PlannedSetterCall {
                method: call.method.clone(),
                args: call.args.clone(),
            });
        }
    }

    errors.finish()?;
    Ok(planned)
}

#[derive(Clone, Debug)]
pub(crate) struct PlannedSetterCall {
    pub method: Ident,
    pub args: Vec<Expr>,
}

fn concrete_validator_type_arg(
    validator: &ValidatorAttr,
    target: &TargetPlan,
    resolved_explicit_type: Option<Type>,
) -> Option<Type> {
    if let Some(resolved) = resolved_explicit_type {
        return Some(resolved);
    }

    if let Some(explicit) = validator.explicit_type() {
        return Some(explicit.clone());
    }

    if validator.uses_type_inference() {
        return Some(target.validate_type.clone());
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
pub(crate) struct TargetPlan {
    pub scope: TargetScope,
    pub policy: TargetPolicy,
    pub raw_type: Type,
    pub validate_type: Type,
    pub cardinality: TargetCardinality,
    pub access: TargetAccess,
}

impl TargetPlan {
    fn for_validator(
        validator: &ValidatorAttr,
        field_ty: &Type,
        scope: TargetScope,
        field_name: &Ident,
    ) -> Result<Self, syn::Error> {
        let raw_type = if scope.is_element() {
            classify_each_collection(field_ty)?.element_ty.clone()
        } else {
            field_ty.clone()
        };
        let policy = if validator.wants_full_target() {
            TargetPolicy::Full
        } else {
            TargetPolicy::UnwrapOption
        };

        Self::reject_ambiguous_option_target_type_arg(validator, scope, policy, field_name)?;

        let validate_type = if policy == TargetPolicy::Full {
            raw_type.clone()
        } else {
            option_inner_type(&raw_type).unwrap_or(&raw_type).clone()
        };
        let cardinality = TargetCardinality::for_type(&raw_type);
        let access = if scope == TargetScope::Field && policy == TargetPolicy::Full {
            TargetAccess::BorrowField
        } else {
            TargetAccess::AlreadyBorrowedLocal
        };

        Ok(Self {
            scope,
            policy,
            raw_type,
            validate_type,
            cardinality,
            access,
        })
    }

    fn reject_ambiguous_option_target_type_arg(
        validator: &ValidatorAttr,
        scope: TargetScope,
        policy: TargetPolicy,
        field_name: &Ident,
    ) -> Result<(), syn::Error> {
        if policy == TargetPolicy::Full {
            return Ok(());
        }

        let Some(explicit_ty) = validator.explicit_type() else {
            return Ok(());
        };
        if option_inner_type(explicit_ty).is_none() {
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
                "explicit `Option<...>` validator type arguments do not request full-target validation for {target_context}; use `full({validator_name}::<_>)` instead"
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

        substitute_infer_type_from_source(explicit_ty, &self.validate_type)
            .map(Some)
            .ok_or_else(|| {
                let rendered_explicit = quote! { #explicit_ty }.to_string();
                let infer_source = &self.validate_type;
                let rendered_source = quote! { #infer_source }.to_string();
                syn::Error::new_spanned(
                    explicit_ty,
                    format!(
                        "cannot infer `_` in `{rendered_explicit}` from `{rendered_source}`; use concrete type arguments or a matching generic shape"
                    ),
                )
            })
    }

    pub(crate) fn is_field_full(&self) -> bool {
        self.scope == TargetScope::Field && self.policy == TargetPolicy::Full
    }

    pub(crate) fn is_field_unwrapped(&self) -> bool {
        self.scope == TargetScope::Field && self.policy == TargetPolicy::UnwrapOption
    }

    pub(crate) fn is_element_full(&self) -> bool {
        self.scope == TargetScope::Element && self.policy == TargetPolicy::Full
    }

    pub(crate) fn is_element_unwrapped(&self) -> bool {
        self.scope == TargetScope::Element && self.policy == TargetPolicy::UnwrapOption
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TargetScope {
    Field,
    Element,
}

impl TargetScope {
    pub(crate) fn is_element(self) -> bool {
        self == Self::Element
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TargetPolicy {
    Full,
    UnwrapOption,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TargetCardinality {
    Required,
    Optional,
}

impl TargetCardinality {
    fn for_type(ty: &Type) -> Self {
        match FieldCardinality::for_type(ty) {
            FieldCardinality::Required => Self::Required,
            FieldCardinality::Optional => Self::Optional,
        }
    }

    pub(crate) fn is_optional(self) -> bool {
        self == Self::Optional
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TargetAccess {
    BorrowField,
    BorrowLocal,
    AlreadyBorrowedLocal,
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
        target: &TargetPlan,
        resolved_explicit_type: Option<Type>,
    ) -> Self {
        if let Some(resolved) = resolved_explicit_type {
            return Self::Resolved(resolved);
        }

        if let Some(explicit) = validator.explicit_type() {
            return Self::Resolved(explicit.clone());
        }

        if validator.uses_type_inference() {
            return Self::Resolved(target.validate_type.clone());
        }

        Self::None
    }
}

#[derive(Clone, Debug)]
pub(crate) enum ErrorStorage {
    Nested { cardinality: FieldCardinality },
    NewtypeInner { cardinality: FieldCardinality },
    NewtypeWithValidators { cardinality: FieldCardinality },
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

    pub(crate) fn cardinality(&self) -> Option<FieldCardinality> {
        match self {
            Self::Nested { cardinality }
            | Self::NewtypeInner { cardinality }
            | Self::NewtypeWithValidators { cardinality } => Some(*cardinality),
            Self::RegularEmpty
            | Self::RegularFieldValidators
            | Self::RegularElementValidators
            | Self::RegularFieldAndElementValidators => None,
        }
    }

    pub(crate) fn is_nested(&self) -> bool {
        matches!(self, Self::Nested { .. })
    }

    pub(crate) fn is_newtype(&self) -> bool {
        matches!(
            self,
            Self::NewtypeInner { .. } | Self::NewtypeWithValidators { .. }
        )
    }

    pub(crate) fn has_generated_field_error_struct(&self) -> bool {
        !self.is_nested()
    }

    pub(crate) fn has_field_validator_slots(&self) -> bool {
        matches!(
            self,
            Self::NewtypeWithValidators { .. }
                | Self::RegularFieldValidators
                | Self::RegularFieldAndElementValidators
        )
    }

    pub(crate) fn has_element_error_slots(&self) -> bool {
        matches!(
            self,
            Self::RegularElementValidators | Self::RegularFieldAndElementValidators
        )
    }

    pub(crate) fn stores_inner_error(&self) -> bool {
        matches!(
            self,
            Self::NewtypeInner { .. } | Self::NewtypeWithValidators { .. }
        )
    }

    pub(crate) fn generates_all_enum(&self) -> bool {
        matches!(
            self,
            Self::NewtypeWithValidators { .. }
                | Self::RegularFieldValidators
                | Self::RegularElementValidators
                | Self::RegularFieldAndElementValidators
        )
    }
}
