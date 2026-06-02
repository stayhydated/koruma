use std::fmt;

use syn::{
    Error, Field, Ident, Index, Member, Path, Result, Token, Type, parenthesized,
    parse::{Parse, ParseStream},
    spanned::Spanned,
    token,
};
use syn_cfg_attr::AttributeHelpers;

use super::SpannedValue;
use super::diagnostics::{KorumaAttrContext, context_error};
use super::keywords::KorumaKeyword;
use super::validator_chain::ValidatorAttr;

/// Field-level modifier parsed from `#[koruma(...)]`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FieldModifierKind {
    Skip,
    Nested,
    Newtype,
}

/// A parsed field-level modifier inside `#[koruma(...)]`.
#[derive(Clone, Debug)]
pub struct FieldModifier {
    pub kind: FieldModifierKind,
    pub ident: Ident,
    pub source: SpannedValue<FieldModifierKind>,
}

impl FieldModifier {
    fn span(&self) -> proc_macro2::Span {
        self.source.span
    }
}

/// Validated lower-snake label for a field or element validator.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatorLabel {
    ident: Ident,
}

impl ValidatorLabel {
    pub fn new(ident: Ident) -> Result<Self> {
        let label_text = ident.to_string();
        if !is_lower_snake_ident(&label_text) {
            return Err(Error::new(
                ident.span(),
                format!("validator label `{label_text}` must be a lower-snake identifier"),
            ));
        }

        Ok(Self { ident })
    }

    pub fn ident(&self) -> &Ident {
        &self.ident
    }

    pub fn span(&self) -> proc_macro2::Span {
        self.ident.span()
    }
}

impl fmt::Display for ValidatorLabel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.ident, f)
    }
}

/// A parsed validator occurrence inside a data-field `#[koruma(...)]` attribute.
#[derive(Clone, Debug)]
pub struct ParsedValidatorUse {
    validator: ValidatorAttr,
    label: Option<ValidatorLabel>,
    target: ValidatorTargetSelector,
    source_span: proc_macro2::Span,
}

impl ParsedValidatorUse {
    pub fn unlabeled(validator: ValidatorAttr) -> Self {
        let source_span = validator.path().span();
        Self {
            validator,
            label: None,
            target: ValidatorTargetSelector::Default,
            source_span,
        }
    }

    pub fn new(
        label: Option<ValidatorLabel>,
        target: ValidatorTargetSelector,
        validator: ValidatorAttr,
    ) -> Self {
        let source_span = validator.path().span();
        Self {
            validator,
            label,
            target,
            source_span,
        }
    }

    pub fn try_new(
        label: Option<Ident>,
        target: ValidatorTargetSelector,
        validator: ValidatorAttr,
    ) -> Result<Self> {
        let label = label.map(ValidatorLabel::new).transpose()?;
        Ok(Self::new(label, target, validator))
    }

    pub fn labeled(label: Ident, validator: ValidatorAttr) -> Result<Self> {
        Self::try_new(Some(label), ValidatorTargetSelector::Default, validator)
    }

    pub fn label_span(&self) -> Option<proc_macro2::Span> {
        self.label.as_ref().map(ValidatorLabel::span)
    }

    pub fn label(&self) -> Option<&ValidatorLabel> {
        self.label.as_ref()
    }

    pub fn validator(&self) -> &ValidatorAttr {
        &self.validator
    }

    pub fn target(&self) -> &ValidatorTargetSelector {
        &self.target
    }

    pub fn source_span(&self) -> proc_macro2::Span {
        self.source_span
    }
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

/// Explicit validation target selection for optional fields or optional `each(...)` elements.
#[derive(Clone, Debug)]
pub enum ValidatorTargetSelector {
    /// Use Koruma's default target: unwrap `Option<T>` and skip `None`.
    Default,
    /// Validate the full field or element value, including `Option<T>`.
    Full { marker: SpannedValue<Ident> },
    /// Explicit spelling of the default unwrapped target.
    Unwrapped { marker: SpannedValue<Ident> },
}

impl ValidatorTargetSelector {
    pub fn marker_span(&self) -> Option<proc_macro2::Span> {
        match self {
            Self::Default => None,
            Self::Full { marker } | Self::Unwrapped { marker } => Some(marker.span),
        }
    }

