use super::*;

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
        Type::FnPtr(fn_ptr) => {
            let mut fn_ptr = fn_ptr.clone();
            let source_fn = match source_ty {
                Type::FnPtr(source) if source.inputs.len() == fn_ptr.inputs.len() => Some(source),
                _ => None,
            };

            for (index, input) in fn_ptr.inputs.iter_mut().enumerate() {
                let child_source = source_fn
                    .and_then(|source| source.inputs.iter().nth(index))
                    .map(|arg| &arg.ty)
                    .unwrap_or(source_ty);
                input.ty = substitute_infer_type_from_source_inner(&input.ty, child_source)?;
            }

            substitute_return_type_from_source(&mut fn_ptr.output, source_ty, source_fn)?;
            Some(Type::FnPtr(fn_ptr))
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
                *qself.ty = substitute_infer_type_from_source_inner(&qself.ty, source_ty)?;
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
            .map(|arg| &arg.ty)
            .unwrap_or(source_ty);
        input.ty = substitute_infer_type_from_source_inner(&input.ty, child_source)?;
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
    source_fn: Option<&syn::TypeFnPtr>,
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
