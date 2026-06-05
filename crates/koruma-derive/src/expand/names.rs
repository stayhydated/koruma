use std::collections::HashMap;

use heck::{ToSnakeCase as _, ToUpperCamelCase as _};
use koruma_derive_core::{FieldInfo, ParsedValidatorUse, ValidatorAttr, ValidatorTypeArg};
use quote::{ToTokens, format_ident};
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
pub(crate) struct ValidatorNamePlan {
    pub field_ident: Ident,
    pub variant_ident: Ident,
    pub doc_name: String,
    label_span: Option<proc_macro2::Span>,
    source_span: proc_macro2::Span,
    collision_context: ValidatorNameCollisionContext,
}

impl ValidatorNamePlan {
    fn for_validator(validator_use: &ParsedValidatorUse) -> Self {
        let field_name = validator_field_name(validator_use);
        let variant_name = validator_variant_name(validator_use);
        let label_span = validator_use.label_span();
        let source_span = validator_use.source_span();
        let field_ident = generated_ident(validator_use, &field_name);
        let variant_ident = generated_ident(validator_use, &variant_name);
        let doc_name = validator_use
            .label()
            .map(ToString::to_string)
            .unwrap_or_else(|| validator_use.validator().path_name());
        let collision_context = ValidatorNameCollisionContext {
            field_name,
            variant_name,
            validator_path_name: validator_use.validator().path_name(),
            suggested_label: suggested_validator_label(validator_use),
            validator_syntax: validator_syntax(validator_use.validator()),
            has_label: validator_use.label().is_some(),
        };

        Self {
            field_ident,
            variant_ident,
            doc_name,
            label_span,
            source_span,
            collision_context,
        }
    }
}

#[derive(Clone, Debug)]
struct ValidatorNameCollisionContext {
    field_name: String,
    variant_name: String,
    validator_path_name: String,
    suggested_label: String,
    validator_syntax: String,
    has_label: bool,
}

pub(crate) fn plan_validator_names(
    siblings: &[ParsedValidatorUse],
    known_field_names: &[Ident],
) -> Result<Vec<ValidatorNamePlan>> {
    let mut namespace = seed_existing_fields(known_field_names);
    let mut errors = ErrorBag::new();
    let mut planned = Vec::new();

    for validator_use in siblings {
        let plan = ValidatorNamePlan::for_validator(validator_use);
        errors.push_result(validate_reserved_label(validator_use));
        errors.push_result(register_validator_names(&mut namespace, &plan));
        planned.push(plan);
    }

    errors.finish()?;
    Ok(planned)
}