    pub fn is_full(&self) -> bool {
        matches!(self, Self::Full { .. })
    }
}

/// A parsed direct field validator inside `#[koruma(...)]`.
#[derive(Clone, Debug)]
pub struct FieldValidationSpec {
    validator: ParsedValidatorUse,
}

impl FieldValidationSpec {
    pub fn validator(&self) -> &ParsedValidatorUse {
        &self.validator
    }
}

/// A parsed `each(...)` element-validation block inside `#[koruma(...)]`.
#[derive(Clone, Debug)]
pub struct ElementValidationSpec {
    marker: Ident,
    marker_source: SpannedValue<Ident>,
    validators: Vec<ParsedValidatorUse>,
}

impl ElementValidationSpec {
    pub fn marker(&self) -> &Ident {
        &self.marker
    }

    pub fn marker_source(&self) -> &SpannedValue<Ident> {
        &self.marker_source
    }

    pub fn validators(&self) -> &[ParsedValidatorUse] {
        &self.validators
    }
}

/// A single typed item inside data-field `#[koruma(...)]`.
#[derive(Clone, Debug)]
pub enum DataFieldKorumaItem {
    Modifier(FieldModifier),
    FieldValidation(Box<FieldValidationSpec>),
    ElementValidation(ElementValidationSpec),
}

/// Represents a parsed data-field `#[koruma(...)]` attribute which can contain multiple validators
/// separated by commas: `#[koruma(Validator1::a(1), Validator2)]`
///
/// Can also include:
/// - `each(...)` modifier for collection validation
/// - `skip` to skip validation for a field
/// - `nested` to validate nested structs that also derive Koruma
/// - `newtype` to validate a newtype wrapper with transparent error access
///
/// # Examples
///
/// ```rust
/// use koruma_derive_core::DataFieldKorumaAttr;
///
/// let multiple: DataFieldKorumaAttr = syn::parse_quote!(
///     Validator1::a(1),
///     Validator2::b(2)
/// );
/// assert_eq!(multiple.field_validator_count(), 2);
///
/// let with_each: DataFieldKorumaAttr = syn::parse_quote!(
///     VecValidator::min(0),
///     each(ElementValidator::max(100))
/// );
/// assert_eq!(with_each.field_validator_count(), 1);
/// assert_eq!(with_each.element_validator_count(), 1);
///
/// let skip: DataFieldKorumaAttr = syn::parse_quote!(skip);
/// assert!(skip.is_skip());
///
/// let nested: DataFieldKorumaAttr = syn::parse_quote!(nested);
/// assert!(nested.is_nested());
/// ```
#[derive(Clone, Debug, Default)]
pub struct DataFieldKorumaAttr {
    items: Vec<DataFieldKorumaItem>,
}

impl DataFieldKorumaAttr {
    pub fn items(&self) -> &[DataFieldKorumaItem] {
        &self.items
    }

    pub fn into_items(self) -> Vec<DataFieldKorumaItem> {
        self.items
    }

    /// Returns whether this attribute has any validators (field or element).
    pub fn has_validators(&self) -> bool {
        self.items.iter().any(|item| match item {
            DataFieldKorumaItem::FieldValidation(_) => true,
            DataFieldKorumaItem::ElementValidation(spec) => !spec.validators.is_empty(),
            DataFieldKorumaItem::Modifier(_) => false,
        })
    }

    /// Returns whether this attribute represents a modifier (skip, nested, newtype).
    pub fn is_modifier(&self) -> bool {
        self.items
            .iter()
            .any(|item| matches!(item, DataFieldKorumaItem::Modifier(_)))
    }

    pub fn is_skip(&self) -> bool {
        self.items.iter().any(|item| {
            matches!(
                item,
                DataFieldKorumaItem::Modifier(FieldModifier {
                    kind: FieldModifierKind::Skip,
                    ..
                })
            )
        })
    }

    pub fn is_nested(&self) -> bool {
        self.items.iter().any(|item| {
            matches!(
                item,
                DataFieldKorumaItem::Modifier(FieldModifier {
                    kind: FieldModifierKind::Nested,
                    ..
                })
            )
        })
    }

    pub fn is_newtype(&self) -> bool {
        self.items.iter().any(|item| {
            matches!(
                item,
                DataFieldKorumaItem::Modifier(FieldModifier {
                    kind: FieldModifierKind::Newtype,
                    ..
                })
            )
        })
    }

    pub fn field_validators(&self) -> impl Iterator<Item = &ValidatorAttr> {
        self.items.iter().filter_map(|item| match item {
            DataFieldKorumaItem::FieldValidation(spec) => Some(spec.validator.validator()),
            DataFieldKorumaItem::Modifier(_) | DataFieldKorumaItem::ElementValidation(_) => None,
        })
    }

