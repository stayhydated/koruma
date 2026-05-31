//! Utility functions for type manipulation and analysis.
//!
//! These utilities are used for working with syn types, particularly
//! for handling `Option<T>`, `Vec<T>`, and type inference placeholders.

use syn::{
    AngleBracketedGenericArguments, Expr, GenericArgument, Ident, ParenthesizedGenericArguments,
    Path, PathArguments, ReturnType, Type, TypeParamBound,
};

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
        Type::BareFn(bare_fn) => {
            let mut bare_fn = bare_fn.clone();
            for input in &mut bare_fn.inputs {
                input.ty = substitute_infer_type(&input.ty, infer_ty);
            }
            substitute_return_type(&mut bare_fn.output, infer_ty);
            Type::BareFn(bare_fn)
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
                qself.ty = Box::new(substitute_infer_type(&qself.ty, infer_ty));
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

fn substitute_path(path: &mut Path, infer_ty: &Type) {
    for segment in &mut path.segments {
        substitute_path_arguments(&mut segment.arguments, infer_ty);
    }
}

fn substitute_path_arguments(arguments: &mut PathArguments, infer_ty: &Type) {
    match arguments {
        PathArguments::None => {},
        PathArguments::AngleBracketed(args) => substitute_angle_args(args, infer_ty),
        PathArguments::Parenthesized(args) => substitute_parenthesized_args(args, infer_ty),
    }
}

fn substitute_angle_args(args: &mut AngleBracketedGenericArguments, infer_ty: &Type) {
    for arg in &mut args.args {
        substitute_generic_argument(arg, infer_ty);
    }
}

fn substitute_parenthesized_args(args: &mut ParenthesizedGenericArguments, infer_ty: &Type) {
    for input in &mut args.inputs {
        *input = substitute_infer_type(input, infer_ty);
    }
    substitute_return_type(&mut args.output, infer_ty);
}

fn substitute_generic_argument(arg: &mut GenericArgument, infer_ty: &Type) {
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

fn substitute_bounds(
    bounds: &mut syn::punctuated::Punctuated<TypeParamBound, syn::Token![+]>,
    infer_ty: &Type,
) {
    for bound in bounds {
        if let TypeParamBound::Trait(trait_bound) = bound {
            substitute_path(&mut trait_bound.path, infer_ty);
        }
    }
}

fn substitute_return_type(return_type: &mut ReturnType, infer_ty: &Type) {
    if let ReturnType::Type(_, ty) = return_type {
        **ty = substitute_infer_type(ty, infer_ty);
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

    angle_type_args(args)
}

fn angle_type_args(args: &AngleBracketedGenericArguments) -> Vec<&Type> {
    args.args
        .iter()
        .filter_map(|arg| match arg {
            GenericArgument::Type(ty) => Some(ty),
            _ => None,
        })
        .collect()
}

fn matching_source_angle_args<'a>(
    explicit_ty: &Type,
    source_ty: &'a Type,
) -> Option<&'a AngleBracketedGenericArguments> {
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

    match (&explicit_segment.arguments, &source_segment.arguments) {
        (PathArguments::AngleBracketed(_), PathArguments::AngleBracketed(source_args)) => {
            Some(source_args)
        },
        _ => None,
    }
}

fn matching_source_type_args<'a>(explicit_ty: &Type, source_ty: &'a Type) -> Option<Vec<&'a Type>> {
    let explicit_args = type_generic_args(explicit_ty);
    let source_args = angle_type_args(matching_source_angle_args(explicit_ty, source_ty)?);

    if explicit_args.len() != source_args.len() {
        return None;
    }

    Some(source_args)
}

