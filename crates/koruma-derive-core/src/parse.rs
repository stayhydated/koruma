//! Parsing logic for `#[koruma(...)]` attributes.
//!
//! This module provides types and functions for parsing koruma validation
//! attributes from syn AST nodes.

use heck::{ToSnakeCase, ToUpperCamelCase};
use syn::{
    Attribute, Error, Expr, Field, Fields, GenericArgument, Ident, Index, ItemStruct, Member, Path,
    PathArguments, Result, Token, Type, parenthesized,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    spanned::Spanned,
    token,
};

use syn_cfg_attr::{AttributeHelpers, ExpandedAttr};

/// Represents a single parsed validator: `ValidatorName(arg = value, ...)`,
/// `ValidatorName<_>(arg = value, ...)`, or `ValidatorName<SomeType>(arg = value, ...)`.
/// Also supports fully-qualified paths like `module::path::ValidatorName<_>`.
///
/// Uses angle bracket syntax (`<>`) for type parameters, which keeps the
/// attribute surface compact while still naturally handling nested generics
/// like `Validator<Option<Vec<T>>>`.
///
/// # Examples
///
/// ```ignore
/// // Simple validator
/// #[koruma(NonEmptyValidation)]
///
/// // Validator with type inference
/// #[koruma(RangeValidation<_>(min = 0, max = 100))]
///
/// // Validator with explicit type
/// #[koruma(RangeValidation<i32>(min = 0, max = 100))]
///
/// // Full path
/// #[koruma(validators::numeric::RangeValidation<_>(min = 0))]
/// ```
#[derive(Clone, Debug)]
pub struct ValidatorAttr {
    /// The validator path, which may be a simple identifier or a full path.
    /// Examples: `StringLengthValidation`, `validators::normal::NumberRangeValidation`
    pub validator: Path,
    /// Whether the validator uses generic placeholder syntax like `<_>` for type
    /// inference from the field type.
    /// When true, the field type is used (unwrapping Option if present).
    pub infer_type: bool,
    /// Explicit type parameter if specified (e.g., `<f64>`, `<Vec<_>>`)
    /// If this contains `_`, it will be substituted with the inner type from the field.
    /// Use `<Option<_>>` to get the full Option type without unwrapping.
    pub explicit_type: Option<Type>,
    /// Key-value argument pairs passed to the validator.
    pub args: Vec<(Ident, Expr)>,
}

impl ValidatorAttr {
    /// Returns the simple name of the validator (the last segment of the path).
    /// Used for generating field names and enum variants.
    pub fn name(&self) -> &Ident {
        &self
            .validator
            .segments
            .last()
            .expect("path should have at least one segment")
            .ident
    }

    /// Returns the full validator path as written, without generic arguments.
    pub fn path_name(&self) -> String {
        self.validator
            .segments
            .iter()
            .map(|segment| segment.ident.to_string())
            .collect::<Vec<_>>()
            .join("::")
    }

    /// Returns a stable snake_case stem for generated field and getter names.
    ///
    /// This returns the fully qualified path flattened into snake_case.
    /// Callers that need to resolve collisions should combine this with
    /// additional disambiguation logic.
    pub fn codegen_snake_name(&self) -> String {
        self.validator
            .segments
            .iter()
            .map(|segment| segment.ident.to_string().to_snake_case())
            .collect::<Vec<_>>()
            .join("_")
    }

    /// Returns a stable UpperCamelCase stem for generated enum variants.
    ///
    /// This returns the fully qualified path flattened into UpperCamelCase.
    /// Callers that need to resolve collisions should combine this with
    /// additional disambiguation logic.
    pub fn codegen_upper_camel_name(&self) -> String {
        self.validator
            .segments
            .iter()
            .map(|segment| segment.ident.to_string().to_upper_camel_case())
            .collect::<Vec<_>>()
            .join("")
    }

    /// Returns whether this validator has any arguments.
    pub fn has_args(&self) -> bool {
        !self.args.is_empty()
    }

    /// Returns whether this validator uses type inference (`<_>` syntax).
    pub fn uses_type_inference(&self) -> bool {
        self.infer_type
    }

