use syn::{Error, Ident};

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

pub(super) fn unsupported_setter_option_error(option: &Ident) -> Error {
    Error::new(
        option.span(),
        format!(
            "unsupported `#[koruma(setter({option}))]` option; supported options are `into`, `required`, `name`, and `default`"
        ),
    )
}

pub(super) fn unsupported_value_option_error(option: &Ident) -> Error {
    Error::new(
        option.span(),
        "unsupported `value(...)` option; supported option is `capture = skip`",
    )
}

pub(super) fn unsupported_capture_policy_error(policy: &Ident) -> Error {
    Error::new(
        policy.span(),
        "unsupported capture policy; supported policy is `skip`",
    )
}
