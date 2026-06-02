//! Parsing logic for `#[koruma(...)]` attributes.
//!
//! This module provides context-specific parsers for koruma validation
//! attributes from syn AST nodes.

use syn::{Error, Ident};

mod data_field;
mod derive_struct;
mod keywords;
#[cfg(feature = "internal-showcase")]
mod showcase;
mod validator_chain;
mod validator_struct;

pub use data_field::{
    DataFieldKorumaAttr, DataFieldKorumaItem, ElementValidationSpec, FieldInfo, FieldModifier,
    FieldModifierKind, FieldValidationSpec, ParsedFieldSpec, ParsedValidatorUse, ValidatorLabel,
    ValidatorTargetSelector, parse_field,
};
pub use derive_struct::{
    StructConstructor, StructKorumaAttr, StructKorumaItem, StructNewtypeOptions, StructOptions,
    parse_struct_options,
};
#[cfg(feature = "internal-showcase")]
pub use showcase::{ShowcaseAttr, ShowcaseInputType, ShowcaseModule, find_showcase_attr};
pub use validator_chain::{BuilderMethodCall, ValidatorAttr, ValidatorSetterArg, ValidatorTypeArg};
pub use validator_struct::{
    CapturePolicy, SetterDefault, ValidatorFieldKorumaItem, ValidatorFieldRole, ValidatorFieldSpec,
    ValidatorSetterSpec, ValidatorStructSpec, ValidatorValueSpec, parse_validator_struct,
};

/// Parsed value paired with the source span that introduced it.
#[derive(Clone, Debug)]
pub struct SpannedValue<T> {
    pub value: T,
    pub span: proc_macro2::Span,
}

impl<T> SpannedValue<T> {
    pub fn new(value: T, span: proc_macro2::Span) -> Self {
        Self { value, span }
    }
}

/// Attribute language supported by a specific koruma macro context.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KorumaAttrContext {
    /// Struct-level `#[koruma(...)]` options on `#[derive(Koruma)]` data types.
    Struct,
    /// Field-level `#[koruma(...)]` options on `#[derive(Koruma)]` data fields.
    DataField,
    /// Field-level `#[koruma(...)]` markers inside `#[koruma::validator]` structs.
    ValidatorField,
    /// `#[showcase(...)]` validator metadata.
    Showcase,
}

impl KorumaAttrContext {
    pub(super) fn name(self) -> &'static str {
        match self {
            KorumaAttrContext::Struct => "derive struct",
            KorumaAttrContext::DataField => "derive data field",
            KorumaAttrContext::ValidatorField => "validator field",
            KorumaAttrContext::Showcase => "showcase",
        }
    }

    pub(super) fn accepted_items(self) -> &'static str {
        match self {
            KorumaAttrContext::Struct => "`try_new`, `newtype`, or `newtype(try_from)`",
            KorumaAttrContext::DataField => {
                "`skip`, `nested`, `newtype`, validators, or `each(...)`"
            },
            KorumaAttrContext::ValidatorField => {
                "`value`, `value(capture = skip)`, or `setter(...)`"
            },
            KorumaAttrContext::Showcase => {
                "`name`, `description`, `create`, `input_type`, or `module`"
            },
        }
    }
}

pub(super) fn context_error(marker: &Ident, context: KorumaAttrContext) -> Error {
    Error::new(
        marker.span(),
        format!(
            "`{}` is not valid in a {} `#[koruma(...)]` attribute; expected {}",
            marker,
            context.name(),
            context.accepted_items()
        ),
    )
}