fn substitute_infer_type_from_source_inner(ty: &Type, source_ty: &Type) -> Option<Type> {
    match ty {
        Type::Array(array) => {
            let mut array = array.clone();
            let child_source = match source_ty {
                Type::Array(source) => &source.elem,
                _ => source_ty,
            };
            array.elem = Box::new(substitute_infer_type_from_source_inner(
                &array.elem,
                child_source,
            )?);
            Some(Type::Array(array))
        },
        Type::BareFn(bare_fn) => {
            let mut bare_fn = bare_fn.clone();
            let source_fn = match source_ty {
                Type::BareFn(source) if source.inputs.len() == bare_fn.inputs.len() => Some(source),
                _ => None,
            };

            for (index, input) in bare_fn.inputs.iter_mut().enumerate() {
                let child_source = source_fn
                    .and_then(|source| source.inputs.iter().nth(index))
                    .map(|arg| &arg.ty)
                    .unwrap_or(source_ty);
                input.ty = substitute_infer_type_from_source_inner(&input.ty, child_source)?;
            }

            substitute_return_type_from_source(&mut bare_fn.output, source_ty, source_fn)?;
            Some(Type::BareFn(bare_fn))
        },
        Type::Group(group) => {
            let mut group = group.clone();
            let child_source = match source_ty {
                Type::Group(source) => &source.elem,
                _ => source_ty,
            };
            group.elem = Box::new(substitute_infer_type_from_source_inner(
                &group.elem,
                child_source,
            )?);
            Some(Type::Group(group))
        },
        Type::ImplTrait(impl_trait) => {
            let mut impl_trait = impl_trait.clone();
            substitute_bounds_from_source(&mut impl_trait.bounds, source_ty)?;
            Some(Type::ImplTrait(impl_trait))
        },
        Type::Infer(_) => Some(source_ty.clone()),
        Type::Paren(paren) => {
            let mut paren = paren.clone();
            let child_source = match source_ty {
                Type::Paren(source) => &source.elem,
                _ => source_ty,
            };
            paren.elem = Box::new(substitute_infer_type_from_source_inner(
                &paren.elem,
                child_source,
            )?);
            Some(Type::Paren(paren))
        },
        Type::Path(type_path) => {
            let explicit_args = type_generic_args(ty);
            let mut new_path = type_path.clone();
            let structural_source_angle_args = matching_source_angle_args(ty, source_ty);
            let structural_source_args = matching_source_type_args(ty, source_ty);
            let structural_parenthesized_args = matching_source_parenthesized_args(ty, source_ty);

            let fallback_source =
                match (explicit_args.len(), type_generic_args(source_ty).as_slice()) {
                    (1, [only]) => Some(*only),
                    (1, []) => Some(source_ty),
                    _ => None,
                };

            let mut source_index = 0usize;
            if let Some(qself) = &mut new_path.qself {
                qself.ty = Box::new(substitute_infer_type_from_source_inner(
                    &qself.ty, source_ty,
                )?);
            }
            let last_segment_index = new_path.path.segments.len().saturating_sub(1);
            for (index, segment) in new_path.path.segments.iter_mut().enumerate() {
                let segment_source_angle_args = (index == last_segment_index)
                    .then_some(structural_source_angle_args)
                    .flatten();
                let segment_source_parenthesized_args = (index == last_segment_index)
                    .then_some(structural_parenthesized_args)
                    .flatten();
                substitute_path_arguments_from_source(
                    &mut segment.arguments,
                    structural_source_args.as_deref(),
                    segment_source_angle_args,
                    segment_source_parenthesized_args,
                    fallback_source,
                    source_ty,
                    &mut source_index,
                )?;
            }
            Some(Type::Path(new_path))
        },
        Type::Ptr(ptr) => {
            let mut ptr = ptr.clone();
            let child_source = match source_ty {
                Type::Ptr(source) => &source.elem,
                _ => source_ty,
            };
            ptr.elem = Box::new(substitute_infer_type_from_source_inner(
                &ptr.elem,
                child_source,
            )?);
            Some(Type::Ptr(ptr))
        },
        Type::Reference(reference) => {
            let mut reference = reference.clone();
            let child_source = match source_ty {
                Type::Reference(source) => {
                    if reference.lifetime.is_none() {
                        reference.lifetime.clone_from(&source.lifetime);
                    }
                    &source.elem
                },
                _ => source_ty,
            };
            reference.elem = Box::new(substitute_infer_type_from_source_inner(
                &reference.elem,
                child_source,
            )?);
            Some(Type::Reference(reference))
        },
        Type::Slice(slice) => {
            let mut slice = slice.clone();
            let child_source = match source_ty {
                Type::Slice(source) => &source.elem,
                _ => source_ty,
            };
            slice.elem = Box::new(substitute_infer_type_from_source_inner(
                &slice.elem,
                child_source,
            )?);
            Some(Type::Slice(slice))
        },
        Type::TraitObject(trait_object) => {
            let mut trait_object = trait_object.clone();
            substitute_bounds_from_source(&mut trait_object.bounds, source_ty)?;
            Some(Type::TraitObject(trait_object))
        },
        Type::Tuple(tuple) => {
            let source_tuple = match source_ty {
                Type::Tuple(source) if source.elems.len() == tuple.elems.len() => Some(source),
                _ => None,
            };

            if source_tuple.is_none() && tuple.elems.iter().any(contains_infer_type) {
                return None;
            }

            let mut tuple = tuple.clone();
            for (index, elem) in tuple.elems.iter_mut().enumerate() {
                let child_source = source_tuple
                    .and_then(|source| source.elems.iter().nth(index))
                    .unwrap_or(source_ty);
                *elem = substitute_infer_type_from_source_inner(elem, child_source)?;
            }
            Some(Type::Tuple(tuple))
        },
        _ => {
            if contains_infer_type(ty) {
                None
            } else {
                Some(ty.clone())
            }
        },
    }
}