    /// Returns whether this validator has an explicit type parameter.
    pub fn has_explicit_type(&self) -> bool {
        self.explicit_type.is_some()
    }
}

impl Parse for ValidatorAttr {
    fn parse(input: ParseStream) -> Result<Self> {
        // Parse validator path first; bare `<...>` generic args remain in the token
        // stream, while turbofish `::<...>` is captured on the final path segment.
        let mut validator: Path = input.parse()?;
        let validator_span = validator.span();
        let last_segment = validator
            .segments
            .last_mut()
            .ok_or_else(|| Error::new(validator_span, "expected validator path"))?;

        if let PathArguments::AngleBracketed(angle_args) = &last_segment.arguments
            && angle_args.colon2_token.is_some()
        {
            return Err(Error::new(
                angle_args.span(),
                "use angle bracket syntax `<...>` instead of turbofish `::<...>` in `#[koruma(...)]`",
            ));
        }

        // Support `Validator<_>` by attaching the parsed generic args to the last
        // path segment before extracting them into `ValidatorAttr`.
        if input.peek(Token![<]) {
            let angle_args: syn::AngleBracketedGenericArguments = input.parse()?;

            if !matches!(last_segment.arguments, PathArguments::None) {
                return Err(Error::new(
                    angle_args.span(),
                    "validator type syntax can only appear once",
                ));
            }

            last_segment.arguments = PathArguments::AngleBracketed(angle_args);
        }

        // Check for generic syntax: `<_>` or `<SomeType>`.
        // `<_>` means "use the field type" (unwrapping Option if present).
        // `<Option<_>>` means "use the full Option type" (without unwrapping).
        // `<Vec<_>>` means "substitute _ with the inner type from the field".
        let (infer_type, explicit_type) = {
            let last_segment = validator
                .segments
                .last_mut()
                .ok_or_else(|| Error::new(validator_span, "expected validator path"))?;

            // Strip generic args from stored validator path and parse them into fields.
            let args = std::mem::replace(&mut last_segment.arguments, PathArguments::None);
            match args {
                PathArguments::None => (false, None),
                PathArguments::AngleBracketed(mut angle_args) => {
                    if angle_args.args.len() != 1 {
                        return Err(Error::new(
                            angle_args.span(),
                            "validator type syntax expects exactly one type argument",
                        ));
                    }

                    let arg = angle_args.args.pop().expect("len checked").into_value();
                    match arg {
                        GenericArgument::Type(Type::Infer(_)) => (true, None),
                        GenericArgument::Type(ty) => (false, Some(ty)),
                        _ => Err(Error::new(
                            arg.span(),
                            "validator type syntax expects a type argument",
                        ))?,
                    }
                },
                PathArguments::Parenthesized(args) => {
                    return Err(Error::new(
                        args.span(),
                        "validator path does not support parenthesized arguments",
                    ));
                },
            }
        };

        let args = if input.peek(token::Paren) {
            let content;
            parenthesized!(content in input);

            let mut args = Vec::new();
            while !content.is_empty() {
                let name: Ident = content.parse()?;
                content.parse::<Token![=]>()?;
                let value: Expr = content.parse()?;

                args.push((name, value));

                if content.peek(Token![,]) {
                    content.parse::<Token![,]>()?;
                }
            }
            args
        } else {
            Vec::new()
        };

        Ok(ValidatorAttr {
            validator,
            infer_type,
            explicit_type,
            args,
        })
    }
}

