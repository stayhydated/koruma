//! Parsing logic for `#[koruma(...)]` attributes.
//!
//! This module provides types and functions for parsing koruma validation
//! attributes from syn AST nodes.

use syn::{
    Attribute, Error, Expr, Field, Fields, Ident, ItemStruct, Meta, Path, Result, Token, Type,
    parenthesized,
    parse::{Parse, ParseStream},
    spanned::Spanned,
    token,
};

/// Check if an attribute is a `koruma` attribute or a `cfg_attr` containing a `koruma` attribute.
/// Returns `Some(true)` if it's a direct koruma attribute, `Some(false)` if it's a cfg_attr with koruma,
/// and `None` if it's neither.
fn is_koruma_attr(attr: &Attribute) -> Option<bool> {
    if attr.path().is_ident("koruma") {
        return Some(true);
    }

    // Check for cfg_attr(condition, koruma(...))
    if attr.path().is_ident("cfg_attr") {
        if let Meta::List(meta_list) = &attr.meta {
            // Parse the tokens inside cfg_attr(...)
            // Format: cfg_attr(condition, attr1, attr2, ...)
            let tokens = meta_list.tokens.clone();
            let mut iter = tokens.into_iter().peekable();

            // Skip the condition (everything until we hit a comma at depth 0)
            let mut depth: i32 = 0;
            loop {
                match iter.next() {
                    Some(proc_macro2::TokenTree::Punct(p)) if p.as_char() == ',' && depth == 0 => {
                        break;
                    },
                    Some(proc_macro2::TokenTree::Group(_)) => {
                        // Groups don't affect our comma detection at top level
                    },
                    Some(proc_macro2::TokenTree::Punct(p)) if p.as_char() == '<' => depth += 1,
                    Some(proc_macro2::TokenTree::Punct(p)) if p.as_char() == '>' => {
                        depth = depth.saturating_sub(1);
                    },
                    None => return None,
                    _ => {},
                }
            }

            // Now check if any of the remaining attributes is "koruma"
            while let Some(token) = iter.next() {
                if let proc_macro2::TokenTree::Ident(ident) = token {
                    if ident == "koruma" {
                        return Some(false);
                    }
                }
                // Skip to next attribute (after comma) if we hit a group
                if let Some(proc_macro2::TokenTree::Punct(p)) = iter.peek() {
                    if p.as_char() == ',' {
                        iter.next();
                    }
                }
            }
        }
    }

    None
}

/// Extract the koruma attribute content from a cfg_attr.
/// For `#[cfg_attr(condition, koruma(content))]`, returns a parsed KorumaAttr.
fn parse_koruma_from_cfg_attr(attr: &Attribute) -> Result<KorumaAttr> {
    if let Meta::List(meta_list) = &attr.meta {
        let tokens = meta_list.tokens.clone();
        let mut iter = tokens.into_iter().peekable();

        // Skip the condition
        let mut depth: i32 = 0;
        loop {
            match iter.next() {
                Some(proc_macro2::TokenTree::Punct(p)) if p.as_char() == ',' && depth == 0 => {
                    break;
                },
                Some(proc_macro2::TokenTree::Punct(p)) if p.as_char() == '<' => depth += 1,
                Some(proc_macro2::TokenTree::Punct(p)) if p.as_char() == '>' => {
                    depth = depth.saturating_sub(1);
                },
                None => {
                    return Err(Error::new_spanned(
                        attr,
                        "invalid cfg_attr: missing attribute after condition",
                    ));
                },
                _ => {},
            }
        }

        // Find koruma(...) and extract its content
        while let Some(token) = iter.next() {
            if let proc_macro2::TokenTree::Ident(ident) = token {
                if ident == "koruma" {
                    // Next should be a group containing the koruma content
                    if let Some(proc_macro2::TokenTree::Group(group)) = iter.next() {
                        return syn::parse2::<KorumaAttr>(group.stream());
                    } else {
                        // koruma without parens - this is valid for things like just "koruma"
                        // but we need to check what follows
                        return Ok(KorumaAttr::default());
                    }
                }
            }
        }

        Err(Error::new_spanned(
            attr,
            "cfg_attr does not contain a koruma attribute",
        ))
    } else {
        Err(Error::new_spanned(attr, "invalid cfg_attr format"))
    }
}

