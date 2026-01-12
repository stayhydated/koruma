use syn::{
    Attribute, Error, Expr, Field, Fields, Ident, ItemStruct, Path, Result, Token, Type,
    parenthesized,
    parse::{Parse, ParseStream},
    spanned::Spanned,
    token,
};

/// Represents a single parsed validator: `ValidatorName(arg = value, ...)` or
/// `ValidatorName::<_>(arg = value, ...)` or `ValidatorName::<SomeType>(arg = value, ...)`
/// Also supports fully-qualified paths like `module::path::ValidatorName::<_>`.
///
/// Uses turbofish syntax (`::<>`) for type parameters, which simplifies parsing
/// and naturally handles nested generics like `Validator::<Option<Vec<T>>>`.
pub(crate) struct ValidatorAttr {
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
/// Can also include `each` modifier for collection validation:
/// `#[koruma(VecValidator(min = 0), each(ElementValidator(max = 100)))]`
/// Can also include `nested` to validate nested structs that also derive Koruma.
/// Can also include `newtype` to validate a newtype wrapper with transparent error access.
pub(crate) struct KorumaAttr {
    /// Validators applied to the field/collection itself
    pub field_validators: Vec<ValidatorAttr>,
    /// Validators applied to each element in a collection (from `each(...)`)
    pub element_validators: Vec<ValidatorAttr>,
    pub is_skip: bool,
    /// Whether this field is a nested Koruma struct
    pub is_nested: bool,
    /// Whether this field is a newtype wrapper (single-field struct deriving Koruma).
    /// Similar to nested, but generates a wrapper error struct with Deref for transparent access.
    pub is_newtype: bool,
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
#[derive(Default)]
pub(crate) struct StructOptions {
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

/// Parse struct-level `#[koruma(...)]` attributes from a list of attributes
pub(crate) fn parse_struct_options(attrs: &[Attribute]) -> Result<StructOptions> {
    for attr in attrs {
        if attr.path().is_ident("koruma") {
            return attr.parse_args::<StructOptions>();
        }
    }
    Ok(StructOptions::default())
}

/// Field info extracted from the struct
pub(crate) struct FieldInfo {
    pub name: Ident,
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

    /// Returns true if this field is a nested Koruma struct
    pub fn is_nested(&self) -> bool {
        self.is_nested
    }

    /// Returns true if this field is a newtype wrapper
    pub fn is_newtype(&self) -> bool {
        self.is_newtype
    }
}

/// Result of parsing a field with #[koruma(...)] attribute
#[allow(clippy::large_enum_variant)]
pub(crate) enum ParseFieldResult {
    /// Field has valid koruma validators
    Valid(FieldInfo),
    /// Field should be skipped (no koruma attribute, or #[koruma(skip)])
    Skip,
    /// Parse error occurred
    Error(Error),
}

pub(crate) fn parse_field(field: &Field) -> ParseFieldResult {
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
        if !attr.path().is_ident("koruma") {
            continue;
        }

        // Parse the attribute content
        let parsed: Result<KorumaAttr> = attr.parse_args();

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

/// Find the field marked with #[koruma(value)] and return its name and type
pub(crate) fn find_value_field(input: &ItemStruct) -> Option<(Ident, Type)> {
    if let Fields::Named(ref fields) = input.fields {
        for field in &fields.named {
            for attr in &field.attrs {
                if attr.path().is_ident("koruma") {
                    // Try to parse as just "value"
                    if let Ok(ident) = attr.parse_args::<Ident>()
                        && ident == "value"
                    {
                        return Some((field.ident.clone().unwrap(), field.ty.clone()));
                    }
                }
            }
        }
    }
    None
}

/// Parsed showcase attribute: `#[showcase(name = "...", description = "...", create = |input| { ... })]`
/// The `create` closure takes a `&str` and returns the validator instance.
/// Optional `input_type` can be "text" (default) or "numeric".
#[cfg(feature = "showcase")]
pub(crate) struct ShowcaseAttr {
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
pub(crate) fn find_showcase_attr(input: &ItemStruct) -> Option<ShowcaseAttr> {
    for attr in &input.attrs {
        if attr.path().is_ident("showcase")
            && let Ok(parsed) = attr.parse_args::<ShowcaseAttr>()
        {
            return Some(parsed);
        }
    }
    None
}
