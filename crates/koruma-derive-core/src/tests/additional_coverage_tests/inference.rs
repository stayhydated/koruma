use super::support::*;

#[test]
fn infer_type_substitution_recurses_through_non_path_types() {
    let infer_target: syn::Type = syn::parse_quote!(String);
    let explicit: syn::Type = syn::parse_quote!((Option<&_>, [_; 4], fn(_) -> _));
    assert!(contains_infer_type(&explicit));

    let substituted = substitute_infer_type(&explicit, &infer_target);
    let expected: syn::Type =
        syn::parse_quote!((Option<&String>, [String; 4], fn(String) -> String));
    assert_eq!(substituted, expected);
    assert!(!contains_infer_type(&substituted));
}

#[test]
fn source_infer_type_substitution_uses_matching_non_path_shape() {
    let explicit: syn::Type = syn::parse_quote!((&_, Vec<_>, [_; 2], fn(_) -> _));
    let source: syn::Type = syn::parse_quote!((&'a str, Vec<u8>, [bool; 2], fn(char) -> usize));
    let substituted = substitute_infer_type_from_source(&explicit, &source)
        .expect("expected tuple-shaped source inference");
    let expected: syn::Type = syn::parse_quote!((&'a str, Vec<u8>, [bool; 2], fn(char) -> usize));
    assert_eq!(substituted, expected);
}

#[test]
fn source_infer_type_substitution_reaches_associated_type_bounds() {
    let explicit: syn::Type = syn::parse_quote!(Box<dyn Iterator<Item = _>>);
    let source: syn::Type = syn::parse_quote!(String);
    let substituted = substitute_infer_type_from_source(&explicit, &source)
        .expect("expected associated type infer substitution");
    let expected: syn::Type = syn::parse_quote!(Box<dyn Iterator<Item = String>>);
    assert_eq!(substituted, expected);

    let explicit: syn::Type = syn::parse_quote!(Box<dyn Iterator<Item = _>>);
    let source: syn::Type = syn::parse_quote!(Box<dyn Iterator<Item = u8>>);
    let substituted = substitute_infer_type_from_source(&explicit, &source)
        .expect("expected associated type inference from matching trait object");
    let expected: syn::Type = syn::parse_quote!(Box<dyn Iterator<Item = u8>>);
    assert_eq!(substituted, expected);
}

#[test]
fn infer_substitution_covers_syntax_only_type_shapes() {
    let infer_target: syn::Type = syn::parse_quote!(usize);

    let grouped = syn::Type::Group(syn::TypeGroup {
        attrs: Vec::new(),
        group_token: Default::default(),
        elem: Box::new(syn::parse_quote!(Option<_>)),
    });
    let grouped_substituted = substitute_infer_type(&grouped, &infer_target);
    assert_eq!(
        quote::quote!(#grouped_substituted).to_string(),
        "Option < usize >"
    );

    let impl_trait: syn::Type = syn::parse_quote!(impl Iterator<Item = _>);
    let substituted = substitute_infer_type(&impl_trait, &infer_target);
    assert_eq!(
        quote::quote!(#substituted).to_string(),
        "impl Iterator < Item = usize >"
    );

    let paren: syn::Type = syn::parse_quote!((Option<_>));
    let substituted = substitute_infer_type(&paren, &infer_target);
    assert_eq!(
        quote::quote!(#substituted).to_string(),
        "(Option < usize >)"
    );

    let ptr: syn::Type = syn::parse_quote!(*const _);
    let substituted = substitute_infer_type(&ptr, &infer_target);
    assert_eq!(quote::quote!(#substituted).to_string(), "* const usize");

    let slice: syn::Type = syn::parse_quote!([_]);
    let substituted = substitute_infer_type(&slice, &infer_target);
    assert_eq!(quote::quote!(#substituted).to_string(), "[usize]");

    let trait_object: syn::Type = syn::parse_quote!(dyn Iterator<Item = _>);
    let substituted = substitute_infer_type(&trait_object, &infer_target);
    assert_eq!(
        quote::quote!(#substituted).to_string(),
        "dyn Iterator < Item = usize >"
    );
}

#[test]
fn source_infer_substitution_covers_structural_and_fallback_shapes() {
    let grouped = syn::Type::Group(syn::TypeGroup {
        attrs: Vec::new(),
        group_token: Default::default(),
        elem: Box::new(syn::parse_quote!(_)),
    });
    let source_group = syn::Type::Group(syn::TypeGroup {
        attrs: Vec::new(),
        group_token: Default::default(),
        elem: Box::new(syn::parse_quote!(String)),
    });
    let substituted = substitute_infer_type_from_source(&grouped, &source_group)
        .expect("group source should be used");
    assert_eq!(quote::quote!(#substituted).to_string(), "String");

    let explicit: syn::Type = syn::parse_quote!(impl Iterator<Item = _>);
    let source: syn::Type = syn::parse_quote!(impl Iterator<Item = u8>);
    let substituted = substitute_infer_type_from_source(&explicit, &source)
        .expect("impl trait bounds should infer from matching source bounds");
    assert_eq!(
        quote::quote!(#substituted).to_string(),
        "impl Iterator < Item = u8 >"
    );

    let explicit: syn::Type = syn::parse_quote!((_));
    let source: syn::Type = syn::parse_quote!((bool));
    let substituted =
        substitute_infer_type_from_source(&explicit, &source).expect("paren source should be used");
    assert_eq!(quote::quote!(#substituted).to_string(), "(bool)");

    let explicit: syn::Type = syn::parse_quote!(*const _);
    let source: syn::Type = syn::parse_quote!(*const str);
    let substituted =
        substitute_infer_type_from_source(&explicit, &source).expect("ptr source should be used");
    assert_eq!(quote::quote!(#substituted).to_string(), "* const str");

    let explicit: syn::Type = syn::parse_quote!([_]);
    let source: syn::Type = syn::parse_quote!([char]);
    let substituted =
        substitute_infer_type_from_source(&explicit, &source).expect("slice source should be used");
    assert_eq!(quote::quote!(#substituted).to_string(), "[char]");

    let explicit: syn::Type = syn::parse_quote!(fn(_) -> _);
    let source: syn::Type = syn::parse_quote!(fn(u8));
    let substituted = substitute_infer_type_from_source(&explicit, &source)
        .expect("fallback source should be used for missing return type");
    assert_eq!(
        quote::quote!(#substituted).to_string(),
        "fn (u8) -> fn (u8)"
    );

    let explicit: syn::Type = syn::parse_quote!((_, _));
    let source: syn::Type = syn::parse_quote!(String);
    assert!(substitute_infer_type_from_source(&explicit, &source).is_none());

    let concrete: syn::Type = syn::parse_quote!(Result<String, Error>);
    let source: syn::Type = syn::parse_quote!(u8);
    let substituted = substitute_infer_type_from_source(&concrete, &source)
        .expect("concrete type should not require source inference");
    assert_eq!(substituted, concrete);
}

#[test]
fn source_infer_substitution_handles_path_associated_items_and_constraints() {
    let explicit: syn::Type = syn::parse_quote!(Parser<Output<_> = _, Item: Into<_>>);
    let source: syn::Type = syn::parse_quote!(Parser<Output<String> = bool, Item: Into<u8>>);
    let substituted = substitute_infer_type_from_source(&explicit, &source)
        .expect("associated generic items should infer from matching source arguments");
    assert_eq!(
        quote::quote!(#substituted).to_string(),
        "Parser < Output < String > = bool , Item : Into < Parser < Output < String > = bool , Item : Into < u8 > > > >"
    );
}

#[test]
fn infer_substitution_covers_qself_parenthesized_args_and_clone_fallbacks() {
    let infer_target: syn::Type = syn::parse_quote!(usize);

    let qself: syn::Type = syn::parse_quote!(<_ as Trait>::Assoc);
    let substituted = substitute_infer_type(&qself, &infer_target);
    assert_eq!(
        quote::quote!(#substituted).to_string(),
        "< usize as Trait > :: Assoc"
    );

    let mut inputs = syn::punctuated::Punctuated::new();
    inputs.push(syn::NamedArg {
        attrs: Vec::new(),
        name: None,
        ty: syn::parse_quote!(_),
    });
    let output: syn::ReturnType = syn::parse_quote!(-> _);
    let mut path: syn::Path = syn::parse_quote!(FnOnce);
    path.segments.last_mut().expect("segment").arguments =
        syn::PathArguments::Parenthesized(syn::ParenthesizedGenericArguments {
            paren_token: Default::default(),
            inputs,
            output,
        });
    let parenthesized_path = syn::Type::Path(syn::TypePath {
        attrs: Vec::new(),
        qself: None,
        path,
    });

    let substituted = substitute_infer_type(&parenthesized_path, &infer_target);
    assert_eq!(
        quote::quote!(#substituted).to_string(),
        "FnOnce (usize) -> usize"
    );

    let associated: syn::Type = syn::parse_quote!(Parser<Output<_> = _, Item: Into<_>>);
    let substituted = substitute_infer_type(&associated, &infer_target);
    assert_eq!(
        quote::quote!(#substituted).to_string(),
        "Parser < Output < usize > = usize , Item : Into < usize > >"
    );

    let never: syn::Type = syn::parse_quote!(!);
    let unchanged = substitute_infer_type(&never, &infer_target);
    assert_eq!(unchanged, never);
}

#[test]
fn source_infer_substitution_covers_fallback_sources_and_qself() {
    for (explicit, source, expected) in [
        (
            syn::parse_quote!([_; 2]),
            syn::parse_quote!(String),
            "[String ; 2]",
        ),
        (
            syn::parse_quote!(fn(_) -> _),
            syn::parse_quote!(String),
            "fn (String) -> String",
        ),
        (
            syn::Type::Group(syn::TypeGroup {
                attrs: Vec::new(),
                group_token: Default::default(),
                elem: Box::new(syn::parse_quote!(_)),
            }),
            syn::parse_quote!(String),
            "String",
        ),
        (
            syn::parse_quote!((_)),
            syn::parse_quote!(String),
            "(String)",
        ),
        (
            syn::parse_quote!(*const _),
            syn::parse_quote!(String),
            "* const String",
        ),
        (syn::parse_quote!(&_), syn::parse_quote!(String), "& String"),
        (
            syn::parse_quote!([_]),
            syn::parse_quote!(String),
            "[String]",
        ),
        (
            syn::parse_quote!(<_ as Trait>::Assoc),
            syn::parse_quote!(String),
            "< String as Trait > :: Assoc",
        ),
    ] {
        let substituted = substitute_infer_type_from_source(&explicit, &source)
            .expect("fallback source inference should succeed");
        assert_eq!(quote::quote!(#substituted).to_string(), expected);
    }

    let verbatim = syn::Type::Verbatim(quote::quote!(Custom<_>));
    let source: syn::Type = syn::parse_quote!(String);
    let substituted = substitute_infer_type_from_source(&verbatim, &source)
        .expect("verbatim types do not expose infer structure");
    assert_eq!(substituted, verbatim);
}

#[test]
fn source_infer_substitution_covers_parenthesized_path_arguments() {
    fn fn_once_type(input: syn::Type, output: syn::ReturnType) -> syn::Type {
        let mut inputs = syn::punctuated::Punctuated::new();
        inputs.push(syn::NamedArg {
            attrs: Vec::new(),
            name: None,
            ty: input,
        });
        let mut path: syn::Path = syn::parse_quote!(FnOnce);
        path.segments.last_mut().expect("segment").arguments =
            syn::PathArguments::Parenthesized(syn::ParenthesizedGenericArguments {
                paren_token: Default::default(),
                inputs,
                output,
            });
        syn::Type::Path(syn::TypePath {
            attrs: Vec::new(),
            qself: None,
            path,
        })
    }

    let explicit = fn_once_type(syn::parse_quote!(_), syn::parse_quote!(-> _));
    let source = fn_once_type(syn::parse_quote!(u8), syn::parse_quote!(-> bool));
    assert!(contains_infer_type(&explicit));

    let substituted = substitute_infer_type_from_source(&explicit, &source)
        .expect("parenthesized path arguments should infer structurally");
    assert_eq!(
        quote::quote!(#substituted).to_string(),
        "FnOnce (u8) -> bool"
    );
}

#[test]
fn infer_detection_covers_remaining_generic_argument_and_return_paths() {
    let assoc_without_infer: syn::Type = syn::parse_quote!(Parser<Output<String> = bool>);
    assert!(!contains_infer_type(&assoc_without_infer));

    let constraint_with_infer: syn::Type = syn::parse_quote!(Parser<Item: Into<_>>);
    assert!(contains_infer_type(&constraint_with_infer));

    let lifetime_bound: syn::Type = syn::parse_quote!(impl Clone + 'static);
    assert!(!contains_infer_type(&lifetime_bound));

    let bare_fn_default_return: syn::Type = syn::parse_quote!(fn(u8));
    assert!(!contains_infer_type(&bare_fn_default_return));

    let infer: syn::Type = syn::parse_quote!(_);
    assert!(contains_infer_type(&infer));

    let first_arg: syn::Type = syn::parse_quote!(Vec<String>);
    let first = first_generic_arg(&first_arg).expect("expected first type arg");
    assert_eq!(quote::quote!(#first).to_string(), "String");

    let lifetime_only: syn::Type = syn::parse_quote!(Borrowed<'a>);
    assert!(first_generic_arg(&lifetime_only).is_none());
}
