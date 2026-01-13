//! Utility functions for type manipulation and analysis.
//!
//! These utilities are used for working with syn types, particularly
//! for handling Option<T>, Vec<T>, and type inference placeholders.

use syn::{Expr, GenericArgument, Ident, PathArguments, Type};

/// Substitute infer placeholders (`_`) in a type with the actual inferred type.
///
/// For example, `Vec<_>` with `infer_ty=String` becomes `Vec<String>`.
///
/// # Examples
///
/// ```ignore
/// use syn::parse_quote;
/// use koruma_derive_core::substitute_infer_type;
///
/// let ty: Type = parse_quote!(Vec<_>);
/// let infer_ty: Type = parse_quote!(String);
/// let result = substitute_infer_type(&ty, &infer_ty);
/// // result is Vec<String>
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

/// Extract the first generic type argument from a type.
///
/// For example, `Vec<String>` → `String`, `HashSet<i32>` → `i32`.
///
/// # Examples
///
/// ```ignore
/// use syn::parse_quote;
/// use koruma_derive_core::first_generic_arg;
///
/// let ty: Type = parse_quote!(Vec<String>);
/// let inner = first_generic_arg(&ty);
/// // inner is Some(&String)
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
/// ```ignore
/// use syn::parse_quote;
/// use koruma_derive_core::contains_infer_type;
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
/// like `RequiredValidation::<Option<_>>`.
///
/// # Examples
///
/// ```ignore
/// use syn::parse_quote;
/// use koruma_derive_core::is_option_infer_type;
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
/// ```ignore
/// use syn::parse_quote;
/// use koruma_derive_core::expr_as_simple_ident;
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
/// ```ignore
/// use syn::parse_quote;
/// use koruma_derive_core::option_inner_type;
///
/// let ty: Type = parse_quote!(Option<String>);
/// let inner = option_inner_type(&ty);
/// // inner is Some(&String)
///
/// let ty2: Type = parse_quote!(String);
/// let inner2 = option_inner_type(&ty2);
/// // inner2 is None
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
/// ```ignore
/// use syn::parse_quote;
/// use koruma_derive_core::vec_inner_type;
///
/// let ty: Type = parse_quote!(Vec<String>);
/// let inner = vec_inner_type(&ty);
/// // inner is Some(&String)
///
/// let ty2: Type = parse_quote!(String);
/// let inner2 = vec_inner_type(&ty2);
/// // inner2 is None
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
/// ```ignore
/// use syn::parse_quote;
/// use koruma_derive_core::is_option_type;
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