fn matching_source_parenthesized_args<'a>(
    explicit_ty: &Type,
    source_ty: &'a Type,
) -> Option<&'a ParenthesizedGenericArguments> {
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

    match (&explicit_segment.arguments, &source_segment.arguments) {
        (
            PathArguments::Parenthesized(explicit_args),
            PathArguments::Parenthesized(source_args),
        ) if explicit_args.inputs.len() == source_args.inputs.len() => Some(source_args),
        _ => None,
    }
}

fn substitute_path_arguments_from_source(
    arguments: &mut PathArguments,
    structural_source_args: Option<&[&Type]>,
    structural_source_angle_args: Option<&AngleBracketedGenericArguments>,
    structural_parenthesized_args: Option<&ParenthesizedGenericArguments>,
    fallback_source: Option<&Type>,
    source_ty: &Type,
    source_index: &mut usize,
) -> Option<()> {
    match arguments {
        PathArguments::None => Some(()),
        PathArguments::AngleBracketed(args) => substitute_angle_args_from_source(
            args,
            structural_source_args,
            structural_source_angle_args,
            fallback_source,
            source_ty,
            source_index,
        ),
        PathArguments::Parenthesized(args) => substitute_parenthesized_args_from_source(
            args,
            structural_parenthesized_args,
            source_ty,
        ),
    }
}

fn substitute_angle_args_from_source(
    args: &mut AngleBracketedGenericArguments,
    structural_source_args: Option<&[&Type]>,
    structural_source_angle_args: Option<&AngleBracketedGenericArguments>,
    fallback_source: Option<&Type>,
    source_ty: &Type,
    source_index: &mut usize,
) -> Option<()> {
    for arg in &mut args.args {
        substitute_generic_argument_from_source(
            arg,
            structural_source_args,
            structural_source_angle_args,
            fallback_source,
            source_ty,
            source_index,
        )?;
    }
    Some(())
}

fn substitute_parenthesized_args_from_source(
    args: &mut ParenthesizedGenericArguments,
    structural_source_args: Option<&ParenthesizedGenericArguments>,
    source_ty: &Type,
) -> Option<()> {
    for (index, input) in args.inputs.iter_mut().enumerate() {
        let child_source = structural_source_args
            .and_then(|source| source.inputs.iter().nth(index))
            .unwrap_or(source_ty);
        *input = substitute_infer_type_from_source_inner(input, child_source)?;
    }

    let source_output = structural_source_args.and_then(|source| match &source.output {
        ReturnType::Type(_, ty) => Some(ty.as_ref()),
        ReturnType::Default => None,
    });
    substitute_return_type_from_source_with_output(&mut args.output, source_ty, source_output)
}

fn substitute_generic_argument_from_source(
    arg: &mut GenericArgument,
    structural_source_args: Option<&[&Type]>,
    structural_source_angle_args: Option<&AngleBracketedGenericArguments>,
    fallback_source: Option<&Type>,
    source_ty: &Type,
    source_index: &mut usize,
) -> Option<()> {
    match arg {
        GenericArgument::Type(inner_ty) => {
            let child_source = structural_source_args
                .and_then(|args| args.get(*source_index).copied())
                .or(fallback_source);
            *source_index += 1;
            if contains_infer_type(inner_ty) {
                *inner_ty = substitute_infer_type_from_source_inner(inner_ty, child_source?)?;
            }
        },
        GenericArgument::AssocType(assoc) => {
            let source_assoc = structural_source_angle_args.and_then(|args| {
                args.args.iter().find_map(|arg| match arg {
                    GenericArgument::AssocType(source_assoc)
                        if source_assoc.ident == assoc.ident =>
                    {
                        Some(source_assoc)
                    },
                    _ => None,
                })
            });

            if let Some(generics) = &mut assoc.generics {
                let source_generics = source_assoc.and_then(|source| source.generics.as_ref());
                let source_type_args = source_generics.map(angle_type_args);
                substitute_angle_args_from_source(
                    generics,
                    source_type_args.as_deref(),
                    source_generics,
                    fallback_source.or(Some(source_ty)),
                    source_ty,
                    source_index,
                )?;
            }
            if contains_infer_type(&assoc.ty) {
                assoc.ty = substitute_infer_type_from_source_inner(
                    &assoc.ty,
                    source_assoc
                        .map(|source| &source.ty)
                        .or(fallback_source)
                        .unwrap_or(source_ty),
                )?;
            }
        },
        GenericArgument::Constraint(constraint) => {
            let source_constraint = structural_source_angle_args.and_then(|args| {
                args.args.iter().find_map(|arg| match arg {
                    GenericArgument::Constraint(source_constraint)
                        if source_constraint.ident == constraint.ident =>
                    {
                        Some(source_constraint)
                    },
                    _ => None,
                })
            });

            if let Some(generics) = &mut constraint.generics {
                let source_generics = source_constraint.and_then(|source| source.generics.as_ref());
                let source_type_args = source_generics.map(angle_type_args);
                substitute_angle_args_from_source(
                    generics,
                    source_type_args.as_deref(),
                    source_generics,
                    fallback_source.or(Some(source_ty)),
                    source_ty,
                    source_index,
                )?;
            }
            substitute_bounds_from_source(
                &mut constraint.bounds,
                source_constraint
                    .map(|_| source_ty)
                    .or(fallback_source)
                    .unwrap_or(source_ty),
            )?;
        },
        _ => {},
    }

    Some(())
}