/// Represents a single parsed validator: `ValidatorName(arg = value, ...)` or
/// `ValidatorName::<_>(arg = value, ...)` or `ValidatorName::<SomeType>(arg = value, ...)`
/// Also supports fully-qualified paths like `module::path::ValidatorName::<_>`.
///
/// Uses turbofish syntax (`::<>`) for type parameters, which simplifies parsing
/// and naturally handles nested generics like `Validator::<Option<Vec<T>>>`.
///
/// # Examples
///
/// ```ignore
/// // Simple validator
/// #[koruma(NonEmptyValidation)]
///
/// // Validator with type inference
/// #[koruma(RangeValidation::<_>(min = 0, max = 100))]
///
/// // Validator with explicit type
/// #[koruma(RangeValidation::<i32>(min = 0, max = 100))]
///
/// // Full path
/// #[koruma(validators::numeric::RangeValidation::<_>(min = 0))]
/// ```
#[derive(Clone, Debug)]
pub struct ValidatorAttr {
    /// The validator path, which may be a simple identifier or a full path.
    /// Examples: `StringLengthValidation`, `validators::normal::NumberRangeValidation`
    pub validator: Path,
    /// Whether the validator uses `::<_>` syntax for type inference from field type.
    /// When true, the field type is used (unwrapping Option if present).
    pub infer_type: bool,
    /// Explicit type parameter if specified (e.g., `::<f64>`, `::<Vec<_>>`)
    /// If this contains `_`, it will be substituted with the inner type from the field.
    /// Use `::<Option<_>>` to get the full Option type without unwrapping.
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

    /// Returns whether this validator has any arguments.
    pub fn has_args(&self) -> bool {
        !self.args.is_empty()
    }

    /// Returns whether this validator uses type inference (`::<_>` syntax).
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
        // Parse the path manually, segment by segment.
        // We need to stop BEFORE consuming any turbofish generics (`::<...>`)
        // because we want to handle those separately for our ::<_> syntax.

        // Check for leading ::
        let leading_colon = if input.peek(Token![::]) {
            Some(input.parse::<Token![::]>()?)
        } else {
            None
        };

        // Parse path segments manually
        let mut segments = syn::punctuated::Punctuated::new();

        loop {
            // Parse an identifier (path segment)
            let ident: Ident = input.parse()?;
            segments.push(syn::PathSegment {
                ident,
                arguments: syn::PathArguments::None,
            });

            // Check what follows:
            // - `::` followed by `<` = turbofish, stop here
            // - `::` followed by ident = more path segments, continue
            // - anything else = end of path
            if input.peek(Token![::]) {
                let fork = input.fork();
                fork.parse::<Token![::]>().ok();

                if fork.peek(Token![<]) {
                    // This is a turbofish, don't consume the ::, let the turbofish handling below do it
                    break;
                } else if fork.peek(Ident) {
                    // More path segments, consume :: and continue
                    segments.push_punct(input.parse()?);
                } else {
                    // :: followed by something else, stop
                    break;
                }
            } else {
                // Not ::, end of path
                break;
            }
        }

        let validator = Path {
            leading_colon,
            segments,
        };

