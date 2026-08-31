use super::*;

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
        Type::Array(array) => contains_infer_type(&array.elem),
        Type::FnPtr(fn_ptr) => {
            fn_ptr.inputs.iter().any(|arg| contains_infer_type(&arg.ty))
                || return_type_contains_infer(&fn_ptr.output)
        },
        Type::Group(group) => contains_infer_type(&group.elem),
        Type::ImplTrait(impl_trait) => bounds_contain_infer(&impl_trait.bounds),
        Type::Infer(_) => true,
        Type::Paren(paren) => contains_infer_type(&paren.elem),
        Type::Path(type_path) => {
            type_path
                .qself
                .as_ref()
                .is_some_and(|qself| contains_infer_type(&qself.ty))
                || path_contains_infer(&type_path.path)
        },
        Type::Ptr(ptr) => contains_infer_type(&ptr.elem),
        Type::Reference(reference) => contains_infer_type(&reference.elem),
        Type::Slice(slice) => contains_infer_type(&slice.elem),
        Type::TraitObject(trait_object) => bounds_contain_infer(&trait_object.bounds),
        Type::Tuple(tuple) => tuple.elems.iter().any(contains_infer_type),
        _ => false,
    }
}

fn path_contains_infer(path: &Path) -> bool {
    path.segments
        .iter()
        .any(|segment| path_arguments_contain_infer(&segment.arguments))
}

fn path_arguments_contain_infer(arguments: &PathArguments) -> bool {
    match arguments {
        PathArguments::None => false,
        PathArguments::AngleBracketed(args) => {
            args.args.iter().any(generic_argument_contains_infer)
        },
        PathArguments::Parenthesized(args) => {
            args.inputs.iter().any(|arg| contains_infer_type(&arg.ty))
                || return_type_contains_infer(&args.output)
        },
    }
}

fn generic_argument_contains_infer(arg: &GenericArgument) -> bool {
    match arg {
        GenericArgument::Type(ty) => contains_infer_type(ty),
        GenericArgument::AssocType(assoc) => {
            assoc
                .generics
                .as_ref()
                .is_some_and(|generics| generics.args.iter().any(generic_argument_contains_infer))
                || contains_infer_type(&assoc.ty)
        },
        GenericArgument::Constraint(constraint) => {
            constraint
                .generics
                .as_ref()
                .is_some_and(|generics| generics.args.iter().any(generic_argument_contains_infer))
                || bounds_contain_infer(&constraint.bounds)
        },
        _ => false,
    }
}

fn bounds_contain_infer(
    bounds: &syn::punctuated::Punctuated<TypeParamBound, syn::Token![+]>,
) -> bool {
    bounds.iter().any(|bound| match bound {
        TypeParamBound::Trait(trait_bound) => path_contains_infer(&trait_bound.path),
        _ => false,
    })
}

fn return_type_contains_infer(return_type: &ReturnType) -> bool {
    match return_type {
        ReturnType::Default => false,
        ReturnType::Type(_, ty) => contains_infer_type(ty),
    }
}