    pub fn element_validators(&self) -> impl Iterator<Item = &ValidatorAttr> {
        self.items.iter().flat_map(|item| {
            match item {
                DataFieldKorumaItem::ElementValidation(spec) => spec.validators.as_slice(),
                DataFieldKorumaItem::Modifier(_) | DataFieldKorumaItem::FieldValidation(_) => &[],
            }
            .iter()
            .map(ParsedValidatorUse::validator)
        })
    }

    pub fn field_validator_count(&self) -> usize {
        self.field_validators().count()
    }

    pub fn element_validator_count(&self) -> usize {
        self.element_validators().count()
    }

    pub fn has_field_validators(&self) -> bool {
        self.field_validators().next().is_some()
    }

    pub fn has_element_validators(&self) -> bool {
        self.element_validators().next().is_some()
    }
}

impl DataFieldKorumaItem {
    pub fn modifier(&self) -> Option<FieldModifierKind> {
        match self {
            DataFieldKorumaItem::Modifier(modifier) => Some(modifier.kind),
            DataFieldKorumaItem::FieldValidation(_) | DataFieldKorumaItem::ElementValidation(_) => {
                None
            },
        }
    }
}

impl Parse for DataFieldKorumaAttr {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.is_empty() {
            return Err(Error::new(
                input.span(),
                "`#[koruma(...)]` must contain a modifier, validator, or `each(...)` block",
            ));
        }

        let mut attr = DataFieldKorumaAttr::default();

        // Parse comma-separated items (validators or each(...))
        while !input.is_empty() {
            if let Some(modifier) = try_parse_field_modifier(input)? {
                attr.items.push(DataFieldKorumaItem::Modifier(modifier));
            } else if let Some(item) = try_parse_each(input)? {
                attr.items.push(item);
            } else {
                attr.items
                    .push(DataFieldKorumaItem::FieldValidation(Box::new(
                        FieldValidationSpec {
                            validator: parse_validator_use(input)?,
                        },
                    )));
            }

            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            } else {
                break;
            }
        }

        Ok(attr)
    }
}

fn try_parse_field_modifier(input: ParseStream) -> Result<Option<FieldModifier>> {
    if !input.peek(Ident) {
        return Ok(None);
    }

    let fork = input.fork();
    let ident: Ident = fork.parse()?;
    if matches!(
        KorumaKeyword::from_ident(&ident),
        Some(KorumaKeyword::Value | KorumaKeyword::TryNew | KorumaKeyword::Setter)
    ) {
        return Err(context_error(&ident, KorumaAttrContext::DataField));
    }

    let kind = match KorumaKeyword::from_ident(&ident) {
        Some(KorumaKeyword::Skip) => FieldModifierKind::Skip,
        Some(KorumaKeyword::Nested) => FieldModifierKind::Nested,
        Some(KorumaKeyword::Newtype) => FieldModifierKind::Newtype,
        _ => return Ok(None),
    };

    if fork.peek(token::Paren) {
        return Err(Error::new(
            ident.span(),
            format!(
                "`{ident}(...)` is not valid in a derive data field `#[koruma(...)]` attribute; expected {}",
                KorumaAttrContext::DataField.accepted_items()
            ),
        ));
    }

    if fork.peek(Token![::]) {
        return Err(Error::new(
            ident.span(),
            format!(
                "`{ident}` is a reserved koruma field modifier; use a different validator path or separate `newtype` from validators with a comma"
            ),
        ));
    }

    if !fork.is_empty() && !fork.peek(Token![,]) {
        return Ok(None);
    }

    let ident = input.parse::<Ident>()?;
    let span = ident.span();
    Ok(Some(FieldModifier {
        kind,
        source: SpannedValue::new(kind, span),
        ident,
    }))
}

fn parse_validator_use(input: ParseStream) -> Result<ParsedValidatorUse> {
    let label = if input.peek(Ident) {
        let fork = input.fork();
        let label: Ident = fork.parse()?;
        if fork.peek(Token![=]) {
            input.parse::<Ident>()?;
            input.parse::<Token![=]>()?;
            Some(label)
        } else {
            None
        }
    } else {
        None
    };

    let (target, validator) = parse_targeted_validator(input)?;
    ParsedValidatorUse::try_new(label, target, validator)
}