/// Represents a parsed `#[koruma(...)]` attribute which can contain multiple validators
/// separated by commas: `#[koruma(Validator1(a = 1), Validator2(b = 2))]`
///
/// Can also include:
/// - `each(...)` modifier for collection validation
/// - `skip` to skip validation for a field
/// - `nested` to validate nested structs that also derive Koruma
/// - `newtype` to validate a newtype wrapper with transparent error access
///
/// # Examples
///
/// ```ignore
/// // Multiple validators
/// #[koruma(Validator1(a = 1), Validator2(b = 2))]
///
/// // Element validation for collections
/// #[koruma(VecValidator(min = 0), each(ElementValidator(max = 100)))]
///
/// // Skip validation
/// #[koruma(skip)]
///
/// // Nested Koruma struct
/// #[koruma(nested)]
/// ```
#[derive(Clone, Debug, Default)]
pub struct KorumaAttr {
    /// Validators applied to the field/collection itself
    pub field_validators: Vec<ValidatorAttr>,
    /// Validators applied to each element in a collection (from `each(...)`)
    pub element_validators: Vec<ValidatorAttr>,
    /// Whether this field should be skipped
    pub is_skip: bool,
    /// Whether this field is a nested Koruma struct
    pub is_nested: bool,
    /// Whether this field is a newtype wrapper (single-field struct deriving Koruma).
    /// Similar to nested, but generates a wrapper error struct with Deref for transparent access.
    pub is_newtype: bool,
}

impl KorumaAttr {
    /// Returns whether this attribute has any validators (field or element).
    pub fn has_validators(&self) -> bool {
        !self.field_validators.is_empty() || !self.element_validators.is_empty()
    }

    /// Returns whether this attribute represents a modifier (skip, nested, newtype).
    pub fn is_modifier(&self) -> bool {
        self.is_skip || self.is_nested || self.is_newtype
    }
}

impl Parse for KorumaAttr {
    fn parse(input: ParseStream) -> Result<Self> {
        // Check for skip, nested, or newtype
        if input.peek(Ident) {
            let fork = input.fork();
            let ident: Ident = fork.parse()?;
            if ident == "skip" && fork.is_empty() {
                input.parse::<Ident>()?; // consume "skip"
                return Ok(KorumaAttr {
                    field_validators: Vec::new(),
                    element_validators: Vec::new(),
                    is_skip: true,
                    is_nested: false,
                    is_newtype: false,
                });
            }
            // Check for nested
            if ident == "nested" && fork.is_empty() {
                input.parse::<Ident>()?; // consume "nested"
                return Ok(KorumaAttr {
                    field_validators: Vec::new(),
                    element_validators: Vec::new(),
                    is_skip: false,
                    is_nested: true,
                    is_newtype: false,
                });
            }
            // Check for newtype - can be standalone or followed by validators
            if ident == "newtype" {
                // Check if newtype is followed by a comma or end of input
                input.parse::<Ident>()?; // consume "newtype"

                // Check for comma followed by validators
                if input.peek(Token![,]) {
                    input.parse::<Token![,]>()?; // consume comma
                    // Continue parsing validators below
                    let mut field_validators = Vec::new();
                    let mut element_validators = Vec::new();

                    while !input.is_empty() {
                        // Check if this is an `each(...)` block
                        if input.peek(Ident) {
                            let fork = input.fork();
                            let ident: Ident = fork.parse()?;
                            if ident == "each" && fork.peek(token::Paren) {
                                input.parse::<Ident>()?; // consume "each"
                                let content;
                                parenthesized!(content in input);

                                while !content.is_empty() {
                                    element_validators.push(content.parse::<ValidatorAttr>()?);
                                    if content.peek(Token![,]) {
                                        content.parse::<Token![,]>()?;
                                    } else {
                                        break;
                                    }
                                }

                                if input.peek(Token![,]) {
                                    input.parse::<Token![,]>()?;
                                }
                                continue;
                            }
                        }

                        field_validators.push(input.parse::<ValidatorAttr>()?);
                        if input.peek(Token![,]) {
                            input.parse::<Token![,]>()?;
                        } else {
                            break;
                        }
                    }

                    return Ok(KorumaAttr {
                        field_validators,
                        element_validators,
                        is_skip: false,
                        is_nested: false,
                        is_newtype: true,
                    });
                }

                // Standalone newtype
                if input.is_empty() {
                    return Ok(KorumaAttr {
                        field_validators: Vec::new(),
                        element_validators: Vec::new(),
                        is_skip: false,
                        is_nested: false,
                        is_newtype: true,
                    });
                }
            }
        }

        let mut field_validators = Vec::new();
        let mut element_validators = Vec::new();

        // Parse comma-separated items (validators or each(...))
        while !input.is_empty() {
            // Check if this is an `each(...)` block
            if input.peek(Ident) {
                let fork = input.fork();
                let ident: Ident = fork.parse()?;
                if ident == "each" && fork.peek(token::Paren) {
                    input.parse::<Ident>()?; // consume "each"
                    let content;
                    parenthesized!(content in input);

                    // Parse validators inside each(...)
                    while !content.is_empty() {
                        element_validators.push(content.parse::<ValidatorAttr>()?);
                        if content.peek(Token![,]) {
                            content.parse::<Token![,]>()?;
                        } else {
                            break;
                        }
                    }

                    // Continue parsing after each(...)
                    if input.peek(Token![,]) {
                        input.parse::<Token![,]>()?;
                    }
                    continue;
                }
            }

            // Regular validator
            field_validators.push(input.parse::<ValidatorAttr>()?);
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            } else {
                break;
            }
        }