fn path_angle_args(path: &Path) -> Option<&AngleBracketedGenericArguments> {
    let segment = path.segments.last()?;
    match &segment.arguments {
        PathArguments::AngleBracketed(args) => Some(args),
        _ => None,
    }
}

fn matching_source_bound_angle_args<'a>(
    path: &Path,
    source_ty: &'a Type,
) -> Option<&'a AngleBracketedGenericArguments> {
    let source_bounds = match source_ty {
        Type::ImplTrait(impl_trait) => &impl_trait.bounds,
        Type::TraitObject(trait_object) => &trait_object.bounds,
        _ => return None,
    };

    let path_segment = path.segments.last()?;
    source_bounds.iter().find_map(|bound| match bound {
        TypeParamBound::Trait(trait_bound)
            if trait_bound
                .path
                .segments
                .last()
                .is_some_and(|source_segment| source_segment.ident == path_segment.ident) =>
        {
            path_angle_args(&trait_bound.path)
        },
        _ => None,
    })
}

fn substitute_bounds_from_source(
    bounds: &mut syn::punctuated::Punctuated<TypeParamBound, syn::Token![+]>,
    source_ty: &Type,
) -> Option<()> {
    for bound in bounds {
        if let TypeParamBound::Trait(trait_bound) = bound {
            let structural_source_angle_args =
                matching_source_bound_angle_args(&trait_bound.path, source_ty);
            let source_type_args = structural_source_angle_args.map(angle_type_args);
            let last_segment_index = trait_bound.path.segments.len().saturating_sub(1);
            let mut source_index = 0usize;

            for (index, segment) in trait_bound.path.segments.iter_mut().enumerate() {
                let segment_source_angle_args = (index == last_segment_index)
                    .then_some(structural_source_angle_args)
                    .flatten();
                substitute_path_arguments_from_source(
                    &mut segment.arguments,
                    source_type_args.as_deref(),
                    segment_source_angle_args,
                    None,
                    Some(source_ty),
                    source_ty,
                    &mut source_index,
                )?;
            }
        }
    }
    Some(())
}

fn substitute_return_type_from_source(
    return_type: &mut ReturnType,
    source_ty: &Type,
    source_fn: Option<&syn::TypeBareFn>,
) -> Option<()> {
    let source_output = source_fn.and_then(|source| match &source.output {
        ReturnType::Type(_, ty) => Some(ty.as_ref()),
        ReturnType::Default => None,
    });
    substitute_return_type_from_source_with_output(return_type, source_ty, source_output)
}

fn substitute_return_type_from_source_with_output(
    return_type: &mut ReturnType,
    source_ty: &Type,
    source_output: Option<&Type>,
) -> Option<()> {
    if let ReturnType::Type(_, ty) = return_type {
        **ty = substitute_infer_type_from_source_inner(ty, source_output.unwrap_or(source_ty))?;
    }
    Some(())
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
        Type::Array(array) => contains_infer_type(&array.elem),
        Type::BareFn(bare_fn) => {
            bare_fn
                .inputs
                .iter()
                .any(|arg| contains_infer_type(&arg.ty))
                || return_type_contains_infer(&bare_fn.output)
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
            args.inputs.iter().any(contains_infer_type) || return_type_contains_infer(&args.output)
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
