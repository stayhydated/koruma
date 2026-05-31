#![allow(dead_code)]

use heck::ToUpperCamelCase;
use koruma_derive_core::{
    BuilderMethodCall, FieldInfo, StructOptions, ValidatorAttr, option_inner_type,
    parse_struct_options,
};
use quote::format_ident;
use syn::{DeriveInput, Expr, Fields, Ident, Path, Type};

use super::codegen::{
    EachIterationKind, FieldCardinality, ValidationSite, classify_each_collection,
    reject_ambiguous_option_target_type_arg, resolve_explicit_infer_type,
    validate_validator_arg_value, validator_field_ident, validator_infer_source_type,
    validator_variant_ident, validator_wants_full_type,
};
use super::collect_field_infos;

#[derive(Clone, Debug)]
pub(crate) struct ValidationPlan {
    pub struct_options: StructOptions,
    pub struct_plan: StructPlan,
    pub field_infos: Vec<FieldInfo>,
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

        let planned_fields: Vec<FieldPlan> = field_infos
            .iter()
            .map(|field| FieldPlan::build(&input.ident, field, &known_field_names))
            .collect::<Result<_, _>>()?;

        let struct_plan = if struct_options.is_newtype() {
            let Some(field_info) = field_infos.first().cloned() else {
                return Err(syn::Error::new_spanned(
                    input,
                    "newtype structs require their only field to participate in validation",
                ));
            };
            let Some(field) = planned_fields.first().cloned() else {
                return Err(syn::Error::new_spanned(
                    input,
                    "newtype structs require a field validation plan",
                ));
            };
            StructPlan::Newtype { field_info, field }
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
            field_infos,
            fields: planned_fields,
            known_field_names,
        })
    }

    pub fn field_infos(&self) -> &[FieldInfo] {
        &self.field_infos
    }

    pub fn field_plan(&self, name: &Ident) -> Option<&FieldPlan> {
        self.fields.iter().find(|field| &field.name == name)
    }

    pub fn struct_newtype(&self) -> Option<(&FieldInfo, &FieldPlan)> {
        match &self.struct_plan {
            StructPlan::Newtype { field_info, field } => Some((field_info, field)),
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
    Newtype {
        field_info: FieldInfo,
        field: FieldPlan,
    },
}

#[derive(Clone, Debug)]
pub(crate) struct FieldPlan {
    pub name: Ident,
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
        let each_collection = field
            .has_element_validators()
            .then(|| classify_each_collection(&field.ty))
            .transpose()?;

        let field_validators = field
            .validation
            .field_validators
            .iter()
            .map(|validator| {
                PlannedValidator::build(validator, field, ValidationSite::Field, known_field_names)
            })
            .collect::<Result<Vec<_>, _>>()?;

        let element_validators = field
            .validation
            .element_validators
            .iter()
            .map(|validator| {
                PlannedValidator::build(
                    validator,
                    field,
                    ValidationSite::Element,
                    known_field_names,
                )
            })
            .collect::<Result<Vec<_>, _>>()?;

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
        let generated_names = GeneratedNames::new(struct_name, field);

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
        self.error_storage
            .cardinality()
            .unwrap_or_else(|| match &self.shape {
                PlannedField::Regular(plan) => plan.cardinality,
                PlannedField::Nested(_) | PlannedField::Newtype(_) => {
                    unreachable!("nested and newtype storage carries cardinality")
                },
            })
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
            .filter(|validator| validator.target == ValidationTarget::FieldFull)
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
        self.field_validators().iter().filter(|validator| {
            matches!(
                validator.target,
                ValidationTarget::FieldUnwrapped | ValidationTarget::FieldOptionalInner
            )
        })
    }

    pub(crate) fn full_element_validators(&self) -> impl Iterator<Item = &PlannedValidator> {
        self.element_validators()
            .iter()
            .filter(|validator| validator.target == ValidationTarget::ElementFull)
    }

    pub(crate) fn unwrapped_element_validators(&self) -> impl Iterator<Item = &PlannedValidator> {
        self.element_validators().iter().filter(|validator| {
            matches!(
                validator.target,
                ValidationTarget::ElementUnwrapped | ValidationTarget::ElementOptionalInner
            )
        })
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
pub(crate) struct GeneratedNames {
    pub field_error_struct: Ident,
    pub field_validator_ref_enum: Ident,
    pub element_error_struct: Ident,
    pub element_validator_ref_enum: Ident,
}

impl GeneratedNames {
    fn new(struct_name: &Ident, field: &FieldInfo) -> Self {
        let field_stem = field.name.to_string().to_upper_camel_case();
        Self {
            field_error_struct: format_ident!("{struct_name}{field_stem}KorumaValidationError"),
            field_validator_ref_enum: format_ident!("{struct_name}{field_stem}KorumaValidatorRef"),
            element_error_struct: format_ident!(
                "{struct_name}{field_stem}ElementKorumaValidationError"
            ),
            element_validator_ref_enum: format_ident!(
                "{struct_name}{field_stem}ElementKorumaValidatorRef"
            ),
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct PlannedValidator {
    pub attr: ValidatorAttr,
    pub target: ValidationTarget,
    pub validation_target_type: Type,
    pub resolved_type_arg: PlannedValidatorTypeArg,
    pub validator_type: PlannedValidatorType,
    pub builder_type: PlannedValidatorType,
    pub setter_calls: Vec<PlannedSetterCall>,
    pub field_ident: Ident,
    pub variant_ident: Ident,
}

impl PlannedValidator {
    fn build(
        validator: &ValidatorAttr,
        field: &FieldInfo,
        site: ValidationSite,
        known_field_names: &[Ident],
    ) -> Result<Self, syn::Error> {
        reject_ambiguous_option_target_type_arg(validator, site, &field.name)?;
        let resolved_explicit_type = resolve_explicit_infer_type(validator, &field.ty, site)?;
        let target = ValidationTarget::for_validator(validator, &field.ty, site)?;
        let validation_target_type =
            validator_infer_source_type(validator, &field.ty, site)?.clone();
        let resolved_type_arg = PlannedValidatorTypeArg::for_validator(
            validator,
            &field.ty,
            site,
            resolved_explicit_type.clone(),
        )?;
        let concrete_type_arg =
            concrete_validator_type_arg(validator, &field.ty, site, resolved_explicit_type)?;
        let validator_type =
            PlannedValidatorType::new(validator.validator.clone(), concrete_type_arg.as_ref());
        let builder_type_arg =
            builder_validator_type_arg(validator, concrete_type_arg.as_ref().cloned());
        let builder_type =
            PlannedValidatorType::new(validator.validator.clone(), builder_type_arg.as_ref());
        let setter_calls = planned_setter_calls(validator.setter_calls(), known_field_names)?;
        let siblings = if site.is_element() {
            &field.validation.element_validators
        } else {
            &field.validation.field_validators
        };

        Ok(Self {
            attr: validator.clone(),
            target,
            validation_target_type,
            resolved_type_arg,
            validator_type: validator_type.clone(),
            builder_type,
            setter_calls,
            field_ident: validator_field_ident(validator, siblings),
            variant_ident: validator_variant_ident(validator, siblings),
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
    calls
        .iter()
        .map(|call| {
            for arg in &call.args {
                validate_validator_arg_value(arg, known_field_names)?;
            }

            Ok(PlannedSetterCall {
                method: call.method.clone(),
                args: call.args.clone(),
            })
        })
        .collect()
}

#[derive(Clone, Debug)]
pub(crate) struct PlannedSetterCall {
    pub method: Ident,
    pub args: Vec<Expr>,
}

fn concrete_validator_type_arg(
    validator: &ValidatorAttr,
    field_ty: &Type,
    site: ValidationSite,
    resolved_explicit_type: Option<Type>,
) -> Result<Option<Type>, syn::Error> {
    if let Some(resolved) = resolved_explicit_type {
        return Ok(Some(resolved));
    }

    if let Some(explicit) = validator.explicit_type() {
        return Ok(Some(explicit.clone()));
    }

    if validator.uses_type_inference() {
        return Ok(Some(
            validator_infer_source_type(validator, field_ty, site)?.clone(),
        ));
    }

    Ok(None)
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ValidationTarget {
    FieldFull,
    FieldUnwrapped,
    FieldOptionalInner,
    ElementFull,
    ElementUnwrapped,
    ElementOptionalInner,
}

impl ValidationTarget {
    fn for_validator(
        validator: &ValidatorAttr,
        field_ty: &Type,
        site: ValidationSite,
    ) -> Result<Self, syn::Error> {
        if site.is_element() {
            let element_ty = classify_each_collection(field_ty)?.element_ty;
            let target = if validator_wants_full_type(validator) {
                Self::ElementFull
            } else if FieldCardinality::for_type(element_ty).is_optional() {
                Self::ElementOptionalInner
            } else {
                Self::ElementUnwrapped
            };
            Ok(target)
        } else if validator_wants_full_type(validator) {
            Ok(Self::FieldFull)
        } else if FieldCardinality::for_type(field_ty).is_optional() {
            Ok(Self::FieldOptionalInner)
        } else {
            Ok(Self::FieldUnwrapped)
        }
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
        field_ty: &Type,
        site: ValidationSite,
        resolved_explicit_type: Option<Type>,
    ) -> Result<Self, syn::Error> {
        if let Some(resolved) = resolved_explicit_type {
            return Ok(Self::Resolved(resolved));
        }

        if let Some(explicit) = validator.explicit_type() {
            return Ok(Self::Resolved(explicit.clone()));
        }

        if validator.uses_type_inference() {
            return Ok(Self::Resolved(
                validator_infer_source_type(validator, field_ty, site)?.clone(),
            ));
        }

        Ok(Self::None)
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