        Ok(KorumaAttr {
            field_validators,
            element_validators,
            is_skip: false,
            is_nested: false,
            is_newtype: false,
        })
    }
}

/// Struct-level options parsed from `#[koruma(...)]`
///
/// # Examples
///
/// ```ignore
/// // Generate try_new constructor
/// #[koruma(try_new)]
/// #[derive(Koruma)]
/// struct User { ... }
///
/// // Newtype wrapper
/// #[koruma(newtype)]
/// #[derive(Koruma)]
/// struct Email(String);
///
/// // Both options
/// #[koruma(try_new, newtype)]
/// #[derive(Koruma)]
/// struct Email(String);
///
/// // TryFrom impl for newtypes - converts inner type to validated wrapper
/// #[koruma(newtype(try_from))]
/// #[derive(Koruma)]
/// struct Email(String);
/// ```
#[derive(Clone, Debug, Default)]
pub struct StructOptions {
    /// Generate a `try_new` function that validates on construction
    pub try_new: bool,
    /// Treat this struct as a newtype (single-field wrapper).
    /// Generates an `.all()` method on the error struct that aggregates
    /// all validators from the single field.
    pub newtype: bool,
    /// Generate a `TryFrom<Inner>` impl for newtype structs.
    /// Set via `newtype(try_from)`. Implies `newtype`. Does NOT imply `try_new`.
    pub try_from: bool,
}

/// Options for the `newtype(...)` attribute.
#[derive(Clone, Debug, Default)]
pub struct NewtypeOptions {
    pub try_from: bool,
}

impl Parse for NewtypeOptions {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut options = NewtypeOptions::default();

        while !input.is_empty() {
            let ident: Ident = input.parse()?;
            match ident.to_string().as_str() {
                "try_from" => options.try_from = true,
                other => {
                    return Err(Error::new(
                        ident.span(),
                        format!("unknown newtype option: `{}`. Expected `try_from`", other),
                    ));
                },
            }

            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(options)
    }
}

impl Parse for StructOptions {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut options = StructOptions::default();

        while !input.is_empty() {
            let ident: Ident = input.parse()?;
            match ident.to_string().as_str() {
                "try_new" => options.try_new = true,
                "newtype" => {
                    options.newtype = true;
                    // Check for nested options: newtype(try_from)
                    if input.peek(syn::token::Paren) {
                        let content;
                        syn::parenthesized!(content in input);
                        let newtype_opts: NewtypeOptions = content.parse()?;
                        options.try_from = newtype_opts.try_from;
                    }
                },
                other => {
                    return Err(Error::new(
                        ident.span(),
                        format!(
                            "unknown struct-level koruma option: `{}`. Expected `try_new` or `newtype`",
                            other
                        ),
                    ));
                },
            }

            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(options)
    }
}

/// Parse struct-level `#[koruma(...)]` attributes from a list of attributes.
///
/// Returns `StructOptions::default()` if no `#[koruma(...)]` attribute is found.
pub fn parse_struct_options(attrs: &[Attribute]) -> Result<StructOptions> {
    let mut merged = StructOptions::default();

    for attr in attrs.to_vec().find_attribute("koruma") {
        let parsed = attr.parse_args::<StructOptions>()?;

        if parsed.try_new {
            if merged.try_new {
                return Err(Error::new(
                    attr.path().span(),
                    "duplicate struct-level koruma option `try_new`",
                ));
            }
            merged.try_new = true;
        }

        if parsed.newtype {
            if merged.newtype {
                return Err(Error::new(
                    attr.path().span(),
                    "duplicate struct-level koruma option `newtype`",
                ));
            }
            merged.newtype = true;
        }

        if parsed.try_from {
            if merged.try_from {
                return Err(Error::new(
                    attr.path().span(),
                    "duplicate struct-level koruma option `newtype(try_from)`",
                ));
            }
            merged.try_from = true;
        }
    }

    Ok(merged)
}

