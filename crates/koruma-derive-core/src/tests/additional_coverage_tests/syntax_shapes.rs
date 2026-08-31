use super::support::*;

#[test]
fn utility_functions_cover_non_happy_paths() {
    let explicit_tuple: syn::Type = syn::parse_quote!((i32, i32));
    let infer_target: syn::Type = syn::parse_quote!(String);
    let unchanged = substitute_infer_type(&explicit_tuple, &infer_target);
    assert_eq!(quote::quote!(#unchanged).to_string(), "(i32 , i32)");

    let explicit_with_infer: syn::Type = syn::parse_quote!(Vec<_>);
    let substituted = substitute_infer_type(&explicit_with_infer, &infer_target);
    assert_eq!(quote::quote!(#substituted).to_string(), "Vec < String >");

    // Type with lifetime only (no type args) - exercises the for loop without Type match
    let ty_with_lifetime_only: syn::Type = syn::parse_quote!(Borrowed<'a>);
    let substituted_lifetime = substitute_infer_type(&ty_with_lifetime_only, &infer_target);
    assert_eq!(
        quote::quote!(#substituted_lifetime).to_string(),
        "Borrowed < 'a >"
    );

    // Nested path with type args - exercises all branches
    let nested_path: syn::Type = syn::parse_quote!(std::collections::HashMap<_, _>);
    let substituted_nested = substitute_infer_type(&nested_path, &infer_target);
    assert_eq!(
        quote::quote!(#substituted_nested).to_string(),
        "std :: collections :: HashMap < String , String >"
    );

    let inferred_map = substitute_infer_type_from_source(
        &nested_path,
        &syn::parse_quote!(std::collections::HashMap<String, i32>),
    )
    .expect("expected matching multi-generic source inference");
    assert_eq!(
        quote::quote!(#inferred_map).to_string(),
        "std :: collections :: HashMap < String , i32 >"
    );

    let wrapped_vec = substitute_infer_type_from_source(
        &syn::parse_quote!(Vec<_>),
        &syn::parse_quote!(Option<String>),
    )
    .expect("expected single-slot wrapper inference");
    assert_eq!(quote::quote!(#wrapped_vec).to_string(), "Vec < String >");

    assert!(
        substitute_infer_type_from_source(
            &syn::parse_quote!(std::collections::HashMap<_, _>),
            &syn::parse_quote!(Option<std::collections::HashMap<String, i32>>),
        )
        .is_none()
    );

    let const_generic: syn::Type = syn::parse_quote!(ArrayLike<1>);
    assert!(first_generic_arg(&const_generic).is_none());
    assert!(!contains_infer_type(&const_generic));

    let lifetime_generic: syn::Type = syn::parse_quote!(Borrowed<'a>);
    assert!(first_generic_arg(&lifetime_generic).is_none());

    let simple_ident_expr: syn::Expr = syn::parse_quote!(password);
    assert_eq!(
        expr_as_simple_ident(&simple_ident_expr).map(ToString::to_string),
        Some("password".to_string())
    );

    let complex_ident_expr: syn::Expr = syn::parse_quote!(self.value);
    assert!(expr_as_simple_ident(&complex_ident_expr).is_none());

    let tuple_type: syn::Type = syn::parse_quote!((i32, i32));
    assert!(option_inner_type(&tuple_type).is_none());
    assert!(vec_inner_type(&tuple_type).is_none());
    assert!(type_to_ident(&tuple_type).is_none());

    let option_without_args: syn::Type = syn::parse_quote!(Option);
    assert!(option_inner_type(&option_without_args).is_none());

    let option_const: syn::Type = syn::parse_quote!(Option<1>);
    assert!(option_inner_type(&option_const).is_none());

    let vec_without_args: syn::Type = syn::parse_quote!(Vec);
    assert!(vec_inner_type(&vec_without_args).is_none());

    let vec_const: syn::Type = syn::parse_quote!(Vec<1>);
    assert!(vec_inner_type(&vec_const).is_none());

    let qualified_option: syn::Type = syn::parse_quote!(std::option::Option<String>);
    let KnownTypeShape::Option { segment, inner } = KnownTypeShape::of(&qualified_option) else {
        panic!("expected qualified option shape");
    };
    assert_eq!(segment.ident.to_string(), "Option");
    assert_eq!(quote::quote!(#inner).to_string(), "String");

    let qualified_vec: syn::Type = syn::parse_quote!(std::vec::Vec<u8>);
    let KnownTypeShape::Vec { segment, inner } = KnownTypeShape::of(&qualified_vec) else {
        panic!("expected qualified vec shape");
    };
    assert_eq!(segment.ident.to_string(), "Vec");
    assert_eq!(quote::quote!(#inner).to_string(), "u8");

    let reference: syn::Type = syn::parse_quote!(&[u8]);
    let KnownTypeShape::Reference { inner, .. } = KnownTypeShape::of(&reference) else {
        panic!("expected reference shape");
    };
    assert!(matches!(
        KnownTypeShape::of(inner),
        KnownTypeShape::Slice { .. }
    ));

    let array: syn::Type = syn::parse_quote!([u8; 4]);
    assert!(matches!(
        KnownTypeShape::of(&array),
        KnownTypeShape::Array { .. }
    ));

    let named_type: syn::Type = syn::parse_quote!(Age);
    assert_eq!(
        type_to_ident(&named_type).map(|ident| ident.to_string()),
        Some("Age".to_string())
    );
}

#[test]
fn utility_functions_cover_remaining_line_paths() {
    let ty_with_lifetime: syn::Type = syn::parse_quote!(Borrowed<'static>);
    assert!(first_generic_arg(&ty_with_lifetime).is_none());
    assert!(!contains_infer_type(&ty_with_lifetime));

    let ty_ref: syn::Type = syn::parse_quote!(&str);
    assert!(!contains_infer_type(&ty_ref));

    let ty_with_lifetime_and_infer: syn::Type = syn::parse_quote!(Wrapper<'static, _>);
    let infer_target: syn::Type = syn::parse_quote!(usize);
    let substituted = substitute_infer_type(&ty_with_lifetime_and_infer, &infer_target);
    assert_eq!(
        quote::quote!(#substituted).to_string(),
        "Wrapper < 'static , usize >"
    );
}

#[test]
fn infer_detection_and_known_type_shape_cover_public_accessors() {
    let grouped = syn::Type::Group(syn::TypeGroup {
        attrs: Vec::new(),
        group_token: Default::default(),
        elem: Box::new(syn::parse_quote!(_)),
    });
    assert!(contains_infer_type(&grouped));

    for ty in [
        syn::parse_quote!(impl Iterator<Item = _>),
        syn::parse_quote!(*const _),
        syn::parse_quote!([_]),
        syn::parse_quote!(dyn Iterator<Item = _>),
        syn::parse_quote!(Parser<Output<_> = _, Item: Into<_>>),
    ] {
        assert!(
            contains_infer_type(&ty),
            "expected infer marker in {}",
            quote::quote!(#ty)
        );
    }

    let option: syn::Type = syn::parse_quote!(Option<String>);
    let option_shape = KnownTypeShape::of(&option);
    assert_eq!(
        option_shape.recognized_name().map(ToString::to_string),
        Some("Option".to_owned())
    );
    let _ = option_shape.span();

    let slice: syn::Type = syn::parse_quote!([u8]);
    let slice_shape = KnownTypeShape::of(&slice);
    assert!(slice_shape.recognized_name().is_none());
    let _ = slice_shape.span();

    let other: syn::Type = syn::parse_quote!(Result<String, Error>);
    let other_shape = KnownTypeShape::of(&other);
    assert!(other_shape.recognized_name().is_none());
    let _ = other_shape.span();
}