fn parse_targeted_validator(
    input: ParseStream,
) -> Result<(ValidatorTargetSelector, ValidatorAttr)> {
    if input.peek(Ident) {
        let fork = input.fork();
        let marker: Ident = fork.parse()?;
        if matches!(
            KorumaKeyword::from_ident(&marker),
            Some(KorumaKeyword::Full | KorumaKeyword::Unwrapped)
        ) {
            if fork.peek(token::Paren) {
                let marker = input.parse::<Ident>()?;
                let content;
                parenthesized!(content in input);
                if content.is_empty() {
                    return Err(Error::new(
                        content.span(),
                        format!("`{marker}(...)` must contain exactly one validator"),
                    ));
                }
                let validator = content.parse::<ValidatorAttr>()?;
                if !content.is_empty() {
                    return Err(Error::new(
                        content.span(),
                        format!("`{marker}(...)` accepts exactly one validator"),
                    ));
                }
                let target = if matches!(
                    KorumaKeyword::from_ident(&marker),
                    Some(KorumaKeyword::Full)
                ) {
                    let span = marker.span();
                    ValidatorTargetSelector::Full {
                        marker: SpannedValue::new(marker, span),
                    }
                } else {
                    let span = marker.span();
                    ValidatorTargetSelector::Unwrapped {
                        marker: SpannedValue::new(marker, span),
                    }
                };
                return Ok((target, validator));
            }

            if !fork.peek(Token![::]) {
                return Err(Error::new(
                    marker.span(),
                    format!(
                        "`{marker}` is a reserved koruma target selector; use `{marker}(Validator::<_>)`"
                    ),
                ));
            }
        }
    }

    Ok((
        ValidatorTargetSelector::Default,
        input.parse::<ValidatorAttr>()?,
    ))
}

fn try_parse_each(input: ParseStream) -> Result<Option<DataFieldKorumaItem>> {
    if !input.peek(Ident) {
        return Ok(None);
    }

    let fork = input.fork();
    let ident: Ident = fork.parse()?;
    if !matches!(KorumaKeyword::from_ident(&ident), Some(KorumaKeyword::Each))
        || !fork.peek(token::Paren)
    {
        if matches!(KorumaKeyword::from_ident(&ident), Some(KorumaKeyword::Each))
            && !fork.peek(Token![::])
        {
            return Err(Error::new(
                ident.span(),
                "`each` is only valid as `each(...)` in a derive data field `#[koruma(...)]` attribute",
            ));
        }
        return Ok(None);
    }

    let marker = input.parse::<Ident>()?;
    let marker_source = SpannedValue::new(marker.clone(), marker.span());
    let content;
    parenthesized!(content in input);
    if content.is_empty() {
        return Err(Error::new(
            content.span(),
            "`each(...)` must contain at least one validator",
        ));
    }

    let mut validators = Vec::new();
    while !content.is_empty() {
        validators.push(parse_validator_use(&content)?);
        if content.peek(Token![,]) {
            content.parse::<Token![,]>()?;
        } else {
            break;
        }
    }

    Ok(Some(DataFieldKorumaItem::ElementValidation(
        ElementValidationSpec {
            marker,
            marker_source,
            validators,
        },
    )))
}

/// Parsed field metadata extracted from all `#[koruma(...)]` attributes on a field.
#[derive(Clone, Debug)]
pub enum ParsedFieldSpec {
    Regular {
        field_validators: Vec<ParsedValidatorUse>,
        element_validators: Vec<ParsedValidatorUse>,
    },
    Nested {
        marker: Ident,
    },
    Newtype {
        marker: Ident,
        field_validators: Vec<ParsedValidatorUse>,
    },
}

/// Source information for a struct field, independent of Koruma participation.
#[derive(Clone, Debug)]
pub struct FieldSource {
    name: Ident,
    member: Member,
    ty: Type,
    index: usize,
}

impl FieldSource {
    fn from_field(field: &Field, index: usize) -> Self {
        let (name, member) = match field.ident.clone() {
            Some(ident) => (ident.clone(), Member::Named(ident)),
            None => (
                quote::format_ident!("_{}", index),
                Member::Unnamed(Index::from(index)),
            ),
        };

        Self {
            name,
            member,
            ty: field.ty.clone(),
            index,
        }
    }

    pub fn new(name: Ident, member: Member, ty: Type, index: usize) -> Self {
        Self {
            name,
            member,
            ty,
            index,
        }
    }

    pub fn name(&self) -> &Ident {
        &self.name
    }

    pub fn member(&self) -> &Member {
        &self.member
    }