/// Validation information extracted from `#[koruma(...)]` attributes.
#[derive(Clone, Debug, Default)]
pub struct ValidationInfo {
    /// Validators for the field/collection itself
    pub field_validators: Vec<ValidatorAttr>,
    /// Validators for each element in a collection
    pub element_validators: Vec<ValidatorAttr>,
    /// Whether this field is a nested Koruma struct
    pub is_nested: bool,
    /// Whether this field is a newtype wrapper
    pub is_newtype: bool,
}

/// Field information extracted from parsing `#[koruma(...)]` attributes.
///
/// This struct contains all the parsed validation information for a single field,
/// including validators, element validators (for collection), and modifier flags.
#[derive(Clone, Debug)]
pub struct FieldInfo {
    /// The field name
    pub name: Ident,
    /// The struct member access (Named or Unnamed index)
    pub member: Member,
    /// The field type
    pub ty: Type,
    /// Validation info for this field
    pub validation: ValidationInfo,
}

impl FieldInfo {
    /// Returns true if this field has element validators (uses `each(...)`)
    pub fn has_element_validators(&self) -> bool {
        !self.validation.element_validators.is_empty()
    }

    /// Returns true if this field has any validators (field or element)
    pub fn has_validators(&self) -> bool {
        !self.validation.field_validators.is_empty()
            || !self.validation.element_validators.is_empty()
    }

    /// Returns true if this field is a nested Koruma struct
    pub fn is_nested(&self) -> bool {
        self.validation.is_nested
    }

    /// Returns true if this field is a newtype wrapper
    pub fn is_newtype(&self) -> bool {
        self.validation.is_newtype
    }

    /// Returns an iterator over all validator names on this field.
    pub fn validator_names(&self) -> impl Iterator<Item = &Ident> {
        self.validation
            .field_validators
            .iter()
            .chain(self.validation.element_validators.iter())
            .map(|v| v.name())
    }
}

/// Result of parsing a field with `#[koruma(...)]` attribute.
///
/// This enum represents the three possible outcomes of parsing a field:
/// - `Valid`: The field has valid koruma validators
/// - `Skip`: The field should be skipped (no koruma attribute, or `#[koruma(skip)]`)
/// - `Error`: A parse error occurred
#[derive(Debug)]
pub enum ParseFieldResult {
    /// Field has valid koruma validators
    Valid(Box<FieldInfo>),
    /// Field should be skipped (no koruma attribute, or #[koruma(skip)])
    Skip,
    /// Parse error occurred
    Error(Error),
}

impl ParseFieldResult {
    /// Returns the field info if this is a `Valid` result.
    pub fn valid(self) -> Option<FieldInfo> {
        match self {
            ParseFieldResult::Valid(info) => Some(*info),
            _ => None,
        }
    }

    /// Returns the error if this is an `Error` result.
    pub fn error(self) -> Option<Error> {
        match self {
            ParseFieldResult::Error(e) => Some(e),
            _ => None,
        }
    }

    /// Returns true if this is a `Valid` result.
    pub fn is_valid(&self) -> bool {
        matches!(self, ParseFieldResult::Valid(_))
    }

    /// Returns true if this is a `Skip` result.
    pub fn is_skip(&self) -> bool {
        matches!(self, ParseFieldResult::Skip)
    }

    /// Returns true if this is an `Error` result.
    pub fn is_error(&self) -> bool {
        matches!(self, ParseFieldResult::Error(_))
    }
}

