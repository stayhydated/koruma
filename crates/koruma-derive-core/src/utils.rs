//! Utility functions for type manipulation and analysis.
//!
//! These utilities are used for working with syn types, particularly
//! for handling `Option<T>`, `Vec<T>`, and type inference placeholders.

use syn::{Expr, GenericArgument, Ident, PathArguments, Type};

/// Substitute infer placeholders (`_`) in a type with the actual inferred type.
///
/// For example, `Vec<_>` with `infer_ty=String` becomes `Vec<String>`.
///
/// # Examples
///
/// ```rust
/// use syn::parse_quote;
/// use koruma_derive_core::substitute_infer_type;
/// use syn::Type;
///
/// let ty: Type = parse_quote!(Vec<_>);
/// let infer_ty: Type = parse_quote!(String);
/// let result = substitute_infer_type(&ty, &infer_ty);
/// let expected: Type = parse_quote!(Vec<String>);
/// assert_eq!(result, expected);
/// ```
pub fn substitute_infer_type(ty: &Type, infer_ty: &Type) -> Type {
    match ty {
        Type::Infer(_) => infer_ty.clone(),
        Type::Path(type_path) => {
            let mut new_path = type_path.clone();
            for segment in &mut new_path.path.segments {
                if let PathArguments::AngleBracketed(args) = &mut segment.arguments {
                    for arg in &mut args.args {
                        if let GenericArgument::Type(inner_ty) = arg {
                            *inner_ty = substitute_infer_type(inner_ty, infer_ty);
                        }
                    }
                }
            }
            Type::Path(new_path)
        },
        _ => ty.clone(),
    }
}

fn type_generic_args(ty: &Type) -> Vec<&Type> {
    let Type::Path(type_path) = ty else {
        return Vec::new();
    };
    let Some(segment) = type_path.path.segments.last() else {
        return Vec::new();
    };

    let PathArguments::AngleBracketed(args) = &segment.arguments else {
        return Vec::new();
    };

    args.args
        .iter()
        .filter_map(|arg| match arg {
            GenericArgument::Type(ty) => Some(ty),
            _ => None,
        })
        .collect()
}

fn matching_source_type_args<'a>(explicit_ty: &Type, source_ty: &'a Type) -> Option<Vec<&'a Type>> {
    let Type::Path(explicit_path) = explicit_ty else {
        return None;
    };
    let Type::Path(source_path) = source_ty else {
        return None;
    };

    let explicit_segment = explicit_path.path.segments.last()?;
    let source_segment = source_path.path.segments.last()?;

    if explicit_segment.ident != source_segment.ident {
        return None;
    }

    let explicit_args = type_generic_args(explicit_ty);
    let source_args = type_generic_args(source_ty);

    if explicit_args.len() != source_args.len() {
        return None;
    }

    Some(source_args)
}

fn substitute_infer_type_from_source_inner(ty: &Type, source_ty: &Type) -> Option<Type> {
    match ty {
        Type::Infer(_) => Some(source_ty.clone()),
        Type::Path(type_path) => {
            let explicit_args = type_generic_args(ty);
            let mut new_path = type_path.clone();
            let structural_source_args = matching_source_type_args(ty, source_ty);

            let fallback_source =
                match (explicit_args.len(), type_generic_args(source_ty).as_slice()) {
                    (1, [only]) => Some(*only),
                    (1, []) => Some(source_ty),
                    _ => None,
                };

            let mut source_index = 0usize;
            for segment in &mut new_path.path.segments {
                if let PathArguments::AngleBracketed(args) = &mut segment.arguments {
                    for arg in &mut args.args {
                        if let GenericArgument::Type(inner_ty) = arg {
                            let child_source = structural_source_args
                                .as_ref()
                                .and_then(|args| args.get(source_index).copied())
                                .or(fallback_source)?;
                            source_index += 1;
                            *inner_ty =
                                substitute_infer_type_from_source_inner(inner_ty, child_source)?;
                        }
                    }
                }
            }
            Some(Type::Path(new_path))
        },
        _ => Some(ty.clone()),
    }
}

/// Substitute infer placeholders (`_`) in a type using the structure of a source type.
///
/// This preserves multi-generic inference when the explicit type shape matches the source type,
/// while still supporting wrapper substitutions like `Vec<_>` from `Option<T>`.
///
/// # Examples
///
/// ```rust
/// use syn::parse_quote;
/// use koruma_derive_core::substitute_infer_type_from_source;
/// use syn::Type;
///
/// let explicit: Type = parse_quote!(std::collections::HashMap<_, _>);
/// let source: Type = parse_quote!(std::collections::HashMap<String, i32>);
/// let result = substitute_infer_type_from_source(&explicit, &source).unwrap();
/// let expected: Type = parse_quote!(std::collections::HashMap<String, i32>);
/// assert_eq!(result, expected);
/// ```
pub fn substitute_infer_type_from_source(ty: &Type, source_ty: &Type) -> Option<Type> {
    substitute_infer_type_from_source_inner(ty, source_ty)
}