    pub fn ty(&self) -> &Type {
        &self.ty
    }

    pub fn index(&self) -> usize {
        self.index
    }
}

/// Parsed data-field participation after field-level `#[koruma(...)]` parsing.
#[derive(Clone, Debug)]
pub enum ParsedDataField {
    Unannotated(FieldSource),
    Skipped {
        source: FieldSource,
        marker: SpannedValue<Ident>,
    },
    Participating(FieldInfo),
}

impl ParsedDataField {
    pub fn source(&self) -> &FieldSource {
        match self {
            Self::Unannotated(source) | Self::Skipped { source, .. } => source,
            Self::Participating(info) => info.source(),
        }
    }

    pub fn participating(self) -> Option<FieldInfo> {
        match self {
            Self::Participating(info) => Some(info),
            Self::Unannotated(_) | Self::Skipped { .. } => None,
        }
    }
}

/// Field information extracted from parsing `#[koruma(...)]` attributes.
///
/// This struct contains all the parsed validation information for a single field,
/// including validators, element validators (for collection), and modifier flags.
#[derive(Clone, Debug)]
pub struct FieldInfo {
    source: FieldSource,
    validation: ParsedFieldSpec,
}

impl FieldInfo {
    fn new(source: FieldSource, validation: ParsedFieldSpec) -> Result<Self> {
        Ok(Self { source, validation })
    }

    pub fn synthetic_newtype(source: FieldSource, marker: Ident) -> Self {
        Self {
            source,
            validation: ParsedFieldSpec::Newtype {
                marker,
                field_validators: Vec::new(),
            },
        }
    }

    pub fn source(&self) -> &FieldSource {
        &self.source
    }

    pub fn name(&self) -> &Ident {
        self.source.name()
    }

    pub fn member(&self) -> &Member {
        self.source.member()
    }

    pub fn ty(&self) -> &Type {
        self.source.ty()
    }

    pub fn index(&self) -> usize {
        self.source.index()
    }

    pub fn validation(&self) -> &ParsedFieldSpec {
        &self.validation
    }

    pub fn field_validators(&self) -> &[ParsedValidatorUse] {
        match &self.validation {
            ParsedFieldSpec::Regular {
                field_validators, ..
            }
            | ParsedFieldSpec::Newtype {
                field_validators, ..
            } => field_validators,
            ParsedFieldSpec::Nested { .. } => &[],
        }
    }

    pub fn element_validators(&self) -> &[ParsedValidatorUse] {
        match &self.validation {
            ParsedFieldSpec::Regular {
                element_validators, ..
            } => element_validators,
            ParsedFieldSpec::Nested { .. } | ParsedFieldSpec::Newtype { .. } => &[],
        }
    }

    /// Returns true if this field has element validators (uses `each(...)`)
    pub fn has_element_validators(&self) -> bool {
        !self.element_validators().is_empty()
    }

    /// Returns true if this field has any validators (field or element)
    pub fn has_validators(&self) -> bool {
        !self.field_validators().is_empty() || !self.element_validators().is_empty()
    }

    /// Returns true if this field is a nested Koruma struct
    pub fn is_nested(&self) -> bool {
        matches!(self.validation, ParsedFieldSpec::Nested { .. })
    }

    /// Returns true if this field is a newtype wrapper
    pub fn is_newtype(&self) -> bool {
        matches!(self.validation, ParsedFieldSpec::Newtype { .. })
    }

    /// Returns an iterator over all validator names on this field.
    pub fn validator_names(&self) -> impl Iterator<Item = &Ident> {
        self.field_validators()
            .iter()
            .chain(self.element_validators().iter())
            .map(|v| v.validator().name())
    }
}

