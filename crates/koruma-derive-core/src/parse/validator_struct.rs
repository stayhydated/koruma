use syn::{
    Error, Expr, Fields, Ident, ItemStruct, Result, Token, Type, parenthesized,
    parse::{Parse, ParseStream},
};
use syn_cfg_attr::{AttributeHelpers, ExpandedAttr};

use super::keywords::KorumaKeyword;
use super::{KorumaAttrContext, SpannedValue, context_error};

/// Parsed validator-field `value` marker metadata.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatorValueSpec {
    pub capture: CapturePolicy,
}

/// Default behavior requested by `#[koruma(setter(default...))]`.
#[derive(Clone, Debug)]
pub enum SetterDefault {
    None,
    Default,
    Expr(Expr),
}

/// Parsed validator-field `setter(...)` metadata.
#[derive(Clone, Debug)]
pub struct ValidatorSetterSpec {
    pub method: Ident,
    pub into: bool,
    pub required: bool,
    pub default: SetterDefault,
}

/// Typed role of a field inside a `#[koruma::validator]` struct.
#[derive(Clone, Debug)]
pub enum ValidatorFieldRole {
    Value(ValidatorValueSpec),
    Setter(ValidatorSetterSpec),
}

/// Typed metadata for one field inside a `#[koruma::validator]` struct.
#[derive(Clone, Debug)]
pub struct ValidatorFieldSpec {
    pub name: Ident,
    pub ty: Type,
    pub role: ValidatorFieldRole,
}

/// Fully parsed and normalized `#[koruma::validator]` struct-field metadata.
#[derive(Clone, Debug)]
pub struct ValidatorStructSpec {
    pub fields: Vec<ValidatorFieldSpec>,
    pub value_index: usize,
}

impl ValidatorStructSpec {
    pub fn value_field(&self) -> &ValidatorFieldSpec {
        &self.fields[self.value_index]
    }

    pub fn value_spec(&self) -> &ValidatorValueSpec {
        let ValidatorFieldRole::Value(value) = &self.value_field().role else {
            unreachable!("value_index should point at a value field")
        };
        value
    }
}

#[derive(Clone, Debug)]
pub enum ValidatorFieldKorumaItem {
    Value(ValidatorValueSpec),
    Setter(ParsedSetterOptions),
}

#[derive(Clone, Debug, Default)]
pub struct ParsedSetterOptions {
    method: Option<Ident>,
    method_marker: Option<SpannedValue<Ident>>,
    into: bool,
    into_marker: Option<SpannedValue<Ident>>,
    required: bool,
    required_marker: Option<SpannedValue<Ident>>,
    default: SetterDefault,
    default_marker: Option<SpannedValue<Ident>>,
}

#[derive(Clone, Debug)]
struct ValidatorFieldMarker {
    ident: Ident,
    item: ValidatorFieldKorumaItem,
}

#[derive(Clone, Debug, Default)]
struct ValidatorFieldKorumaAttr {
    markers: Vec<ValidatorFieldMarker>,
}

impl Default for SetterDefault {
    fn default() -> Self {
        Self::None
    }
}

impl Parse for ValidatorFieldKorumaAttr {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut attr = ValidatorFieldKorumaAttr::default();

        while !input.is_empty() {
            let ident: Ident = input.parse()?;
            let item = match KorumaKeyword::from_ident(&ident) {
                Some(KorumaKeyword::Value) => {
                    let capture = if input.peek(syn::token::Paren) {
                        parse_value_capture_policy(input)?
                    } else {
                        CapturePolicy::CloneInput
                    };
                    ValidatorFieldKorumaItem::Value(ValidatorValueSpec { capture })
                },
                Some(KorumaKeyword::Setter) => {
                    ValidatorFieldKorumaItem::Setter(parse_setter_options(input)?)
                },
                _ => return Err(context_error(&ident, KorumaAttrContext::ValidatorField)),
            };
            attr.markers.push(ValidatorFieldMarker { ident, item });

            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            } else {
                break;
            }
        }

        Ok(attr)
    }
}

