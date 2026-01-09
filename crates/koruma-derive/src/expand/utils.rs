use syn::{Expr, GenericArgument, Ident, PathArguments, Type};

/// Substitute infer placeholders (`_`) in a type with the actual inferred type.
/// For example, `Vec<_>` with infer_ty=`String` becomes `Vec<String>`.
pub(crate) fn substitute_infer_type(ty: &Type, infer_ty: &Type) -> Type {
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
/// For example, `Vec<String>` → `String`, `HashSet<i32>` → `i32`.
pub(crate) fn first_generic_arg(ty: &Type) -> Option<&Type> {
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
pub(crate) fn contains_infer_type(ty: &Type) -> bool {
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
/// This is used when a validator explicitly wants the full Option type, like RequiredValidation.
pub(crate) fn is_option_infer_type(ty: &Type) -> bool {
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
/// If so, return the identifier. This is used to detect field references in validator args.
pub(crate) fn expr_as_simple_ident(expr: &Expr) -> Option<&Ident> {
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

/// Extract the inner type T from Option<T>
pub(crate) fn option_inner_type(ty: &Type) -> Option<&Type> {
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

/// Extract the inner type T from Vec<T>
pub(crate) fn vec_inner_type(ty: &Type) -> Option<&Type> {
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

/// Check if a field type is Option<T>
pub(crate) fn is_option_type(ty: &Type) -> bool {
    option_inner_type(ty).is_some()
}