/// Parse a single field and extract its koruma validation information.
///
/// This function handles:
/// - Preserving optional validator labels for downstream name generation
/// - The `skip`, `nested`, and `newtype` modifiers
///
/// # Returns
///
/// - `Ok(ParsedDataField::Participating(FieldInfo))` if the field participates in validation.
/// - `Ok(ParsedDataField::Unannotated(_))` if the field has no koruma attributes.
/// - `Ok(ParsedDataField::Skipped { .. })` if the field is marked with `skip`.
/// - `Err(Error)` if parsing failed, such as duplicate or conflicting modifiers.
pub fn parse_field(field: &Field, index: usize) -> Result<ParsedDataField> {
    let source = FieldSource::from_field(field, index);

    let attrs = field.attrs.to_vec();
    let koruma_attrs = attrs.find_attribute("koruma");
    let items = match koruma_attrs.as_slice() {
        [] => Vec::new(),
        [attr] => attr.parse_args::<DataFieldKorumaAttr>()?.into_items(),
        [_, duplicate, ..] => {
            return Err(Error::new(
                duplicate.path().span(),
                "only one field-level `#[koruma(...)]` attribute is allowed; combine validators and modifiers in a single attribute",
            ));
        },
    };

    let validation = normalize_field_items(field, items)?;
    match validation {
        NormalizedFieldSpec::Unannotated => Ok(ParsedDataField::Unannotated(source)),
        NormalizedFieldSpec::Skipped { marker } => Ok(ParsedDataField::Skipped { source, marker }),
        NormalizedFieldSpec::Participating(validation) => Ok(ParsedDataField::Participating(
            FieldInfo::new(source, validation)?,
        )),
    }
}

enum NormalizedFieldSpec {
    Unannotated,
    Skipped { marker: SpannedValue<Ident> },
    Participating(ParsedFieldSpec),
}

fn normalize_field_items(
    _field: &Field,
    items: Vec<DataFieldKorumaItem>,
) -> Result<NormalizedFieldSpec> {
    let mut all_field_validators = Vec::new();
    let mut all_element_validators = Vec::new();
    let mut mode_modifier: Option<FieldModifier> = None;
    let mut first_field_validator_path: Option<Path> = None;
    let mut first_element_marker: Option<SpannedValue<Ident>> = None;

    for item in items {
        match item {
            DataFieldKorumaItem::Modifier(modifier) => {
                if mode_modifier.is_some() {
                    return Err(Error::new(
                        modifier.span(),
                        "duplicate or conflicting field modifier in `#[koruma(...)]`",
                    ));
                }
                mode_modifier = Some(modifier);
            },
            DataFieldKorumaItem::FieldValidation(spec) => {
                let FieldValidationSpec { validator } = *spec;
                if first_field_validator_path.is_none() {
                    first_field_validator_path = Some(validator.validator().path().clone());
                }
                all_field_validators.push(validator);
            },
            DataFieldKorumaItem::ElementValidation(spec) => {
                if first_element_marker.is_none() {
                    first_element_marker = Some(spec.marker_source.clone());
                }
                for validator in spec.validators {
                    all_element_validators.push(validator);
                }
            },
        }
    }

    let has_field_validators = !all_field_validators.is_empty();
    let has_element_validators = !all_element_validators.is_empty();

    let conflict_span = |fallback: proc_macro2::Span| {
        first_field_validator_path
            .as_ref()
            .map(Spanned::span)
            .or_else(|| first_element_marker.as_ref().map(|marker| marker.span))
            .unwrap_or(fallback)
    };

    let spec = match mode_modifier {
        Some(modifier) => match modifier.kind {
            FieldModifierKind::Skip => {
                if has_field_validators || has_element_validators {
                    return Err(Error::new(
                        conflict_span(modifier.span()),
                        "fields marked `#[koruma(skip)]` cannot also use validators or `each(...)`",
                    ));
                }

                let marker = SpannedValue::new(modifier.ident, modifier.source.span);
                return Ok(NormalizedFieldSpec::Skipped { marker });
            },
            FieldModifierKind::Nested => {
                if has_field_validators || has_element_validators {
                    return Err(Error::new(
                        conflict_span(modifier.span()),
                        "fields marked `#[koruma(nested)]` cannot also use validators or `each(...)`",
                    ));
                }

                ParsedFieldSpec::Nested {
                    marker: modifier.ident,
                }
            },
            FieldModifierKind::Newtype => {
                if has_element_validators {
                    return Err(Error::new(
                        first_element_marker
                            .as_ref()
                            .map(|marker| marker.span)
                            .unwrap_or_else(|| modifier.span()),
                        "fields marked `#[koruma(newtype)]` cannot also use `each(...)`; validate elements before wrapping or attach validators to the inner type",
                    ));
                }

                ParsedFieldSpec::Newtype {
                    marker: modifier.ident,
                    field_validators: all_field_validators,
                }
            },
        },
        None if !has_field_validators && !has_element_validators => {
            return Ok(NormalizedFieldSpec::Unannotated);
        },
        None => ParsedFieldSpec::Regular {
            field_validators: all_field_validators,
            element_validators: all_element_validators,
        },
    };

    Ok(NormalizedFieldSpec::Participating(spec))
}
