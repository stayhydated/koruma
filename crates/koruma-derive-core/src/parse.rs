//! Parsing logic for `#[koruma(...)]` attributes.
//!
//! This module provides context-specific parsers for koruma validation
//! attributes from syn AST nodes.

mod data_field;
mod derive_struct;
mod diagnostics;
mod keywords;
#[cfg(feature = "internal-showcase")]
mod showcase;
mod validator_chain;
mod validator_struct;

pub use data_field::{
    DataFieldKorumaAttr, DataFieldKorumaItem, ElementValidationSpec, FieldInfo, FieldModifier,
    FieldModifierKind, FieldSource, FieldValidationSpec, ParsedDataField, ParsedFieldSpec,
    ParsedValidatorUse, ValidatorLabel, ValidatorTargetSelector, parse_field,
};
pub use derive_struct::{
    NewtypeConstructor, RegularConstructor, StructKorumaAttr, StructKorumaItem, StructMode,
    StructNewtypeOptions, StructOptions, parse_struct_options,
};
pub use diagnostics::KorumaAttrContext;
#[cfg(feature = "internal-showcase")]
pub use showcase::{ShowcaseAttr, ShowcaseInputType, ShowcaseModule, find_showcase_attr};
pub use validator_chain::{
    BuilderMethodCall, ValidatorAttr, ValidatorPath, ValidatorSetterArg, ValidatorTypeArg,
};
pub use validator_struct::{
    CapturePolicy, SetterDefault, SetterInputPolicy, SetterPresence, ValidatorFieldKorumaItem,
    ValidatorFieldRole, ValidatorFieldSpec, ValidatorSetterSpec, ValidatorStructSpec,
    ValidatorValueSpec, parse_validator_struct,
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
