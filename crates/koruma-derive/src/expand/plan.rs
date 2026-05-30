#![allow(dead_code)]

use heck::ToUpperCamelCase;
use koruma_derive_core::{
    FieldInfo, StructOptions, ValidatorAttr, is_option_type, parse_struct_options,
};
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{DeriveInput, Fields, Ident, Type};

use super::codegen::{
    each_element_type, effective_validation_type, resolve_explicit_infer_type,
    validate_each_collection_type, validate_full_type_option_target, validator_field_ident,
    validator_type_for_field, validator_variant_ident, validator_wants_full_type,
};
use super::collect_field_infos;

#[derive(Clone, Debug)]
pub(crate) struct ValidationPlan {
    pub struct_options: StructOptions,
    pub field_infos: Vec<FieldInfo>,
    pub fields: Vec<FieldPlan>,
    pub struct_newtype_field_info: Option<FieldInfo>,
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

        let struct_newtype_field_info = if struct_options.is_newtype() {
            Some(
                field_infos
                    .first()
                    .cloned()
                    .expect("single-field struct newtypes should expose one participating field"),
            )
        } else {
            None
        };

        let known_field_names = fields
            .iter()
            .filter_map(|field| field.ident.clone())
            .collect();

        let planned_fields = field_infos
            .iter()
            .map(|field| FieldPlan::build(&input.ident, field))
            .collect::<Result<_, _>>()?;

        Ok(Self {
            struct_options,
            field_infos,
            fields: planned_fields,
            struct_newtype_field_info,
            known_field_names,
        })
    }

    pub fn field_infos(&self) -> &[FieldInfo] {
        &self.field_infos
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
pub(crate) struct FieldPlan {
    pub name: Ident,
    pub mode: PlannedFieldMode,
    pub generated_names: GeneratedNames,
    pub field_validators: Vec<PlannedValidator>,
    pub element_validators: Vec<PlannedValidator>,
    pub error_storage: ErrorStorage,
    pub generates_all_enum: bool,
}

impl FieldPlan {
    fn build(struct_name: &Ident, field: &FieldInfo) -> Result<Self, syn::Error> {
        if field.has_element_validators() {
            validate_each_collection_type(&field.ty)?;
        }

        let field_validators = field
            .validation
            .field_validators
            .iter()
            .map(|validator| PlannedValidator::build(validator, field, false))
            .collect::<Result<Vec<_>, _>>()?;

        let element_validators = field
            .validation
            .element_validators
            .iter()
            .map(|validator| PlannedValidator::build(validator, field, true))
            .collect::<Result<Vec<_>, _>>()?;

        let mode = if field.is_nested() {
            PlannedFieldMode::Nested
        } else if field.is_newtype() {
            PlannedFieldMode::Newtype
        } else {
            PlannedFieldMode::Regular
        };

        let error_storage = match mode {
            PlannedFieldMode::Nested => ErrorStorage::Nested,
            PlannedFieldMode::Newtype => ErrorStorage::Newtype {
                optional: is_option_type(&field.ty),
                has_field_validators: !field.validation.field_validators.is_empty(),
            },
            PlannedFieldMode::Regular => ErrorStorage::Regular {
                has_field_validators: !field.validation.field_validators.is_empty(),
                has_element_validators: field.has_element_validators(),
            },
        };

        Ok(Self {
            name: field.name.clone(),
            mode,
            generated_names: GeneratedNames::new(struct_name, field),
            generates_all_enum: !field.validation.field_validators.is_empty()
                || field.has_element_validators(),
            field_validators,
            element_validators,
            error_storage,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PlannedFieldMode {
    Regular,
    Nested,
    Newtype,
}

#[derive(Clone, Debug)]
pub(crate) struct GeneratedNames {
    pub field_error_struct: Ident,
    pub field_validator_enum: Ident,
    pub element_error_struct: Ident,
    pub element_validator_enum: Ident,
}

impl GeneratedNames {
    fn new(struct_name: &Ident, field: &FieldInfo) -> Self {
        let field_stem = field.name.to_string().to_upper_camel_case();
        Self {
            field_error_struct: format_ident!("{struct_name}{field_stem}KorumaValidationError"),
            field_validator_enum: format_ident!("{struct_name}{field_stem}KorumaValidator"),
            element_error_struct: format_ident!(
                "{struct_name}{field_stem}ElementKorumaValidationError"
            ),
            element_validator_enum: format_ident!(
                "{struct_name}{field_stem}ElementKorumaValidator"
            ),
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct PlannedValidator {
    pub attr: ValidatorAttr,
    pub target: ValidationTarget,
    pub resolved_type_arg: PlannedValidatorTypeArg,
    pub validator_type: TokenStream2,
    pub field_ident: Ident,
    pub variant_ident: Ident,
    pub getter_return_type: TokenStream2,
}

impl PlannedValidator {
    fn build(
        validator: &ValidatorAttr,
        field: &FieldInfo,
        validate_each: bool,
    ) -> Result<Self, syn::Error> {
        validate_full_type_option_target(validator, &field.ty, validate_each, &field.name)?;
        let resolved_explicit_type =
            resolve_explicit_infer_type(validator, &field.ty, validate_each)?;
        let target = ValidationTarget::for_validator(validator, &field.ty, validate_each);
        let validator_type = validator_type_for_field(validator, &field.ty, validate_each);
        let siblings = if validate_each {
            &field.validation.element_validators
        } else {
            &field.validation.field_validators
        };

        Ok(Self {
            attr: validator.clone(),
            target,
            resolved_type_arg: PlannedValidatorTypeArg::for_validator(
                validator,
                &field.ty,
                validate_each,
                resolved_explicit_type,
            ),
            validator_type: validator_type.clone(),
            field_ident: validator_field_ident(validator, siblings),
            variant_ident: validator_variant_ident(validator, siblings),
            getter_return_type: quote! { Option<&#validator_type> },
        })
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
    fn for_validator(validator: &ValidatorAttr, field_ty: &Type, validate_each: bool) -> Self {
        if validate_each {
            let element_ty = each_element_type(field_ty);
            if validator_wants_full_type(validator) {
                Self::ElementFull
            } else if is_option_type(element_ty) {
                Self::ElementOptionalInner
            } else {
                Self::ElementUnwrapped
            }
        } else if validator_wants_full_type(validator) {
            Self::FieldFull
        } else if is_option_type(field_ty) {
            Self::FieldOptionalInner
        } else {
            Self::FieldUnwrapped
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) enum PlannedValidatorTypeArg {
    None,
    Resolved(Type),
}

impl PlannedValidatorTypeArg {
    fn for_validator(
        validator: &ValidatorAttr,
        field_ty: &Type,
        validate_each: bool,
        resolved_explicit_type: Option<Type>,
    ) -> Self {
        if let Some(resolved) = resolved_explicit_type {
            return Self::Resolved(resolved);
        }

        if let Some(explicit) = validator.explicit_type() {
            return Self::Resolved(explicit.clone());
        }

        if validator.uses_type_inference() {
            return Self::Resolved(effective_validation_type(field_ty, validate_each).clone());
        }

        Self::None
    }
}

#[derive(Clone, Debug)]
pub(crate) enum ErrorStorage {
    Nested,
    Newtype {
        optional: bool,
        has_field_validators: bool,
    },
    Regular {
        has_field_validators: bool,
        has_element_validators: bool,
    },
}
