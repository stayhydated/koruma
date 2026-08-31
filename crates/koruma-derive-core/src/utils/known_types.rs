use super::*;

/// Syntactic shape of a Rust type as recognized by koruma derive helpers.
///
/// This is intentionally syntax-only macro recognition. It does not resolve
/// type aliases or trait implementations.
#[derive(Clone, Copy, Debug)]
pub enum KnownTypeShape<'a> {
    Option {
        segment: &'a PathSegment,
        inner: &'a Type,
    },
    Vec {
        segment: &'a PathSegment,
        inner: &'a Type,
    },
    Slice {
        span: Span,
        inner: &'a Type,
    },
    Array {
        span: Span,
        inner: &'a Type,
    },
    Reference {
        span: Span,
        inner: &'a Type,
    },
    Other(&'a Type),
}

impl<'a> KnownTypeShape<'a> {
    pub fn of(ty: &'a Type) -> Self {
        match ty {
            Type::Group(group) => Self::of(&group.elem),
            Type::Paren(paren) => Self::of(&paren.elem),
            Type::Array(array) => Self::Array {
                span: array.span(),
                inner: &array.elem,
            },
            Type::Reference(reference) => Self::Reference {
                span: reference.span(),
                inner: &reference.elem,
            },
            Type::Slice(slice) => Self::Slice {
                span: slice.span(),
                inner: &slice.elem,
            },
            Type::Path(type_path) => {
                if let Some((segment, inner)) = path_last_generic_type(&type_path.path, "Option") {
                    Self::Option { segment, inner }
                } else if let Some((segment, inner)) =
                    path_last_generic_type(&type_path.path, "Vec")
                {
                    Self::Vec { segment, inner }
                } else {
                    Self::Other(ty)
                }
            },
            _ => Self::Other(ty),
        }
    }

    pub fn recognized_name(&self) -> Option<&Ident> {
        match self {
            Self::Option { segment, .. } | Self::Vec { segment, .. } => Some(&segment.ident),
            Self::Slice { .. } | Self::Array { .. } | Self::Reference { .. } | Self::Other(_) => {
                None
            },
        }
    }

    pub fn span(&self) -> Span {
        match self {
            Self::Option { segment, .. } | Self::Vec { segment, .. } => segment.span(),
            Self::Slice { span, .. } | Self::Array { span, .. } | Self::Reference { span, .. } => {
                *span
            },
            Self::Other(ty) => ty.span(),
        }
    }
}

fn path_last_generic_type<'a>(path: &'a Path, ident: &str) -> Option<(&'a PathSegment, &'a Type)> {
    let segment = path.segments.last()?;
    if segment.ident != ident {
        return None;
    }

    let PathArguments::AngleBracketed(args) = &segment.arguments else {
        return None;
    };

    match args.args.first()? {
        GenericArgument::Type(inner) => Some((segment, inner)),
        _ => None,
    }
}

/// Extract the inner type T from `Option<T>`.
///
/// Returns `None` if the type is not an `Option`.
///
/// # Examples
///
/// ```rust
/// use syn::parse_quote;
/// use koruma_derive_core::option_inner_type;
/// use syn::Type;
///
/// let ty: Type = parse_quote!(Option<String>);
/// let inner = option_inner_type(&ty);
/// let expected: Type = parse_quote!(String);
/// assert_eq!(inner, Some(&expected));
///
/// let ty2: Type = parse_quote!(String);
/// let inner2 = option_inner_type(&ty2);
/// assert!(inner2.is_none());
/// ```
pub fn option_inner_type(ty: &Type) -> Option<&Type> {
    match KnownTypeShape::of(ty) {
        KnownTypeShape::Option { inner, .. } => Some(inner),
        _ => None,
    }
}

/// Extract the inner type T from `Vec<T>`.
///
/// Returns `None` if the type is not a `Vec`.
///
/// # Examples
///
/// ```rust
/// use syn::parse_quote;
/// use koruma_derive_core::vec_inner_type;
/// use syn::Type;
///
/// let ty: Type = parse_quote!(Vec<String>);
/// let inner = vec_inner_type(&ty);
/// let expected: Type = parse_quote!(String);
/// assert_eq!(inner, Some(&expected));
///
/// let ty2: Type = parse_quote!(String);
/// let inner2 = vec_inner_type(&ty2);
/// assert!(inner2.is_none());
/// ```
pub fn vec_inner_type(ty: &Type) -> Option<&Type> {
    match KnownTypeShape::of(ty) {
        KnownTypeShape::Vec { inner, .. } => Some(inner),
        _ => None,
    }
}

/// Check if a field type is `Option<T>`.
///
/// # Examples
///
/// ```rust
/// use syn::parse_quote;
/// use koruma_derive_core::is_option_type;
/// use syn::Type;
///
/// let ty1: Type = parse_quote!(Option<String>);
/// assert!(is_option_type(&ty1));
///
/// let ty2: Type = parse_quote!(String);
/// assert!(!is_option_type(&ty2));
/// ```
pub fn is_option_type(ty: &Type) -> bool {
    option_inner_type(ty).is_some()
}

/// Extract the ident (name) from a type path.
///
/// Returns `None` if the type is not a simple path type.
///
/// # Examples
///
/// ```rust
/// use syn::parse_quote;
/// use koruma_derive_core::type_to_ident;
/// use syn::Type;
///
/// let ty: Type = parse_quote!(Age);
/// assert_eq!(type_to_ident(&ty).map(|ident| ident.to_string()), Some("Age".to_string()));
///
/// let ty2: Type = parse_quote!(Option<Age>);
/// assert_eq!(type_to_ident(&ty2).map(|ident| ident.to_string()), Some("Option".to_string()));
/// ```
pub fn type_to_ident(ty: &Type) -> Option<Ident> {
    match ty {
        Type::Path(type_path) => type_path.path.segments.last().map(|s| s.ident.clone()),
        _ => None,
    }
}