/// Parse a single field and extract its koruma validation information.
///
/// This function handles:
/// - Multiple `#[koruma(...)]` attributes on the same field
/// - Combining validators from multiple attributes
/// - Detecting duplicate validators
/// - The `skip`, `nested`, and `newtype` modifiers
///
/// # Returns
///
/// - `ParseFieldResult::Valid(FieldInfo)` if the field has validators
/// - `ParseFieldResult::Skip` if the field has no koruma attributes or is marked with `skip`
/// - `ParseFieldResult::Error(Error)` if parsing failed (e.g., duplicate validators)
/// - `ParseFieldResult::Valid(FieldInfo)` if the field has validators
/// - `ParseFieldResult::Skip` if the field has no koruma attributes or is marked with `skip`
/// - `ParseFieldResult::Error(Error)` if parsing failed (e.g., duplicate validators)
pub fn parse_field(field: &Field, index: usize) -> ParseFieldResult {
    let (name, member) = match field.ident.clone() {
        Some(ident) => (ident.clone(), Member::Named(ident)),
        None => (
            quote::format_ident!("_{}", index),
            Member::Unnamed(Index::from(index)),
        ),
    };
    let ty = field.ty.clone();

    // Collect validators from ALL #[koruma(...)] attributes on this field
    let mut all_field_validators = Vec::new();
    let mut all_element_validators = Vec::new();
    let mut is_skip = false;
    let mut is_nested = false;
    let mut is_newtype = false;

    // Track seen validator names to detect duplicates
    let mut seen_field_validators = std::collections::HashSet::new();
    let mut seen_element_validators = std::collections::HashSet::new();

    for attr in field.attrs.to_vec().find_attribute("koruma") {
        // Parse the attribute content
        let parsed: Result<KorumaAttr> = attr.parse_args::<KorumaAttr>();

        match parsed {
            Ok(koruma_attr) => {
                // Check for skip - if any attribute says skip, skip the field
                if koruma_attr.is_skip {
                    is_skip = true;
                    continue;
                }
                // Check for nested
                if koruma_attr.is_nested {
                    is_nested = true;
                    continue;
                }
                // Check for newtype
                if koruma_attr.is_newtype {
                    is_newtype = true;
                    // Don't continue here - newtype can have validators too
                    // e.g., #[koruma(newtype, RequiredValidation)]
                }
                // Collect validators from this attribute, checking for duplicates
                for validator in koruma_attr.field_validators {
                    let validator_name = validator.path_name();
                    if !seen_field_validators.insert(validator_name.clone()) {
                        return ParseFieldResult::Error(Error::new(
                            validator.validator.span(),
                            format!(
                                "duplicate validator `{}` on field `{}`",
                                validator_name, name
                            ),
                        ));
                    }
                    all_field_validators.push(validator);
                }
                for validator in koruma_attr.element_validators {
                    let validator_name = validator.path_name();
                    if !seen_element_validators.insert(validator_name.clone()) {
                        return ParseFieldResult::Error(Error::new(
                            validator.validator.span(),
                            format!(
                                "duplicate element validator `{}` on field `{}`",
                                validator_name, name
                            ),
                        ));
                    }
                    all_element_validators.push(validator);
                }
            },
            Err(e) => {
                return ParseFieldResult::Error(e);
            },
        }
    }

    // If skip was specified, skip the field
    if is_skip {
        return ParseFieldResult::Skip;
    }

    if is_nested && is_newtype {
        return ParseFieldResult::Error(Error::new_spanned(
            field,
            "fields cannot combine `#[koruma(nested)]` and `#[koruma(newtype)]`, even across multiple `#[koruma(...)]` attributes",
        ));
    }

    if is_newtype && !all_element_validators.is_empty() {
        return ParseFieldResult::Error(Error::new_spanned(
            field,
            "fields marked `#[koruma(newtype)]` cannot also use `each(...)`; element validation is not supported for newtype wrappers",
        ));
    }

    // Check for nested
    if is_nested {
        return ParseFieldResult::Valid(Box::new(FieldInfo {
            name,
            member: member.clone(),
            ty,
            validation: ValidationInfo {
                field_validators: all_field_validators,
                element_validators: all_element_validators,
                is_nested: true,
                is_newtype: false,
            },
        }));
    }

    // Check for newtype
    if is_newtype {
        return ParseFieldResult::Valid(Box::new(FieldInfo {
            name,
            member: member.clone(),
            ty,
            validation: ValidationInfo {
                field_validators: all_field_validators,
                element_validators: all_element_validators,
                is_nested: false,
                is_newtype: true,
            },
        }));
    }

    // Must have at least one validator or modifier
    if all_field_validators.is_empty() && all_element_validators.is_empty() {
        return ParseFieldResult::Skip;
    }

    ParseFieldResult::Valid(Box::new(FieldInfo {
        name,
        member: member.clone(),
        ty,
        validation: ValidationInfo {
            field_validators: all_field_validators,
            element_validators: all_element_validators,
            is_nested: false,
            is_newtype: false,
        },
    }))
}

