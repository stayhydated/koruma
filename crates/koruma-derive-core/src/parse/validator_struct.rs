use syn::{
    Attribute, Error, Expr, Fields, Ident, ItemStruct, Result, Token, Type, parenthesized,
    parse::{Parse, ParseStream},
    spanned::Spanned as _,
    token,
};

use super::SpannedValue;
use super::diagnostics::{KorumaAttrContext, context_error, unsupported_setter_option_error};
use super::keywords::KorumaKeyword;

/// Parsed validator-field `value` marker metadata.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatorValueSpec {
    capture: CapturePolicy,
}

impl ValidatorValueSpec {
    pub fn capture(&self) -> CapturePolicy {
        self.capture
    }
}

/// Default behavior requested by `#[koruma(setter(default...))]`.
#[derive(Clone, Debug)]
pub enum SetterDefault {
    Default,
    Expr(Box<Expr>),
}

/// Input conversion policy requested by `#[koruma(setter(...))]`.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum SetterInputPolicy {
    #[default]
    Exact,
    Into,
}

impl SetterInputPolicy {
    pub fn accepts_into(self) -> bool {
        matches!(self, Self::Into)
    }
}

/// Presence policy requested by `#[koruma(setter(...))]`.
#[derive(Clone, Debug)]
pub enum SetterPresence {
    Required,
    Optional,
    Defaulted(SetterDefault),
}

/// Parsed validator-field `setter(...)` metadata.
#[derive(Clone, Debug)]
pub struct ValidatorSetterSpec {
    method: Ident,
    input: SetterInputPolicy,
    presence: SetterPresence,
}

impl ValidatorSetterSpec {
    pub fn method(&self) -> &Ident {
        &self.method
    }

    pub fn input(&self) -> SetterInputPolicy {
        self.input
    }

    pub fn presence(&self) -> &SetterPresence {
        &self.presence
    }
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
    name: Ident,
    ty: Type,
    role: ValidatorFieldRole,
}

impl ValidatorFieldSpec {
    pub fn name(&self) -> &Ident {
        &self.name
    }

    pub fn ty(&self) -> &Type {
        &self.ty
    }

    pub fn role(&self) -> &ValidatorFieldRole {
        &self.role
    }
}

/// Fully parsed and normalized `#[koruma::validator]` struct-field metadata.
#[derive(Clone, Debug)]
pub struct ValidatorStructSpec {
    fields: Vec<ValidatorFieldSpec>,
    value_index: usize,
}

impl ValidatorStructSpec {
    pub fn fields(&self) -> &[ValidatorFieldSpec] {
        &self.fields
    }

    pub fn value_index(&self) -> usize {
        self.value_index
    }

    pub fn value_field(&self) -> &ValidatorFieldSpec {
        &self.fields[self.value_index]
    }

    pub fn value_spec(&self) -> &ValidatorValueSpec {
        let ValidatorFieldRole::Value(value) = self.value_field().role() else {
            unreachable!("value_index should point at a value field")
        };
        value
    }
}

#[derive(Clone, Debug)]
pub enum ValidatorFieldKorumaItem {
    Value(ValidatorValueSpec),
    Setter(Box<ParsedSetterOptions>),
}

#[derive(Clone, Debug, Default)]
pub struct ParsedSetterOptions {
    method: Option<Ident>,
    method_marker: Option<SpannedValue<Ident>>,
    input: SetterInputPolicy,
    into_marker: Option<SpannedValue<Ident>>,
    required: bool,
    required_marker: Option<SpannedValue<Ident>>,
    default: Option<SetterDefault>,
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

impl Parse for ValidatorFieldKorumaAttr {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut attr = ValidatorFieldKorumaAttr::default();

        while !input.is_empty() {
            let ident: Ident = input.parse()?;
            let item = match KorumaKeyword::from_ident(&ident) {
                Some(KorumaKeyword::Value) => {
                    if input.peek(token::Paren) {
                        return Err(Error::new(
                            ident.span(),
                            "parenthesized `value` markers are unsupported; use `skip_capture` to skip value capture",
                        ));
                    }
                    ValidatorFieldKorumaItem::Value(ValidatorValueSpec {
                        capture: CapturePolicy::CloneInput,
                    })
                },
                Some(KorumaKeyword::SkipCapture) => {
                    if input.peek(token::Paren) || input.peek(Token![::]) {
                        return Err(Error::new(
                            ident.span(),
                            format!(
                                "`{ident}` is only valid as a bare validator-field `#[koruma(...)]` marker; expected {}",
                                KorumaAttrContext::ValidatorField.accepted_items()
                            ),
                        ));
                    }
                    ValidatorFieldKorumaItem::Value(ValidatorValueSpec {
                        capture: CapturePolicy::Skip,
                    })
                },
                Some(KorumaKeyword::Setter) => {
                    ValidatorFieldKorumaItem::Setter(Box::new(parse_setter_options(input)?))
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
                options.input = SetterInputPolicy::Into;
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
                options.default = Some(if content.peek(Token![=]) {
                    content.parse::<Token![=]>()?;
                    SetterDefault::Expr(Box::new(content.parse()?))
                } else {
                    SetterDefault::Default
                });
                options.default_marker = Some(SpannedValue::new(ident.clone(), ident.span()));
            },
            _ => return Err(unsupported_setter_option_error(&ident)),
        }

        if content.peek(Token![,]) {
            content.parse::<Token![,]>()?;
        } else {
            break;
        }
    }

    Ok(options)
}

fn validator_field_attr(attr: &Attribute) -> Result<ValidatorFieldKorumaAttr> {
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
            input: SetterInputPolicy::Exact,
            presence: SetterPresence::Optional,
        };
        let mut has_setter_metadata = false;
        let mut setter_method_marker: Option<SpannedValue<Ident>> = None;
        let mut setter_into_marker: Option<SpannedValue<Ident>> = None;
        let mut setter_required_marker: Option<SpannedValue<Ident>> = None;
        let mut setter_default_marker: Option<SpannedValue<Ident>> = None;

        let koruma_attrs = field
            .attrs
            .iter()
            .filter(|attr| attr.path().is_ident("koruma"))
            .collect::<Vec<_>>();
        match koruma_attrs.as_slice() {
            [] => {},
            [_, duplicate, ..] => {
                return Err(Error::new(
                    duplicate.path().span(),
                    "only one validator-field `#[koruma(...)]` attribute is allowed; combine markers in a single attribute",
                ));
            },
            [attr] => {
                let markers = validator_field_attr(attr)?;
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
                                        "field `{field_name}` has multiple value markers; `value` and `skip_capture` are the same validator-field role"
                                    ),
                                ));
                            }
                            value = Some((marker.ident, value_spec));
                        },
                        ValidatorFieldKorumaItem::Setter(setter_options) => {
                            has_setter_metadata = true;
                            merge_setter_options(
                                &mut setter,
                                *setter_options,
                                &mut setter_method_marker,
                                &mut setter_into_marker,
                                &mut setter_required_marker,
                                &mut setter_default_marker,
                            )?;
                        },
                    }
                }
            },
        };

        if value.is_some() && has_setter_metadata {
            return Err(Error::new_spanned(
                field,
                "`#[koruma(value)]` fields cannot also use `#[koruma(setter(...))]`",
            ));
        }

        if setter_required_marker.is_some() && setter_default_marker.is_some() {
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
    if options.input == SetterInputPolicy::Into {
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
        setter.input = SetterInputPolicy::Into;
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
        setter.presence = SetterPresence::Required;
    }
    if let Some(default) = options.default {
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
        setter.presence = SetterPresence::Defaulted(default);
    }

    Ok(())
}