fn parse_setter_options(input: ParseStream) -> Result<ParsedSetterOptions> {
    let content;
    parenthesized!(content in input);
    let mut options = ParsedSetterOptions::default();

    while !content.is_empty() {
        let ident: Ident = content.parse()?;
        match KorumaKeyword::from_ident(&ident) {
            Some(KorumaKeyword::Into) => {
                if options.into_marker.is_some() {
                    return Err(Error::new(ident.span(), "duplicate `setter(into)` option"));
                }
                options.into = true;
                options.into_marker = Some(SpannedValue::new(ident.clone(), ident.span()));
            },
            Some(KorumaKeyword::Required) => {
                if options.required_marker.is_some() {
                    return Err(Error::new(
                        ident.span(),
                        "duplicate `setter(required)` option",
                    ));
                }
                options.required = true;
                options.required_marker = Some(SpannedValue::new(ident.clone(), ident.span()));
            },
            Some(KorumaKeyword::Name) => {
                if options.method_marker.is_some() {
                    return Err(Error::new(
                        ident.span(),
                        "duplicate `setter(name = ...)` option",
                    ));
                }
                content.parse::<Token![=]>()?;
                options.method = Some(content.parse()?);
                options.method_marker = Some(SpannedValue::new(ident.clone(), ident.span()));
            },
            Some(KorumaKeyword::Default) => {
                if options.default_marker.is_some() {
                    return Err(Error::new(
                        ident.span(),
                        "duplicate `setter(default)` option",
                    ));
                }
                options.default = if content.peek(Token![=]) {
                    content.parse::<Token![=]>()?;
                    SetterDefault::Expr(content.parse()?)
                } else {
                    SetterDefault::Default
                };
                options.default_marker = Some(SpannedValue::new(ident.clone(), ident.span()));
            },
            _ => {
                return Err(Error::new(
                    ident.span(),
                    format!(
                        "unsupported `#[koruma(setter({ident}))]` option; supported options are `into`, `required`, `name`, and `default`"
                    ),
                ));
            },
        }

        if content.peek(Token![,]) {
            content.parse::<Token![,]>()?;
        } else {
            break;
        }
    }

    Ok(options)
}

fn parse_value_capture_policy(input: ParseStream) -> Result<CapturePolicy> {
    let content;
    parenthesized!(content in input);

    let key: Ident = content.parse()?;
    if !matches!(
        KorumaKeyword::from_ident(&key),
        Some(KorumaKeyword::Capture)
    ) {
        return Err(Error::new(
            key.span(),
            "unsupported `value(...)` option; supported option is `capture = skip`",
        ));
    }
    content.parse::<Token![=]>()?;
    let policy: Ident = content.parse()?;
    let capture = if matches!(
        KorumaKeyword::from_ident(&policy),
        Some(KorumaKeyword::Skip)
    ) {
        CapturePolicy::Skip
    } else {
        return Err(Error::new(
            policy.span(),
            "unsupported capture policy; supported policy is `skip`",
        ));
    };

    if content.peek(Token![,]) {
        content.parse::<Token![,]>()?;
    }
    if !content.is_empty() {
        return Err(Error::new(
            content.span(),
            "unexpected extra tokens in `value(...)`; supported option is `capture = skip`",
        ));
    }

    Ok(capture)
}

fn validator_field_attr(attr: &ExpandedAttr) -> Result<ValidatorFieldKorumaAttr> {
    attr.parse_args::<ValidatorFieldKorumaAttr>()
}

/// Describes whether the `#[koruma(value)]` field should capture the input
/// value in derived validation errors.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CapturePolicy {
    /// Store the validated value in the generated validator instance.
    CloneInput,
    /// Skip storing the validated value and leave the field at its default.
    Skip,
}

/// Parse all fields in a `#[koruma::validator]` struct into typed metadata.
///
/// This strict API proves that exactly one field is marked `#[koruma(value)]`,
/// value fields do not also define setter metadata, all validator-field
/// `#[koruma(...)]` attributes use validator-field grammar, and required
/// setters are not also defaulted.
pub fn parse_validator_struct(input: &ItemStruct) -> Result<ValidatorStructSpec> {
    parse_validator_fields(input)?.ok_or_else(|| {
        Error::new_spanned(
            input,
            "koruma::validator requires a field marked with #[koruma(value)].\n\
             Example:\n\
             #[koruma(value)]\n\
             actual: Option<i32>",
        )
    })
}