struct ValidatorFieldMarkers(Punctuated<Ident, Token![,]>);

impl Parse for ValidatorFieldMarkers {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self(Punctuated::<Ident, Token![,]>::parse_terminated(
            input,
        )?))
    }
}

fn validator_field_markers(attr: &ExpandedAttr) -> Result<Punctuated<Ident, Token![,]>> {
    attr.parse_args::<ValidatorFieldMarkers>()
        .map(|markers| markers.0)
}

/// Describes whether the `#[koruma(value)]` field should capture the input
/// value in derived validation errors.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ValueFieldCapture {
    /// Store the validated value in the generated validator instance.
    Capture,
    /// Skip storing the validated value and leave the field at its default.
    Skip,
}

/// Parsed information about the field marked with `#[koruma(value)]`.
#[derive(Clone, Debug)]
pub struct ValueFieldInfo {
    pub name: Ident,
    pub ty: Type,
    pub capture: ValueFieldCapture,
}

/// Find the field marked with `#[koruma(value)]` and return its parsed info.
///
/// This strict variant validates that validator structs use exactly one
/// `#[koruma(value)]` marker and that validator-field `#[koruma(...)]`
/// attributes only contain the `value` and `skip_capture` markers.
pub fn find_value_field_info_strict(input: &ItemStruct) -> Result<Option<ValueFieldInfo>> {
    let Fields::Named(ref fields) = input.fields else {
        return Ok(None);
    };

    let mut found: Option<ValueFieldInfo> = None;

    for field in &fields.named {
        let Some(field_name) = field.ident.clone() else {
            continue;
        };
        let mut field_has_value = false;
        let mut field_skip_capture = false;

        for attr in field.attrs.to_vec().find_attribute("koruma") {
            let markers = validator_field_markers(&attr)?;
            if markers.is_empty() {
                return Err(Error::new_spanned(
                    attr.path(),
                    "validator fields only support `#[koruma(value)]` and `#[koruma(skip_capture)]`",
                ));
            }

            for marker in markers {
                if marker == "value" {
                    if field_has_value {
                        return Err(Error::new(
                            marker.span(),
                            format!("field `{field_name}` has multiple `#[koruma(value)]` markers"),
                        ));
                    }

                    field_has_value = true;
                    continue;
                }

                if marker == "skip_capture" {
                    if field_skip_capture {
                        return Err(Error::new(
                            marker.span(),
                            format!(
                                "field `{field_name}` has multiple `#[koruma(skip_capture)]` markers"
                            ),
                        ));
                    }

                    field_skip_capture = true;
                    continue;
                }

                return Err(Error::new(
                    marker.span(),
                    "validator fields only support `#[koruma(value)]` and `#[koruma(skip_capture)]`",
                ));
            }
        }

        if field_skip_capture && !field_has_value {
            return Err(Error::new(
                field_name.span(),
                format!(
                    "field `{field_name}` uses `#[koruma(skip_capture)]` but is missing `#[koruma(value)]`"
                ),
            ));
        }

        if field_has_value {
            if let Some(existing) = &found {
                return Err(Error::new(
                    field_name.span(),
                    format!(
                        "koruma::validator requires exactly one `#[koruma(value)]` field, found both `{}` and `{}`",
                        existing.name, field_name
                    ),
                ));
            }

            found = Some(ValueFieldInfo {
                name: field_name,
                ty: field.ty.clone(),
                capture: if field_skip_capture {
                    ValueFieldCapture::Skip
                } else {
                    ValueFieldCapture::Capture
                },
            });
        }
    }

    Ok(found)
}