        // Check for turbofish generic syntax: ::<_> or ::<SomeType>
        // ::<_> means "use the field type" (unwrapping Option if present)
        // ::<Option<_>> means "use the full Option type" (without unwrapping)
        // ::<Vec<_>> means "substitute _ with the inner type from the field"
        let (infer_type, explicit_type) = if input.peek(Token![::]) {
            // Look ahead to check if < follows ::
            let fork = input.fork();
            let has_turbofish = fork.parse::<Token![::]>().is_ok() && fork.peek(Token![<]);

            if has_turbofish {
                input.parse::<Token![::]>()?;
                input.parse::<Token![<]>()?;

                // Check for ::<_> syntax (type inference with Option unwrapping)
                if input.peek(Token![_]) {
                    input.parse::<Token![_]>()?;
                    input.parse::<Token![>]>()?;
                    (true, None)
                }
                // Explicit type: ::<SomeType>
                else {
                    let ty: Type = input.parse()?;
                    input.parse::<Token![>]>()?;
                    (false, Some(ty))
                }
            } else {
                (false, None)
            }
        } else if input.peek(Token![<]) {
            // User used old syntax without ::, give helpful error
            return Err(Error::new(
                input.span(),
                "use turbofish syntax for type parameters: `Validator::<_>` not `Validator<_>`",
            ));
        } else {
            (false, None)
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
            // Check for newtype
            if ident == "newtype" && fork.is_empty() {
                input.parse::<Ident>()?; // consume "newtype"
                return Ok(KorumaAttr {
                    field_validators: Vec::new(),
                    element_validators: Vec::new(),
                    is_skip: false,
                    is_nested: false,
                    is_newtype: true,
                });
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
/// ```
#[derive(Clone, Debug, Default)]
pub struct StructOptions {
    /// Generate a `try_new` function that validates on construction
    pub try_new: bool,
    /// Treat this struct as a newtype (single-field wrapper).
    /// Generates an `.all()` method on the error struct that aggregates
    /// all validators from the single field.
    pub newtype: bool,
}

impl Parse for StructOptions {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut options = StructOptions::default();

        while !input.is_empty() {
            let ident: Ident = input.parse()?;
            match ident.to_string().as_str() {
                "try_new" => options.try_new = true,
                "newtype" => options.newtype = true,
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
    for attr in attrs {
        // Check if this is a koruma attribute (direct or via cfg_attr)
        match is_koruma_attr(attr) {
            Some(true) => {
                // Direct #[koruma(...)]
                return attr.parse_args::<StructOptions>();
            },
            Some(false) => {
                // cfg_attr containing koruma - parse struct options from it
                return parse_struct_options_from_cfg_attr(attr);
            },
            None => continue,
        }
    }
    Ok(StructOptions::default())
}

/// Extract struct options from a cfg_attr containing koruma.
fn parse_struct_options_from_cfg_attr(attr: &Attribute) -> Result<StructOptions> {
    if let Meta::List(meta_list) = &attr.meta {
        let tokens = meta_list.tokens.clone();
        let mut iter = tokens.into_iter().peekable();

        // Skip the condition
        let mut depth: i32 = 0;
        loop {
            match iter.next() {
                Some(proc_macro2::TokenTree::Punct(p)) if p.as_char() == ',' && depth == 0 => {
                    break;
                },
                Some(proc_macro2::TokenTree::Punct(p)) if p.as_char() == '<' => depth += 1,
                Some(proc_macro2::TokenTree::Punct(p)) if p.as_char() == '>' => {
                    depth = depth.saturating_sub(1);
                },
                None => {
                    return Err(Error::new_spanned(
                        attr,
                        "invalid cfg_attr: missing attribute after condition",
                    ));
                },
                _ => {},
            }
        }

        // Find koruma(...) and extract its content
        while let Some(token) = iter.next() {
            if let proc_macro2::TokenTree::Ident(ident) = token {
                if ident == "koruma" {
                    // Next should be a group containing the koruma content
                    if let Some(proc_macro2::TokenTree::Group(group)) = iter.next() {
                        return syn::parse2::<StructOptions>(group.stream());
                    } else {
                        // koruma without parens - return default
                        return Ok(StructOptions::default());
                    }
                }
            }
        }

        Err(Error::new_spanned(
            attr,
            "cfg_attr does not contain a koruma attribute",
        ))
    } else {
        Err(Error::new_spanned(attr, "invalid cfg_attr format"))
    }
}

/// Field information extracted from parsing `#[koruma(...)]` attributes.
///
/// This struct contains all the parsed validation information for a single field,
/// including validators, element validators (for collections), and modifier flags.
#[derive(Clone, Debug)]
pub struct FieldInfo {
    /// The field name
    pub name: Ident,
    /// The field type
    pub ty: Type,
    /// Validators for the field/collection itself
    pub field_validators: Vec<ValidatorAttr>,
    /// Validators for each element in a collection
    pub element_validators: Vec<ValidatorAttr>,
    /// Whether this field is a nested Koruma struct
    pub is_nested: bool,
    /// Whether this field is a newtype wrapper
    pub is_newtype: bool,
}

impl FieldInfo {
    /// Returns true if this field has element validators (uses `each(...)`)
    pub fn has_element_validators(&self) -> bool {
        !self.element_validators.is_empty()
    }

    /// Returns true if this field has any validators (field or element)
    pub fn has_validators(&self) -> bool {
        !self.field_validators.is_empty() || !self.element_validators.is_empty()
    }

    /// Returns true if this field is a nested Koruma struct
    pub fn is_nested(&self) -> bool {
        self.is_nested
    }

    /// Returns true if this field is a newtype wrapper
    pub fn is_newtype(&self) -> bool {
        self.is_newtype
    }

    /// Returns an iterator over all validator names on this field.
    pub fn validator_names(&self) -> impl Iterator<Item = &Ident> {
        self.field_validators
            .iter()
            .chain(self.element_validators.iter())
            .map(|v| v.name())
    }
}

/// Result of parsing a field with `#[koruma(...)]` attribute.
///
/// This enum represents the three possible outcomes of parsing a field:
/// - `Valid`: The field has valid koruma validators
/// - `Skip`: The field should be skipped (no koruma attribute, or `#[koruma(skip)]`)
/// - `Error`: A parse error occurred
#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
pub enum ParseFieldResult {
    /// Field has valid koruma validators
    Valid(FieldInfo),
    /// Field should be skipped (no koruma attribute, or #[koruma(skip)])
    Skip,
    /// Parse error occurred
    Error(Error),
}

impl ParseFieldResult {
    /// Returns the field info if this is a `Valid` result.
    pub fn valid(self) -> Option<FieldInfo> {
        match self {
            ParseFieldResult::Valid(info) => Some(info),
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
pub fn parse_field(field: &Field) -> ParseFieldResult {
    let Some(name) = field.ident.clone() else {
        return ParseFieldResult::Skip;
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

    for attr in &field.attrs {
        // Check if this is a koruma attribute (direct or via cfg_attr)
        let is_direct = match is_koruma_attr(attr) {
            Some(true) => true,   // Direct #[koruma(...)]
            Some(false) => false, // cfg_attr containing koruma
            None => continue,     // Not a koruma attribute
        };

        // Parse the attribute content
        let parsed: Result<KorumaAttr> = if is_direct {
            attr.parse_args()
        } else {
            parse_koruma_from_cfg_attr(attr)
        };

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
                    continue;
                }
                // Collect validators from this attribute, checking for duplicates
                for validator in koruma_attr.field_validators {
                    let validator_name = validator.name().to_string();
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
                    let validator_name = validator.name().to_string();
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

    // Check for nested
    if is_nested {
        return ParseFieldResult::Valid(FieldInfo {
            name,
            ty,
            field_validators: all_field_validators,
            element_validators: all_element_validators,
            is_nested: true,
            is_newtype: false,
        });
    }

    // Check for newtype
    if is_newtype {
        return ParseFieldResult::Valid(FieldInfo {
            name,
            ty,
            field_validators: all_field_validators,
            element_validators: all_element_validators,
            is_nested: false,
            is_newtype: true,
        });
    }

    // Must have at least one validator or modifier
    if all_field_validators.is_empty() && all_element_validators.is_empty() {
        return ParseFieldResult::Skip;
    }

    ParseFieldResult::Valid(FieldInfo {
        name,
        ty,
        field_validators: all_field_validators,
        element_validators: all_element_validators,
        is_nested: false,
        is_newtype: false,
    })
}

/// Find the field marked with `#[koruma(value)]` and return its name and type.
///
/// This is used by the `#[koruma::validator]` attribute macro to find which
/// field should receive the value being validated.
pub fn find_value_field(input: &ItemStruct) -> Option<(Ident, Type)> {
    if let Fields::Named(ref fields) = input.fields {
        for field in &fields.named {
            for attr in &field.attrs {
                // Check if this is a koruma attribute (direct or via cfg_attr)
                match is_koruma_attr(attr) {
                    Some(true) => {
                        // Direct #[koruma(...)] - try to parse as just "value"
                        if let Ok(ident) = attr.parse_args::<Ident>()
                            && ident == "value"
                        {
                            return Some((field.ident.clone().unwrap(), field.ty.clone()));
                        }
                    },
                    Some(false) => {
                        // cfg_attr containing koruma - check if it's "value"
                        if is_koruma_value_in_cfg_attr(attr) {
                            return Some((field.ident.clone().unwrap(), field.ty.clone()));
                        }
                    },
                    None => continue,
                }
            }
        }
    }
    None
}

/// Check if a cfg_attr contains `koruma(value)`.
fn is_koruma_value_in_cfg_attr(attr: &Attribute) -> bool {
    if let Meta::List(meta_list) = &attr.meta {
        let tokens = meta_list.tokens.clone();
        let mut iter = tokens.into_iter().peekable();

        // Skip the condition
        let mut depth: i32 = 0;
        loop {
            match iter.next() {
                Some(proc_macro2::TokenTree::Punct(p)) if p.as_char() == ',' && depth == 0 => {
                    break;
                },
                Some(proc_macro2::TokenTree::Punct(p)) if p.as_char() == '<' => depth += 1,
                Some(proc_macro2::TokenTree::Punct(p)) if p.as_char() == '>' => {
                    depth = depth.saturating_sub(1);
                },
                None => return false,
                _ => {},
            }
        }

        // Find koruma(...) and check if it contains "value"
        while let Some(token) = iter.next() {
            if let proc_macro2::TokenTree::Ident(ident) = token {
                if ident == "koruma" {
                    // Next should be a group containing the koruma content
                    if let Some(proc_macro2::TokenTree::Group(group)) = iter.next() {
                        // Check if the content is just "value"
                        if let Ok(ident) = syn::parse2::<Ident>(group.stream()) {
                            return ident == "value";
                        }
                    }
                    return false;
                }
            }
        }
    }
    false
}

/// Parsed showcase attribute: `#[showcase(name = "...", description = "...", create = |input| { ... })]`
///
/// The `create` closure takes a `&str` and returns the validator instance.
/// Optional `input_type` can be "text" (default) or "numeric".
#[cfg(feature = "showcase")]
#[derive(Clone, Debug)]
pub struct ShowcaseAttr {
    pub name: syn::LitStr,
    pub description: syn::LitStr,
    pub create: syn::ExprClosure,
    pub input_type: Option<Ident>,
}

#[cfg(feature = "showcase")]
impl Parse for ShowcaseAttr {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut name: Option<syn::LitStr> = None;
        let mut description: Option<syn::LitStr> = None;
        let mut create: Option<syn::ExprClosure> = None;
        let mut input_type: Option<Ident> = None;

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
            input_type,
        })
    }
}

/// Find and parse showcase attribute from struct
#[cfg(feature = "showcase")]
pub fn find_showcase_attr(input: &ItemStruct) -> Option<ShowcaseAttr> {
    for attr in &input.attrs {
        if attr.path().is_ident("showcase")
            && let Ok(parsed) = attr.parse_args::<ShowcaseAttr>()
        {
            return Some(parsed);
        }
    }
    None
}
