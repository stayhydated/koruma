use std::collections::HashMap;

use heck::{ToSnakeCase, ToUpperCamelCase};
use koruma_derive_core::{FieldInfo, ParsedValidatorUse};
use quote::format_ident;
use syn::{Error, Ident, Result};

use super::error_bag::ErrorBag;
use super::generated_api::{
    GeneratedApiNameKind, GeneratedApiNamespace, RegisteredApiName, reserved_error_api_name,
    seed_existing_fields,
};

#[derive(Clone, Debug)]
pub(crate) struct GeneratedNames {
    pub field_error_struct: Ident,
    pub field_validator_ref_enum: Ident,
    pub element_error_struct: Ident,
    pub element_validator_ref_enum: Ident,
}

impl GeneratedNames {
    pub(crate) fn for_field(struct_name: &Ident, field: &FieldInfo) -> Self {
        let field_stem = field.name().to_string().to_upper_camel_case();
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
pub(crate) struct GeneratedDeriveApi {
    pub main_error_struct: Ident,
    fields: HashMap<usize, GeneratedNames>,
}

impl GeneratedDeriveApi {
    pub(crate) fn build(struct_name: &Ident, fields: &[FieldInfo]) -> Result<Self> {
        let mut namespace = GeneratedApiNamespace::new();
        let main_error_struct = main_error_struct_ident(struct_name);
        namespace.register_ident(
            &main_error_struct,
            GeneratedApiNameKind::MainErrorStruct,
            |existing| generated_type_collision_message(&main_error_struct, existing),
        )?;

        let mut planned_fields = HashMap::new();
        for field in fields {
            let names = GeneratedNames::for_field(struct_name, field);
            register_generated_field_names(&mut namespace, &names)?;
            planned_fields.insert(field.index(), names);
        }

        Ok(Self {
            main_error_struct,
            fields: planned_fields,
        })
    }

    pub(crate) fn field_names(&self, field: &FieldInfo) -> GeneratedNames {
        self.fields
            .get(&field.index())
            .cloned()
            .expect("generated derive API should have names for each parsed field")
    }
}

pub(crate) fn main_error_struct_ident(struct_name: &Ident) -> Ident {
    format_ident!("{}KorumaValidationError", struct_name)
}

pub(crate) fn tuple_field_ident(index: usize) -> Ident {
    format_ident!("_{}", index)
}

#[derive(Clone, Debug)]
pub(crate) struct ValidatorGeneratedNames {
    pub field_ident: Ident,
    pub variant_ident: Ident,
}

pub(crate) fn validator_names(
    validator_use: &ParsedValidatorUse,
    siblings: &[ParsedValidatorUse],
    known_field_names: &[Ident],
) -> Result<ValidatorGeneratedNames> {
    let _ = (siblings, known_field_names);

    let label_span = validator_use.label_span();
    let field_name = validator_field_name(validator_use);
    let variant_name = validator_variant_name(validator_use);
    let field_ident = match label_span {
        Some(span) => format_ident!("{}", field_name, span = span),
        None => format_ident!("{}", field_name),
    };
    let variant_ident = match label_span {
        Some(span) => format_ident!("{}", variant_name, span = span),
        None => format_ident!("{}", variant_name),
    };

    Ok(ValidatorGeneratedNames {
        field_ident,
        variant_ident,
    })
}

pub(crate) fn validate_validator_uses(
    siblings: &[ParsedValidatorUse],
    known_field_names: &[Ident],
) -> Result<()> {
    let mut namespace = seed_existing_fields(known_field_names);
    let mut errors = ErrorBag::new();

    for validator_use in siblings {
        errors.push_result(validate_reserved_label(validator_use));
        errors.push_result(register_validator_names(&mut namespace, validator_use));
    }

    errors.finish()
}

fn register_validator_names(
    namespace: &mut super::generated_api::GeneratedApiNamespace,
    validator_use: &ParsedValidatorUse,
) -> Result<()> {
    let field_name = validator_field_name(validator_use);
    let variant_name = validator_variant_name(validator_use);

    if validator_use.label().is_none() && reserved_error_api_name(&field_name) {
        return Err(Error::new(
            validator_use
                .label_span()
                .unwrap_or_else(|| validator_use.source_span()),
            format!(
                "`{field_name}` is reserved by generated Koruma error APIs; use a different validator label"
            ),
        ));
    }

    let field_ident = generated_ident(validator_use, &field_name);
    namespace.register_ident(
        &field_ident,
        GeneratedApiNameKind::ValidatorGetter,
        |existing| validator_collision_message(validator_use, &field_name, &variant_name, existing),
    )?;

    let variant_ident = generated_ident(validator_use, &variant_name);
    namespace.register_ident(
        &variant_ident,
        GeneratedApiNameKind::ValidatorVariant,
        |existing| validator_collision_message(validator_use, &field_name, &variant_name, existing),
    )?;

    Ok(())
}

fn register_generated_field_names(
    namespace: &mut GeneratedApiNamespace,
    names: &GeneratedNames,
) -> Result<()> {
    namespace.register_ident(
        &names.field_error_struct,
        GeneratedApiNameKind::FieldErrorStruct,
        |existing| generated_type_collision_message(&names.field_error_struct, existing),
    )?;
    namespace.register_ident(
        &names.field_validator_ref_enum,
        GeneratedApiNameKind::FieldValidatorRefEnum,
        |existing| generated_type_collision_message(&names.field_validator_ref_enum, existing),
    )?;
    namespace.register_ident(
        &names.element_error_struct,
        GeneratedApiNameKind::ElementErrorStruct,
        |existing| generated_type_collision_message(&names.element_error_struct, existing),
    )?;
    namespace.register_ident(
        &names.element_validator_ref_enum,
        GeneratedApiNameKind::ElementValidatorRefEnum,
        |existing| generated_type_collision_message(&names.element_validator_ref_enum, existing),
    )
}

fn generated_type_collision_message(requested: &Ident, existing: &RegisteredApiName) -> String {
    format!(
        "generated API type `{requested}` collides with generated {} `{}`",
        generated_name_kind_label(existing.kind),
        existing.ident
    )
}

fn generated_name_kind_label(kind: GeneratedApiNameKind) -> &'static str {
    match kind {
        GeneratedApiNameKind::MainErrorStruct => "main error struct",
        GeneratedApiNameKind::FieldErrorStruct => "field error struct",
        GeneratedApiNameKind::FieldValidatorRefEnum => "field validator enum",
        GeneratedApiNameKind::ElementErrorStruct => "element error struct",
        GeneratedApiNameKind::ElementValidatorRefEnum => "element validator enum",
        GeneratedApiNameKind::ValidatorGetter => "validator getter",
        GeneratedApiNameKind::ValidatorVariant => "validator enum variant",
        GeneratedApiNameKind::ExistingField => "input field",
        GeneratedApiNameKind::BuilderType => "builder type",
        GeneratedApiNameKind::BuilderModule => "builder module",
        GeneratedApiNameKind::BuilderMethod => "builder method",
        GeneratedApiNameKind::OptionalBuilderMethod => "optional builder method",
        GeneratedApiNameKind::ReservedBuilderMethod => "reserved builder method",
        GeneratedApiNameKind::UserGeneric => "user generic",
        GeneratedApiNameKind::RequiredStateGeneric => "required state generic",
    }
}

fn validate_reserved_label(validator_use: &ParsedValidatorUse) -> Result<()> {
    let Some(label) = validator_use.label() else {
        return Ok(());
    };
    let label_text = label.to_string();

    if reserved_error_api_name(&label_text) {
        return Err(Error::new(
            label.span(),
            format!(
                "`{label_text}` is reserved by generated Koruma error APIs; use a different validator label"
            ),
        ));
    }

    Ok(())
}

fn validator_collision_message(
    validator_use: &ParsedValidatorUse,
    field_name: &str,
    variant_name: &str,
    existing: &RegisteredApiName,
) -> String {
    if existing.kind == GeneratedApiNameKind::ExistingField {
        return format!(
            "validator label `{field_name}` conflicts with a generated field name; use a different label"
        );
    }

    if validator_use.label().is_some() {
        return format!(
            "validator label `{field_name}` collides with another validator getter or `{variant_name}` enum variant in this field; use a unique label"
        );
    }

    format!(
        "validator `{}` generates duplicate getter `{field_name}` or `{variant_name}` enum variant in this field; add explicit validator labels such as `label_name = Validator`",
        validator_use.validator().path_name()
    )
}

fn generated_ident(validator_use: &ParsedValidatorUse, name: &str) -> Ident {
    match validator_use.label_span() {
        Some(span) => format_ident!("{}", name, span = span),
        None => format_ident!("{}", name, span = validator_use.source_span()),
    }
}

fn validator_field_name(validator_use: &ParsedValidatorUse) -> String {
    validator_use
        .label()
        .map(ToString::to_string)
        .unwrap_or_else(|| validator_use.validator().name().to_string().to_snake_case())
}

fn validator_variant_name(validator_use: &ParsedValidatorUse) -> String {
    validator_use
        .label()
        .map(|label| label.to_string().to_upper_camel_case())
        .unwrap_or_else(|| {
            validator_use
                .validator()
                .name()
                .to_string()
                .to_upper_camel_case()
        })
}
