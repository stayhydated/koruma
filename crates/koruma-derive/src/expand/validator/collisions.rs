use super::*;

pub(super) fn reject_state_ident_collision(
    generated_names: &mut GeneratedApiNamespace,
    state_ident: &Ident,
) -> Result<(), syn::Error> {
    generated_names.register_ident(
        state_ident,
        GeneratedApiNameKind::RequiredStateGeneric,
        |existing| state_ident_collision_message(state_ident, existing),
    )
}

pub(super) fn reject_builder_helper_name_collisions(
    builder_name: &Ident,
    module_name: &Ident,
) -> Result<(), syn::Error> {
    let mut generated_names = GeneratedApiNamespace::new();
    generated_names.register_ident(
        builder_name,
        GeneratedApiNameKind::BuilderType,
        |existing| builder_helper_collision_message(builder_name, existing),
    )?;
    generated_names.register_ident(
        module_name,
        GeneratedApiNameKind::BuilderModule,
        |existing| builder_helper_collision_message(module_name, existing),
    )
}

pub(super) fn reject_generated_method_collisions(slots: &[BuilderSlot]) -> Result<(), syn::Error> {
    let mut generated_names = builder_method_namespace();

    for slot in slots {
        let Some(method) = slot.setter_method() else {
            continue;
        };
        let method_name = method.to_string();
        if reserved_builder_method_name(&method_name) {
            return Err(syn::Error::new(
                method.span(),
                format!("setter method name `{method_name}` is reserved by koruma"),
            ));
        }
        generated_names.register_ident(
            method,
            GeneratedApiNameKind::BuilderMethod,
            |existing| builder_method_collision_message(&method_name, existing),
        )?;
    }

    for slot in slots {
        let BuilderSlot::OptionalSetter(slot) = slot else {
            continue;
        };
        if slot.maybe_inner_ty.is_none() {
            continue;
        }
        let maybe_method = format_ident!("maybe_{}", slot.method);
        let maybe_name = maybe_method.to_string();
        generated_names.register_ident(
            &maybe_method,
            GeneratedApiNameKind::OptionalBuilderMethod,
            |existing| optional_builder_method_collision_message(&maybe_name, existing),
        )?;
    }

    Ok(())
}

pub(super) fn state_ident_collision_message(
    state_ident: &Ident,
    existing: &RegisteredApiName,
) -> String {
    match existing.kind {
        GeneratedApiNameKind::UserGeneric => {
            format!("generated required-state generic `{state_ident}` collides with a user generic")
        },
        _ => format!(
            "generated required-state generic `{state_ident}` collides with generated name `{}`",
            existing.ident
        ),
    }
}

pub(super) fn builder_method_collision_message(
    method_name: &str,
    existing: &RegisteredApiName,
) -> String {
    if reserved_builder_method_name(method_name)
        || existing.kind == GeneratedApiNameKind::ReservedBuilderMethod
    {
        return format!("setter method name `{method_name}` is reserved by koruma");
    }

    format!(
        "setter method `{method_name}` collides with another setter `{}`",
        existing.ident
    )
}

pub(super) fn optional_builder_method_collision_message(
    maybe_name: &str,
    existing: &RegisteredApiName,
) -> String {
    match existing.kind {
        GeneratedApiNameKind::BuilderMethod => format!(
            "generated optional setter method `{maybe_name}` collides with setter `{}`",
            existing.ident
        ),
        _ => format!(
            "generated optional setter method `{maybe_name}` collides with generated method `{}`",
            existing.ident
        ),
    }
}

pub(super) fn builder_helper_collision_message(
    requested: &Ident,
    existing: &RegisteredApiName,
) -> String {
    format!(
        "generated builder helper `{requested}` collides with generated {} `{}`",
        builder_api_kind_label(existing.kind),
        existing.ident
    )
}

pub(super) fn builder_api_kind_label(kind: GeneratedApiNameKind) -> &'static str {
    match kind {
        GeneratedApiNameKind::BuilderType => "builder type",
        GeneratedApiNameKind::BuilderModule => "builder module",
        GeneratedApiNameKind::BuilderMethod => "builder method",
        GeneratedApiNameKind::OptionalBuilderMethod => "optional builder method",
        GeneratedApiNameKind::ReservedBuilderMethod => "reserved builder method",
        GeneratedApiNameKind::UserGeneric => "user generic",
        GeneratedApiNameKind::RequiredStateGeneric => "required state generic",
        GeneratedApiNameKind::ExistingField => "input field",
        GeneratedApiNameKind::MainErrorStruct => "main error struct",
        GeneratedApiNameKind::FieldErrorStruct => "field error struct",
        GeneratedApiNameKind::FieldValidatorRefEnum => "field validator enum",
        GeneratedApiNameKind::ElementErrorStruct => "element error struct",
        GeneratedApiNameKind::ElementValidatorRefEnum => "element validator enum",
        GeneratedApiNameKind::ValidatorGetter => "validator getter",
        GeneratedApiNameKind::ValidatorVariant => "validator enum variant",
    }
}