fn parse_validator_fields(input: &ItemStruct) -> Result<Option<ValidatorStructSpec>> {
    let Fields::Named(ref fields) = input.fields else {
        return Ok(None);
    };

    let mut parsed_fields: Vec<ValidatorFieldSpec> = Vec::new();
    let mut value_index: Option<usize> = None;

    for field in &fields.named {
        let Some(field_name) = field.ident.clone() else {
            continue;
        };

        let mut value: Option<(Ident, ValidatorValueSpec)> = None;
        let mut setter = ValidatorSetterSpec {
            method: field_name.clone(),
            into: false,
            required: false,
            default: SetterDefault::None,
        };
        let mut has_setter_metadata = false;
        let mut setter_method_marker: Option<SpannedValue<Ident>> = None;
        let mut setter_into_marker: Option<SpannedValue<Ident>> = None;
        let mut setter_required_marker: Option<SpannedValue<Ident>> = None;
        let mut setter_default_marker: Option<SpannedValue<Ident>> = None;

        for attr in field.attrs.to_vec().find_attribute("koruma") {
            let markers = validator_field_attr(&attr)?;
            if markers.markers.is_empty() {
                return Err(Error::new_spanned(
                    attr.path(),
                    format!(
                        "`#[koruma(...)]` on validator fields must contain {}; data-field modifiers like `nested`, `newtype`, and validators are only valid on derive data fields",
                        KorumaAttrContext::ValidatorField.accepted_items()
                    ),
                ));
            }

            for marker in markers.markers {
                match marker.item {
                    ValidatorFieldKorumaItem::Value(value_spec) => {
                        if value.is_some() {
                            return Err(Error::new(
                                marker.ident.span(),
                                format!(
                                    "field `{field_name}` has multiple `#[koruma(value)]` markers"
                                ),
                            ));
                        }
                        value = Some((marker.ident, value_spec));
                    },
                    ValidatorFieldKorumaItem::Setter(setter_options) => {
                        has_setter_metadata = true;
                        merge_setter_options(
                            &mut setter,
                            setter_options,
                            &mut setter_method_marker,
                            &mut setter_into_marker,
                            &mut setter_required_marker,
                            &mut setter_default_marker,
                        )?;
                    },
                }
            }
        }

        if value.is_some() && has_setter_metadata {
            return Err(Error::new_spanned(
                field,
                "`#[koruma(value)]` fields cannot also use `#[koruma(setter(...))]`",
            ));
        }

        if setter.required && !matches!(setter.default, SetterDefault::None) {
            return Err(Error::new_spanned(
                field,
                "`required` and `default` cannot be combined in `#[koruma(setter(...))]`",
            ));
        }

        let role = if let Some((_, value_spec)) = value {
            if let Some(existing_index) = value_index {
                let existing = &parsed_fields[existing_index].name;
                return Err(Error::new(
                    field_name.span(),
                    format!(
                        "koruma::validator requires exactly one `#[koruma(value)]` field, found both `{}` and `{}`",
                        existing, field_name
                    ),
                ));
            }
            value_index = Some(parsed_fields.len());
            ValidatorFieldRole::Value(value_spec)
        } else {
            ValidatorFieldRole::Setter(setter)
        };

        parsed_fields.push(ValidatorFieldSpec {
            name: field_name,
            ty: field.ty.clone(),
            role,
        });
    }

    match value_index {
        Some(value_index) => Ok(Some(ValidatorStructSpec {
            fields: parsed_fields,
            value_index,
        })),
        None => Ok(None),
    }
}

fn merge_setter_options(
    setter: &mut ValidatorSetterSpec,
    options: ParsedSetterOptions,
    method_marker: &mut Option<SpannedValue<Ident>>,
    into_marker: &mut Option<SpannedValue<Ident>>,
    required_marker: &mut Option<SpannedValue<Ident>>,
    default_marker: &mut Option<SpannedValue<Ident>>,
) -> Result<()> {
    if let Some(method) = options.method {
        if method_marker.is_some() {
            return Err(Error::new(
                options
                    .method_marker
                    .as_ref()
                    .map(|marker| marker.span)
                    .unwrap_or_else(|| method.span()),
                "duplicate `setter(name = ...)` option",
            ));
        }
        *method_marker = options.method_marker;
        setter.method = method;
    }
    if options.into {
        if into_marker.is_some() {
            return Err(Error::new(
                options
                    .into_marker
                    .as_ref()
                    .map(|marker| marker.span)
                    .unwrap_or_else(|| setter.method.span()),
                "duplicate `setter(into)` option",
            ));
        }
        *into_marker = options.into_marker;
        setter.into = true;
    }
    if options.required {
        if required_marker.is_some() {
            return Err(Error::new(
                options
                    .required_marker
                    .as_ref()
                    .map(|marker| marker.span)
                    .unwrap_or_else(|| setter.method.span()),
                "duplicate `setter(required)` option",
            ));
        }
        *required_marker = options.required_marker;
        setter.required = true;
    }
    if !matches!(options.default, SetterDefault::None) {
        if default_marker.is_some() {
            return Err(Error::new(
                options
                    .default_marker
                    .as_ref()
                    .map(|marker| marker.span)
                    .unwrap_or_else(|| setter.method.span()),
                "duplicate `setter(default)` option",
            ));
        }
        *default_marker = options.default_marker;
        setter.default = options.default;
    }

    Ok(())
}
