use heck::{ToSnakeCase, ToUpperCamelCase};
use koruma_derive_core::{FieldInfo, ParsedValidatorUse};
use quote::format_ident;
use syn::{Error, Ident, Result};

use super::error_bag::ErrorBag;

#[derive(Clone, Debug)]
pub(crate) struct GeneratedNames {
    pub field_error_struct: Ident,
    pub field_validator_ref_enum: Ident,
    pub element_error_struct: Ident,
    pub element_validator_ref_enum: Ident,
}

impl GeneratedNames {
    pub(crate) fn for_field(struct_name: &Ident, field: &FieldInfo) -> Self {
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

#[derive(Clone, Debug)]
struct ValidatorNameCandidate {
    field_name: String,
    variant_name: String,
}

pub(crate) fn validator_names(
    validator_use: &ParsedValidatorUse,
    siblings: &[ParsedValidatorUse],
    known_field_names: &[Ident],
) -> Result<ValidatorGeneratedNames> {
    let candidates = name_candidates(siblings);
    validate_name_candidate(validator_use, known_field_names, &candidates)?;

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
    let candidates = name_candidates(siblings);
    let mut errors = ErrorBag::new();

    for validator_use in siblings {
        errors.push_result(validate_label(validator_use));
        errors.push_result(validate_name_candidate(
            validator_use,
            known_field_names,
            &candidates,
        ));
    }

    errors.finish()
}

fn name_candidates(siblings: &[ParsedValidatorUse]) -> Vec<ValidatorNameCandidate> {
    siblings
        .iter()
        .map(|validator_use| ValidatorNameCandidate {
            field_name: validator_field_name(validator_use),
            variant_name: validator_variant_name(validator_use),
        })
        .collect()
}

fn validate_name_candidate(
    validator_use: &ParsedValidatorUse,
    known_field_names: &[Ident],
    candidates: &[ValidatorNameCandidate],
) -> Result<()> {
    let field_name = validator_field_name(validator_use);
    let variant_name = validator_variant_name(validator_use);

    if validator_use.label.is_none() && reserved_validator_name(&field_name) {
        return Err(Error::new(
            validator_use
                .label_span()
                .unwrap_or(validator_use.source_span),
            format!(
                "`{field_name}` is reserved by generated Koruma error APIs; use a different validator label"
            ),
        ));
    }

    if known_field_names.iter().any(|known| *known == field_name) {
        return Err(Error::new(
            validator_use
                .label_span()
                .unwrap_or(validator_use.source_span),
            format!(
                "validator label `{field_name}` conflicts with a generated field name; use a different label"
            ),
        ));
    }

    let collisions = candidates
        .iter()
        .filter(|candidate| {
            candidate.field_name == field_name || candidate.variant_name == variant_name
        })
        .count();
    if collisions > 1 {
        return Err(name_collision_error(
            validator_use,
            &field_name,
            &variant_name,
        ));
    }

    Ok(())
}

fn validate_label(validator_use: &ParsedValidatorUse) -> Result<()> {
    let Some(label) = validator_use.label.as_ref() else {
        return Ok(());
    };
    let label_text = label.to_string();

    if !is_lower_snake_ident(&label_text) {
        return Err(Error::new(
            label.span(),
            format!("validator label `{label_text}` must be a lower-snake identifier"),
        ));
    }

    if reserved_validator_name(&label_text) {
        return Err(Error::new(
            label.span(),
            format!(
                "`{label_text}` is reserved by generated Koruma error APIs; use a different validator label"
            ),
        ));
    }

    Ok(())
}

fn name_collision_error(
    validator_use: &ParsedValidatorUse,
    field_name: &str,
    variant_name: &str,
) -> Error {
    let span = validator_use
        .label_span()
        .unwrap_or(validator_use.source_span);
    if validator_use.label.is_some() {
        return Error::new(
            span,
            format!(
                "validator label `{field_name}` collides with another validator getter or `{variant_name}` enum variant in this field; use a unique label"
            ),
        );
    }

    Error::new(
        span,
        format!(
            "validator `{}` generates duplicate getter `{field_name}` or `{variant_name}` enum variant in this field; add explicit validator labels such as `label_name = Validator`",
            validator_use.validator.path_name()
        ),
    )
}

fn validator_field_name(validator_use: &ParsedValidatorUse) -> String {
    validator_use
        .label
        .as_ref()
        .map(ToString::to_string)
        .unwrap_or_else(|| validator_use.validator.name().to_string().to_snake_case())
}

fn validator_variant_name(validator_use: &ParsedValidatorUse) -> String {
    validator_use
        .label
        .as_ref()
        .map(|label| label.to_string().to_upper_camel_case())
        .unwrap_or_else(|| {
            validator_use
                .validator
                .name()
                .to_string()
                .to_upper_camel_case()
        })
}

fn is_lower_snake_ident(label: &str) -> bool {
    let mut previous_underscore = false;
    for (index, ch) in label.chars().enumerate() {
        let valid = if index == 0 {
            ch.is_ascii_lowercase()
        } else {
            ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_'
        };
        if !valid {
            return false;
        }
        if ch == '_' {
            if previous_underscore {
                return false;
            }
            previous_underscore = true;
        } else {
            previous_underscore = false;
        }
    }

    !label.ends_with('_')
}

fn reserved_validator_name(name: &str) -> bool {
    matches!(
        name,
        "inner" | "all" | "element_errors" | "is_empty" | "has_errors"
    )
}