fn register_validator_names(
    namespace: &mut super::generated_api::GeneratedApiNamespace,
    plan: &ValidatorNamePlan,
) -> Result<()> {
    let context = &plan.collision_context;

    if !context.has_label && reserved_error_api_name(&context.field_name) {
        return Err(Error::new(
            plan.label_span.unwrap_or(plan.source_span),
            format!(
                "`{}` is reserved by generated Koruma error APIs; use a different validator label",
                context.field_name
            ),
        ));
    }

    namespace.register_ident(
        &plan.field_ident,
        GeneratedApiNameKind::ValidatorGetter,
        |existing| validator_collision_message(context, existing),
    )?;

    namespace.register_ident(
        &plan.variant_ident,
        GeneratedApiNameKind::ValidatorVariant,
        |existing| validator_collision_message(context, existing),
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
    context: &ValidatorNameCollisionContext,
    existing: &RegisteredApiName,
) -> String {
    if existing.kind == GeneratedApiNameKind::ExistingField {
        return format!(
            "validator label `{}` conflicts with a generated field name; use a different label",
            context.field_name
        );
    }

    if context.has_label {
        return format!(
            "validator label `{}` collides with another validator getter or `{}` enum variant in this field; use a unique label",
            context.field_name, context.variant_name
        );
    }

    format!(
        "validator `{}` generates duplicate getter `{}` or `{}` enum variant in this field; add explicit validator labels such as `{} = {}`",
        context.validator_path_name,
        context.field_name,
        context.variant_name,
        context.suggested_label,
        context.validator_syntax
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

fn suggested_validator_label(validator_use: &ParsedValidatorUse) -> String {
    let field_name = validator_field_name(validator_use);
    validator_use
        .validator()
        .setter_calls()
        .first()
        .map(|call| format!("{}_{}", call.method(), field_name))
        .unwrap_or_else(|| format!("{field_name}_validator"))
}

fn validator_syntax(validator: &ValidatorAttr) -> String {
    let mut syntax = validator.path_name();
    match validator.type_arg() {
        ValidatorTypeArg::None => {},
        ValidatorTypeArg::Infer => syntax.push_str("::<_>"),
        ValidatorTypeArg::Explicit(ty) => {
            syntax.push_str("::<");
            syntax.push_str(&ty.to_token_stream().to_string());
            syntax.push('>');
        },
    }

    for (index, call) in validator.setter_calls().iter().enumerate() {
        let args = call
            .args()
            .iter()
            .map(ToTokens::to_token_stream)
            .map(|tokens| tokens.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        if index == 0 {
            syntax.push_str("::");
        } else {
            syntax.push('.');
        }
        syntax.push_str(&call.method().to_string());
        syntax.push('(');
        syntax.push_str(&args);
        syntax.push(')');
    }

    syntax
}

#[cfg(test)]
mod tests {
    use super::super::generated_api::{GeneratedApiNameKind, RegisteredApiName};
    use super::*;

    #[test]
    fn generated_name_kind_labels_cover_every_registered_kind() {
        for (kind, expected) in [
            (GeneratedApiNameKind::MainErrorStruct, "main error struct"),
            (GeneratedApiNameKind::FieldErrorStruct, "field error struct"),
            (
                GeneratedApiNameKind::FieldValidatorRefEnum,
                "field validator enum",
            ),
            (
                GeneratedApiNameKind::ElementErrorStruct,
                "element error struct",
            ),
            (
                GeneratedApiNameKind::ElementValidatorRefEnum,
                "element validator enum",
            ),
            (GeneratedApiNameKind::ValidatorGetter, "validator getter"),
            (
                GeneratedApiNameKind::ValidatorVariant,
                "validator enum variant",
            ),
            (GeneratedApiNameKind::ExistingField, "input field"),
            (GeneratedApiNameKind::BuilderType, "builder type"),
            (GeneratedApiNameKind::BuilderModule, "builder module"),
            (GeneratedApiNameKind::BuilderMethod, "builder method"),
            (
                GeneratedApiNameKind::OptionalBuilderMethod,
                "optional builder method",
            ),
            (
                GeneratedApiNameKind::ReservedBuilderMethod,
                "reserved builder method",
            ),
            (GeneratedApiNameKind::UserGeneric, "user generic"),
            (
                GeneratedApiNameKind::RequiredStateGeneric,
                "required state generic",
            ),
        ] {
            assert_eq!(generated_name_kind_label(kind), expected);
        }
    }

    #[test]
    fn validator_syntax_renders_type_arguments_and_direct_setters() {
        let plain: ValidatorAttr = syn::parse_quote!(RequiredValidation);
        assert_eq!(validator_syntax(&plain), "RequiredValidation");

        let inferred: ValidatorAttr = syn::parse_quote!(RangeValidation::<_>::min(0).max(10));
        assert_eq!(
            validator_syntax(&inferred),
            "RangeValidation::<_>::min(0).max(10)"
        );

        let explicit: ValidatorAttr =
            syn::parse_quote!(validators::RangeValidation::<Option<i32>>::min(0));
        assert_eq!(
            validator_syntax(&explicit),
            "validators::RangeValidation::<Option < i32 >>::min(0)"
        );
    }

    #[test]
    fn validator_collision_messages_cover_labeled_unlabeled_and_field_collisions() {
        let existing_field = RegisteredApiName {
            kind: GeneratedApiNameKind::ExistingField,
            ident: quote::format_ident!("value"),
        };
        let labeled = ValidatorNameCollisionContext {
            field_name: "value".to_owned(),
            variant_name: "Value".to_owned(),
            validator_path_name: "demo::ValueValidation".to_owned(),
            suggested_label: "min_value_validation".to_owned(),
            validator_syntax: "demo::ValueValidation".to_owned(),
            has_label: true,
        };
        assert!(
            validator_collision_message(&labeled, &existing_field)
                .contains("conflicts with a generated field name")
        );

        let existing_getter = RegisteredApiName {
            kind: GeneratedApiNameKind::ValidatorGetter,
            ident: quote::format_ident!("value"),
        };
        assert!(
            validator_collision_message(&labeled, &existing_getter)
                .contains("collides with another validator getter")
        );

        let unlabeled = ValidatorNameCollisionContext {
            has_label: false,
            ..labeled
        };
        assert!(
            validator_collision_message(&unlabeled, &existing_getter)
                .contains("add explicit validator labels")
        );
    }

    #[test]
    fn unlabeled_reserved_generated_error_names_are_rejected() {
        let reserved_validator = ParsedValidatorUse::unlabeled(syn::parse_quote!(All));
        let err = plan_validator_names(&[reserved_validator], &[])
            .expect_err("unlabeled validator generating `all` should be rejected");
        assert!(
            err.to_string()
                .contains("reserved by generated Koruma error APIs")
        );
    }
}
