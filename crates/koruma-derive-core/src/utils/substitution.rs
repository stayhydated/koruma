use super::*;

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
        Type::Array(array) => {
            let mut array = array.clone();
            array.elem = Box::new(substitute_infer_type(&array.elem, infer_ty));
            Type::Array(array)
        },
        Type::FnPtr(fn_ptr) => {
            let mut fn_ptr = fn_ptr.clone();
            for input in &mut fn_ptr.inputs {
                input.ty = substitute_infer_type(&input.ty, infer_ty);
            }
            substitute_return_type(&mut fn_ptr.output, infer_ty);
            Type::FnPtr(fn_ptr)
        },
        Type::Group(group) => {
            let mut group = group.clone();
            group.elem = Box::new(substitute_infer_type(&group.elem, infer_ty));
            Type::Group(group)
        },
        Type::ImplTrait(impl_trait) => {
            let mut impl_trait = impl_trait.clone();
            substitute_bounds(&mut impl_trait.bounds, infer_ty);
            Type::ImplTrait(impl_trait)
        },
        Type::Infer(_) => infer_ty.clone(),
        Type::Paren(paren) => {
            let mut paren = paren.clone();
            paren.elem = Box::new(substitute_infer_type(&paren.elem, infer_ty));
            Type::Paren(paren)
        },
        Type::Path(type_path) => {
            let mut new_path = type_path.clone();
            if let Some(qself) = &mut new_path.qself {
                *qself.ty = substitute_infer_type(&qself.ty, infer_ty);
            }
            substitute_path(&mut new_path.path, infer_ty);
            Type::Path(new_path)
        },
        Type::Ptr(ptr) => {
            let mut ptr = ptr.clone();
            ptr.elem = Box::new(substitute_infer_type(&ptr.elem, infer_ty));
            Type::Ptr(ptr)
        },
        Type::Reference(reference) => {
            let mut reference = reference.clone();
            reference.elem = Box::new(substitute_infer_type(&reference.elem, infer_ty));
            Type::Reference(reference)
        },
        Type::Slice(slice) => {
            let mut slice = slice.clone();
            slice.elem = Box::new(substitute_infer_type(&slice.elem, infer_ty));
            Type::Slice(slice)
        },
        Type::TraitObject(trait_object) => {
            let mut trait_object = trait_object.clone();
            substitute_bounds(&mut trait_object.bounds, infer_ty);
            Type::TraitObject(trait_object)
        },
        Type::Tuple(tuple) => {
            let mut tuple = tuple.clone();
            for elem in &mut tuple.elems {
                *elem = substitute_infer_type(elem, infer_ty);
            }
            Type::Tuple(tuple)
        },
        _ => ty.clone(),
    }
}

pub(super) fn substitute_path(path: &mut Path, infer_ty: &Type) {
    for segment in &mut path.segments {
        substitute_path_arguments(&mut segment.arguments, infer_ty);
    }
}

pub(super) fn substitute_path_arguments(arguments: &mut PathArguments, infer_ty: &Type) {
    match arguments {
        PathArguments::None => {},
        PathArguments::AngleBracketed(args) => substitute_angle_args(args, infer_ty),
        PathArguments::Parenthesized(args) => substitute_parenthesized_args(args, infer_ty),
    }
}

pub(super) fn substitute_angle_args(args: &mut AngleBracketedGenericArguments, infer_ty: &Type) {
    for arg in &mut args.args {
        substitute_generic_argument(arg, infer_ty);
    }
}

pub(super) fn substitute_parenthesized_args(
    args: &mut ParenthesizedGenericArguments,
    infer_ty: &Type,
) {
    for input in &mut args.inputs {
        input.ty = substitute_infer_type(&input.ty, infer_ty);
    }
    substitute_return_type(&mut args.output, infer_ty);
}

pub(super) fn substitute_generic_argument(arg: &mut GenericArgument, infer_ty: &Type) {
    match arg {
        GenericArgument::Type(inner_ty) => {
            *inner_ty = substitute_infer_type(inner_ty, infer_ty);
        },
        GenericArgument::AssocType(assoc) => {
            if let Some(generics) = &mut assoc.generics {
                substitute_angle_args(generics, infer_ty);
            }
            assoc.ty = substitute_infer_type(&assoc.ty, infer_ty);
        },
        GenericArgument::Constraint(constraint) => {
            if let Some(generics) = &mut constraint.generics {
                substitute_angle_args(generics, infer_ty);
            }
            substitute_bounds(&mut constraint.bounds, infer_ty);
        },
        _ => {},
    }
}

pub(super) fn substitute_bounds(
    bounds: &mut syn::punctuated::Punctuated<TypeParamBound, syn::Token![+]>,
    infer_ty: &Type,
) {
    for bound in bounds {
        if let TypeParamBound::Trait(trait_bound) = bound {
            substitute_path(&mut trait_bound.path, infer_ty);
        }
    }
}

pub(super) fn substitute_return_type(return_type: &mut ReturnType, infer_ty: &Type) {
    if let ReturnType::Type(_, ty) = return_type {
        **ty = substitute_infer_type(ty, infer_ty);
    }
}