/// Find the field marked with `#[koruma(value)]` and return its name and type.
///
/// This strict variant validates that validator structs use exactly one
/// `#[koruma(value)]` marker and that validator-field `#[koruma(...)]`
/// attributes only contain the `value` and `skip_capture` markers.
pub fn find_value_field_strict(input: &ItemStruct) -> Result<Option<(Ident, Type)>> {
    Ok(find_value_field_info_strict(input)?.map(|info| (info.name, info.ty)))
}

/// Find the field marked with `#[koruma(value)]` and return its name and type.
///
/// This is used by the `#[koruma::validator]` attribute macro to find which
/// field should receive the value being validated.
pub fn find_value_field(input: &ItemStruct) -> Option<(Ident, Type)> {
    find_value_field_strict(input).ok().flatten()
}

/// Find the field marked with `#[koruma(value)]` and return its parsed info.
pub fn find_value_field_info(input: &ItemStruct) -> Option<ValueFieldInfo> {
    find_value_field_info_strict(input).ok().flatten()
}

/// Parsed showcase attribute:
/// `#[showcase(name = "...", description = "...", create = |input| { ... }, input_type = Text)]`
///
/// The `create` closure takes a `&str` and returns the validator instance.
/// Required `input_type` must be `Text` or `Numeric`.
/// Optional `module` can be "string", "format", "numeric", "collection", or "general".
#[cfg(feature = "internal-showcase")]
#[derive(Clone, Debug)]
pub struct ShowcaseAttr {
    pub name: syn::LitStr,
    pub description: syn::LitStr,
    pub create: syn::ExprClosure,
    pub input_type: Ident,
    pub module: Option<syn::LitStr>,
}

#[cfg(feature = "internal-showcase")]
impl Parse for ShowcaseAttr {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut name: Option<syn::LitStr> = None;
        let mut description: Option<syn::LitStr> = None;
        let mut create: Option<syn::ExprClosure> = None;
        let mut input_type: Option<Ident> = None;
        let mut module: Option<syn::LitStr> = None;

        while !input.is_empty() {
            let ident: Ident = input.parse()?;
            input.parse::<Token![=]>()?;

            match ident.to_string().as_str() {
                "name" => {
                    name = Some(input.parse()?);
                },
                "description" => {
                    description = Some(input.parse()?);
                },
                "create" => {
                    create = Some(input.parse()?);
                },
                "input_type" => {
                    input_type = Some(input.parse()?);
                },
                "module" => {
                    module = Some(input.parse()?);
                },
                other => {
                    return Err(Error::new(
                        ident.span(),
                        format!("unknown showcase attribute: {}", other),
                    ));
                },
            }

            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(ShowcaseAttr {
            name: name
                .ok_or_else(|| Error::new(input.span(), "showcase requires `name` attribute"))?,
            description: description.ok_or_else(|| {
                Error::new(input.span(), "showcase requires `description` attribute")
            })?,
            create: create
                .ok_or_else(|| Error::new(input.span(), "showcase requires `create` attribute"))?,
            input_type: match input_type {
                Some(input_type)
                    if matches!(input_type.to_string().as_str(), "Text" | "Numeric") =>
                {
                    input_type
                },
                Some(input_type) => {
                    return Err(Error::new(
                        input_type.span(),
                        "showcase `input_type` must be `Text` or `Numeric`",
                    ));
                },
                None => {
                    return Err(Error::new(
                        input.span(),
                        "showcase requires `input_type` attribute",
                    ));
                },
            },
            module,
        })
    }
}

/// Find and parse showcase attribute from struct
#[cfg(feature = "internal-showcase")]
pub fn find_showcase_attr(input: &ItemStruct) -> Result<Option<ShowcaseAttr>> {
    for attr in &input.attrs {
        if attr.path().is_ident("showcase") {
            return Ok(Some(attr.parse_args::<ShowcaseAttr>()?));
        }
    }
    Ok(None)
}
