use std::collections::HashMap;

use heck::ToUpperCamelCase as _;
use quote::format_ident;
use syn::{Error, Ident, Result};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GeneratedApiNameKind {
    ExistingField,
    MainErrorStruct,
    FieldErrorStruct,
    FieldValidatorRefEnum,
    ElementErrorStruct,
    ElementValidatorRefEnum,
    ValidatorGetter,
    ValidatorVariant,
    BuilderType,
    BuilderModule,
    BuilderMethod,
    OptionalBuilderMethod,
    ReservedBuilderMethod,
    UserGeneric,
    RequiredStateGeneric,
}

#[derive(Clone, Debug)]
pub(crate) struct RegisteredApiName {
    pub kind: GeneratedApiNameKind,
    pub ident: Ident,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct GeneratedApiNamespace {
    names: HashMap<String, RegisteredApiName>,
}

impl GeneratedApiNamespace {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn register_ident<F>(
        &mut self,
        ident: &Ident,
        kind: GeneratedApiNameKind,
        collision_message: F,
    ) -> Result<()>
    where
        F: FnOnce(&RegisteredApiName) -> String,
    {
        let name = ident.to_string();
        if let Some(existing) = self.names.get(&name) {
            return Err(Error::new(ident.span(), collision_message(existing)));
        }

        self.names.insert(
            name,
            RegisteredApiName {
                kind,
                ident: ident.clone(),
            },
        );
        Ok(())
    }

    pub(crate) fn reserve_ident(&mut self, ident: &Ident, kind: GeneratedApiNameKind) {
        self.names.insert(
            ident.to_string(),
            RegisteredApiName {
                kind,
                ident: ident.clone(),
            },
        );
    }
}

pub(crate) fn reserved_error_api_name(name: &str) -> bool {
    matches!(
        name,
        "inner" | "all" | "element_errors" | "is_empty" | "has_errors"
    )
}

pub(crate) fn reserved_builder_method_name(name: &str) -> bool {
    RESERVED_BUILDER_METHOD_NAMES.contains(&name)
}

const RESERVED_BUILDER_METHOD_NAMES: &[&str] = &["with_value", "build", "__koruma_builder"];

pub(crate) fn builder_method_namespace() -> GeneratedApiNamespace {
    let mut namespace = GeneratedApiNamespace::new();
    for reserved in RESERVED_BUILDER_METHOD_NAMES {
        namespace.reserve_ident(
            &format_ident!("{reserved}"),
            GeneratedApiNameKind::ReservedBuilderMethod,
        );
    }
    namespace
}

pub(crate) fn seed_existing_fields(fields: &[Ident]) -> GeneratedApiNamespace {
    let mut namespace = GeneratedApiNamespace::new();
    for field in fields {
        namespace.reserve_ident(field, GeneratedApiNameKind::ExistingField);
    }
    namespace
}

pub(crate) fn state_ident_for(ident: &Ident) -> Ident {
    format_ident!("__Koruma{}State", ident.to_string().to_upper_camel_case())
}

pub(crate) fn user_generic_namespace(generics: &syn::Generics) -> GeneratedApiNamespace {
    let mut namespace = GeneratedApiNamespace::new();
    for ident in user_generic_names(generics) {
        namespace.reserve_ident(&ident, GeneratedApiNameKind::UserGeneric);
    }
    namespace
}

fn user_generic_names(generics: &syn::Generics) -> Vec<Ident> {
    generics
        .params
        .iter()
        .filter_map(|param| match param {
            syn::GenericParam::Type(param) => Some(param.ident.clone()),
            syn::GenericParam::Const(param) => Some(param.ident.clone()),
            syn::GenericParam::Lifetime(_) => None,
        })
        .collect()
}