/// Extract the first generic type argument from a type.
///
/// For example, `Vec<String>` → `String`, `HashSet<i32>` → `i32`.
///
/// # Examples
///
/// ```rust
/// use syn::parse_quote;
/// use koruma_derive_core::first_generic_arg;
/// use syn::Type;
///
/// let ty: Type = parse_quote!(Vec<String>);
/// let inner = first_generic_arg(&ty);
/// let expected: Type = parse_quote!(String);
/// assert_eq!(inner, Some(&expected));
/// ```
pub fn first_generic_arg(ty: &Type) -> Option<&Type> {
    if let Type::Path(type_path) = ty
        && let Some(segment) = type_path.path.segments.last()
        && let PathArguments::AngleBracketed(args) = &segment.arguments
    {
        for arg in &args.args {
            if let GenericArgument::Type(inner_ty) = arg {
                return Some(inner_ty);
            }
        }
    }
    None
}

/// Check if a type contains any infer placeholders (`_`).
///
/// # Examples
///
/// ```rust
/// use syn::parse_quote;
/// use koruma_derive_core::contains_infer_type;
/// use syn::Type;
///
/// let ty1: Type = parse_quote!(Vec<_>);
/// assert!(contains_infer_type(&ty1));
///
/// let ty2: Type = parse_quote!(Vec<String>);
/// assert!(!contains_infer_type(&ty2));
/// ```
pub fn contains_infer_type(ty: &Type) -> bool {
    match ty {
        Type::Infer(_) => true,
        Type::Path(type_path) => {
            for segment in &type_path.path.segments {
                if let PathArguments::AngleBracketed(args) = &segment.arguments {
                    for arg in &args.args {
                        if let GenericArgument::Type(inner_ty) = arg
                            && contains_infer_type(inner_ty)
                        {
                            return true;
                        }
                    }
                }
            }
            false
        },
        _ => false,
    }
}

/// Check if a type is `Option<_>` (Option wrapping an infer placeholder).
///
/// This is used when a validator explicitly wants the full Option type,
/// like `RequiredValidation::<Option<_>>::builder()`.
///
/// # Examples
///
/// ```rust
/// use syn::parse_quote;
/// use koruma_derive_core::is_option_infer_type;
/// use syn::Type;
///
/// let ty1: Type = parse_quote!(Option<_>);
/// assert!(is_option_infer_type(&ty1));
///
/// let ty2: Type = parse_quote!(Option<String>);
/// assert!(!is_option_infer_type(&ty2));
/// ```
pub fn is_option_infer_type(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty
        && let Some(segment) = type_path.path.segments.last()
        && segment.ident == "Option"
        && let PathArguments::AngleBracketed(args) = &segment.arguments
    {
        for arg in &args.args {
            if let GenericArgument::Type(inner_ty) = arg {
                return matches!(inner_ty, Type::Infer(_));
            }
        }
    }
    false
}

/// Check if an expression is a simple identifier (bare field name like `password`).
///
/// If so, return the identifier. This is used to detect field references in validator args.
///
/// # Examples
///
/// ```rust
/// use syn::parse_quote;
/// use koruma_derive_core::expr_as_simple_ident;
/// use syn::Expr;
///
/// let expr: Expr = parse_quote!(password);
/// let ident = expr_as_simple_ident(&expr);
/// assert_eq!(ident.unwrap().to_string(), "password");
///
/// let expr2: Expr = parse_quote!(self.password);
/// let ident2 = expr_as_simple_ident(&expr2);
/// assert!(ident2.is_none());
/// ```
pub fn expr_as_simple_ident(expr: &Expr) -> Option<&Ident> {
    if let Expr::Path(expr_path) = expr
        && expr_path.qself.is_none()
        && expr_path.path.segments.len() == 1
        && expr_path.path.segments[0].arguments.is_empty()
    {
        Some(&expr_path.path.segments[0].ident)
    } else {
        None
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
    let Type::Path(type_path) = ty else {
        return None;
    };
    let segment = type_path.path.segments.last()?;

    if segment.ident != "Option" {
        return None;
    }

    let PathArguments::AngleBracketed(args) = &segment.arguments else {
        return None;
    };

    match args.args.first()? {
        GenericArgument::Type(inner) => Some(inner),
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
    let Type::Path(type_path) = ty else {
        return None;
    };
    let segment = type_path.path.segments.last()?;

    if segment.ident != "Vec" {
        return None;
    }

    let PathArguments::AngleBracketed(args) = &segment.arguments else {
        return None;
    };

    match args.args.first()? {
        GenericArgument::Type(inner) => Some(inner),
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
