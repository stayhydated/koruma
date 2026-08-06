use crate::{
    DataFieldKorumaAttr, DataFieldKorumaItem, FieldInfo, FieldModifierKind, KnownTypeShape,
    ParsedDataField, SetterDefault, SetterPresence, StructKorumaAttr, StructKorumaItem, StructMode,
    ValidatorAttr, ValidatorFieldRole, ValidatorLabel, ValidatorTargetSelector,
    contains_infer_type, expr_as_simple_ident, first_generic_arg, option_inner_type, parse_field,
    parse_struct_options, parse_validator_struct, substitute_infer_type,
    substitute_infer_type_from_source, type_to_ident, vec_inner_type,
};

fn parse_field_info(field: &syn::Field) -> FieldInfo {
    let ParsedDataField::Participating(info) = parse_field(field, 0).expect("expected field parse")
    else {
        panic!("expected participating field")
    };
    info
}

#[test]
fn validator_attr_helpers_and_error_paths() {
    let plain: ValidatorAttr = syn::parse_quote!(RangeValidation);
    assert!(!plain.has_args());
    assert!(!plain.uses_type_inference());
    assert!(!plain.has_explicit_type());

    let with_builder_methods: ValidatorAttr = syn::parse_quote!(RangeValidation.min(0).max(10));
    assert!(with_builder_methods.has_args());
    let builder_calls = with_builder_methods.setter_calls();
    assert_eq!(builder_calls.len(), 2);
    assert_eq!(builder_calls[0].method().to_string(), "min");
    assert_eq!(builder_calls[1].method().to_string(), "max");

    let associated_setter: Result<ValidatorAttr, _> = syn::parse_str("RangeValidation::min(0)");
    assert!(
        associated_setter
            .expect_err("expected associated setter starters to be rejected")
            .to_string()
            .contains("expected validator chain")
    );

    let infer: ValidatorAttr = syn::parse_quote!(GenericValidation::<_>);
    assert!(infer.uses_type_inference());
    assert!(!infer.has_explicit_type());

    let explicit: ValidatorAttr = syn::parse_quote!(GenericValidation::<i32>);
    assert!(!explicit.uses_type_inference());
    assert!(explicit.has_explicit_type());

    let builder_only: Result<ValidatorAttr, _> = syn::parse_str("RangeValidation.builder()");
    assert!(
        builder_only
            .expect_err("expected builder entrypoint to be rejected")
            .to_string()
            .contains("outside Koruma's validator attribute grammar")
    );

    let builder_chain: Result<ValidatorAttr, _> =
        syn::parse_str("RangeValidation::<_>.builder().min(0).max(10)");
    assert!(
        builder_chain
            .expect_err("expected builder entrypoint chains to be rejected")
            .to_string()
            .contains("outside Koruma's validator attribute grammar")
    );

    let fq_direct_chain: ValidatorAttr =
        syn::parse_quote!(validators::normal::NumberRangeValidation::<_>.min(1).max(5));
    assert_eq!(
        fq_direct_chain.path_name(),
        "validators::normal::NumberRangeValidation"
    );
    assert_eq!(fq_direct_chain.setter_calls().len(), 2);

    let too_many_types: Result<ValidatorAttr, _> = syn::parse_str("GenericValidation::<i32, u32>");
    assert!(
        too_many_types
            .expect_err("expected parse error")
            .to_string()
            .contains("exactly one type argument")
    );

    let non_type_generic: Result<ValidatorAttr, _> = syn::parse_str("GenericValidation::<1>");
    assert!(
        non_type_generic
            .expect_err("expected parse error")
            .to_string()
            .contains("expects a type argument")
    );

    let rejected_shorthand: Result<ValidatorAttr, _> = syn::parse_str("GenericValidation<_>");
    assert!(
        rejected_shorthand
            .expect_err("expected shorthand syntax to be rejected")
            .to_string()
            .contains("requires a dot validator chain")
    );

    let builder_with_build: Result<ValidatorAttr, _> =
        syn::parse_str("GenericValidation.min(1).build()");
    assert!(
        builder_with_build
            .expect_err("expected validator chains to reject .build()")
            .to_string()
            .contains("injects builder creation, value capture, and `.build()` automatically")
    );

    let builder_with_arg: Result<ValidatorAttr, _> = syn::parse_str("RangeValidation.builder(123)");
    assert!(
        builder_with_arg
            .expect_err("expected builder args to be rejected")
            .to_string()
            .contains("outside Koruma's validator attribute grammar")
    );

    let builder_with_turbofish: Result<ValidatorAttr, _> =
        syn::parse_str("RangeValidation::builder::<i32>()");
    assert!(
        builder_with_turbofish
            .expect_err("expected builder turbofish to be rejected")
            .to_string()
            .contains("expected validator chain")
    );

    let builder_then_build: Result<ValidatorAttr, _> =
        syn::parse_str("RangeValidation.builder().build()");
    assert!(
        builder_then_build
            .expect_err("expected .build() to be rejected")
            .to_string()
            .contains("outside Koruma's validator attribute grammar")
    );

    let parenthesized_path: Result<ValidatorAttr, _> = syn::parse_str("std::ops::Fn(i32)");
    assert!(parenthesized_path.is_err());

    let direct_parenthesized: Result<ValidatorAttr, _> = syn::parse_str("Fn(i32)");
    assert!(direct_parenthesized.is_err());
}

#[test]
fn koruma_attr_helpers_and_newtype_parsing_paths() {
    let attr: DataFieldKorumaAttr =
        syn::parse_quote!(RangeValidation.min(0).max(10), each(PositiveValidation));
    assert!(attr.has_validators());
    assert!(!attr.is_modifier());

    let skip: DataFieldKorumaAttr = syn::parse_quote!(skip);
    assert!(!skip.has_validators());
    assert!(skip.is_modifier());

    let nested: DataFieldKorumaAttr = syn::parse_quote!(nested);
    assert!(!nested.has_validators());
    assert!(nested.is_modifier());

    let newtype_only: DataFieldKorumaAttr = syn::parse_quote!(newtype);
    assert!(newtype_only.is_newtype());
    assert!(newtype_only.is_modifier());

    let newtype_with_validators: DataFieldKorumaAttr = syn::parse_quote!(
        newtype,
        each(PositiveValidation),
        RangeValidation.min(0).max(1)
    );
    assert!(newtype_with_validators.is_newtype());
    assert!(newtype_with_validators.has_validators());
    assert_eq!(newtype_with_validators.field_validator_count(), 1);
    assert_eq!(newtype_with_validators.element_validator_count(), 1);
}

#[test]
fn context_specific_koruma_attr_types_parse_normalized_items() {
    let data_attr: DataFieldKorumaAttr =
        syn::parse_quote!(nested, each(PositiveValidation), RangeValidation.min(0));
    assert_eq!(data_attr.items().len(), 3);
    assert!(matches!(
        data_attr.items()[0],
        DataFieldKorumaItem::Modifier(_)
    ));
    assert!(matches!(
        data_attr.items()[1],
        DataFieldKorumaItem::ElementValidation(_)
    ));
    assert!(matches!(
        data_attr.items()[2],
        DataFieldKorumaItem::FieldValidation(_)
    ));

    let struct_attr: StructKorumaAttr = syn::parse_quote!(try_new, newtype, try_from);
    assert_eq!(struct_attr.items().len(), 3);
    assert!(matches!(struct_attr.items()[0], StructKorumaItem::TryNew));
    assert!(matches!(struct_attr.items()[1], StructKorumaItem::Newtype));
    assert!(matches!(struct_attr.items()[2], StructKorumaItem::TryFrom));
}

#[test]
fn parsed_semantic_nodes_keep_actionable_source_markers() {
    let data_attr: DataFieldKorumaAttr = syn::parse_quote!(
        required = full(RequiredValidation::<_>),
        each(item_required = unwrapped(RequiredValidation::<_>))
    );

    let DataFieldKorumaItem::FieldValidation(field_spec) = &data_attr.items()[0] else {
        panic!("expected field validator");
    };
    assert_eq!(
        field_spec.validator().label().map(ToString::to_string),
        Some("required".to_owned())
    );
    assert!(matches!(
        field_spec.validator().target(),
        ValidatorTargetSelector::Full { .. }
    ));

    let DataFieldKorumaItem::ElementValidation(element_spec) = &data_attr.items()[1] else {
        panic!("expected element validator");
    };
    assert_eq!(element_spec.marker_source().value().to_string(), "each");
    assert_eq!(
        element_spec.validators()[0]
            .label()
            .map(ToString::to_string),
        Some("item_required".to_owned())
    );
    assert!(matches!(
        element_spec.validators()[0].target(),
        ValidatorTargetSelector::Unwrapped { .. }
    ));
}

#[test]
fn field_info_and_parse_field_participation_helpers() {
    let field: syn::Field = syn::parse_quote! {
        #[koruma(RangeValidation.min(0).max(10), each(PositiveValidation))]
        value: Vec<i32>
    };
    let info = parse_field_info(&field);
    assert!(info.has_validators());
    assert!(info.has_element_validators());
    assert!(!info.is_nested());
    assert!(!info.is_newtype());
    let validator_names: Vec<_> = info.validator_names().map(ToString::to_string).collect();
    assert_eq!(
        validator_names,
        vec!["RangeValidation", "PositiveValidation"]
    );

    let nested_field: syn::Field = syn::parse_quote! {
        #[koruma(nested)]
        inner: Inner
    };
    assert!(parse_field_info(&nested_field).is_nested());

    let newtype_field: syn::Field = syn::parse_quote! {
        #[koruma(newtype)]
        wrapped: Wrapper
    };
    assert!(parse_field_info(&newtype_field).is_newtype());

    let valid_result = parse_field(&field, 0);
    assert!(matches!(
        valid_result.expect("expected field parse"),
        ParsedDataField::Participating(_)
    ));

    let skip_field: syn::Field = syn::parse_quote! { plain: i32 };
    let skip_result = parse_field(&skip_field, 0);
    assert!(matches!(
        skip_result.expect("expected field parse"),
        ParsedDataField::Unannotated(_)
    ));

    let generic_field: syn::Field = syn::parse_quote! {
        #[koruma(RangeValidation::<_>)]
        broken: i32
    };
    let ParsedDataField::Participating(generic_info) =
        parse_field(&generic_field, 0).expect("expected field parse")
    else {
        panic!("expected parsed field info")
    };
    assert!(
        generic_info.field_validators()[0]
            .validator()
            .uses_type_inference()
    );

    let explicit_field: syn::Field = syn::parse_quote! {
        #[koruma(RangeValidation::<i32>.min(0).max(10))]
        constrained: i32
    };
    let ParsedDataField::Participating(explicit_info) =
        parse_field(&explicit_field, 0).expect("expected field parse")
    else {
        panic!("expected parsed field info")
    };
    assert!(
        explicit_info.field_validators()[0]
            .validator()
            .explicit_type()
            .is_some()
    );

    let valid_result_for_error = parse_field(&field, 0);
    assert!(valid_result_for_error.is_ok());
}

#[test]
fn parse_field_allows_distinct_fully_qualified_validators() {
    let field: syn::Field = syn::parse_quote! {
        #[koruma(foo::RangeValidation.min(0).max(10), bar::RangeValidation.min(11).max(20))]
        value: i32
    };

    let info = parse_field_info(&field);
    let validator_paths: Vec<_> = info
        .field_validators()
        .iter()
        .map(|validator_use| validator_use.validator().path_name())
        .collect();
    assert_eq!(
        validator_paths,
        vec!["foo::RangeValidation", "bar::RangeValidation"]
    );
}

#[test]
fn parse_validator_struct_rejects_missing_marker() {
    let input: syn::ItemStruct = syn::parse_quote! {
        struct Validator {
            #[koruma(setter)]
            checked: i32,
        }
    };
    assert!(
        parse_validator_struct(&input)
            .expect_err("expected missing value field")
            .to_string()
            .contains("requires a value field")
    );

    let tuple_input: syn::ItemStruct = syn::parse_quote! {
        struct TupleValidator(i32);
    };
    assert!(
        parse_validator_struct(&tuple_input)
            .expect_err("expected tuple validator missing value field")
            .to_string()
            .contains("requires a value field")
    );
}

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
fn koruma_attr_newtype_parser_handles_trailing_commas() {
    let with_trailing_commas: DataFieldKorumaAttr = syn::parse_str(
        "newtype, each(RangeValidation.min(0).max(1), PositiveValidation,), RequiredValidation,",
    )
    .expect("newtype parser should accept commas");
    assert!(with_trailing_commas.is_newtype());
    assert_eq!(with_trailing_commas.field_validator_count(), 1);
    assert_eq!(with_trailing_commas.element_validator_count(), 2);

    let plain_with_each: DataFieldKorumaAttr = syn::parse_str(
        "each(RangeValidation.min(0).max(1), PositiveValidation,), RequiredValidation,",
    )
    .expect("plain parser should accept commas");
    assert!(!plain_with_each.is_newtype());
    assert_eq!(plain_with_each.field_validator_count(), 1);
    assert_eq!(plain_with_each.element_validator_count(), 2);
}

#[test]
fn parser_edge_cases_cover_remaining_parse_lines() {
    let empty_attr: Result<DataFieldKorumaAttr, _> = syn::parse_str("");
    assert!(
        empty_attr
            .expect_err("empty koruma attributes should be rejected")
            .to_string()
            .contains("must contain a modifier, validator, or `each(...)` block")
    );

    let chain_with_spaces: ValidatorAttr = syn::parse_str("RangeValidation :: < _ > . min(0)")
        .expect("expected dot validator syntax to parse");
    assert!(chain_with_spaces.uses_type_inference());
    assert_eq!(chain_with_spaces.setter_calls().len(), 1);

    // Parenthesized path arguments branch.
    let parenthesized_path: Result<ValidatorAttr, _> = syn::parse_str("Fn(i32)");
    assert!(parenthesized_path.is_err());

    // `newtype, each(...), ::Path` exercises comma continuation and non-ident validator path
    // in the newtype parser loop.
    let newtype_with_each_and_path: DataFieldKorumaAttr =
        syn::parse_str("newtype, each(::demo::ElemValidation), ::demo::FieldValidation")
            .expect("newtype attr with `each` and absolute path should parse");
    assert!(newtype_with_each_and_path.is_newtype());
    assert_eq!(newtype_with_each_and_path.element_validator_count(), 1);
    assert_eq!(newtype_with_each_and_path.field_validator_count(), 1);

    let newtype_without_comma_falls_through: Result<DataFieldKorumaAttr, _> =
        syn::parse_str("newtype::demo::FieldValidation");
    assert!(
        newtype_without_comma_falls_through
            .expect_err("reserved newtype fallback parsing should be rejected")
            .to_string()
            .contains("reserved koruma field modifier")
    );

    // Non-ident path in the non-newtype parser loop.
    let absolute_path_only: DataFieldKorumaAttr =
        syn::parse_str("::demo::FieldValidation").expect("absolute validator path should parse");
    assert!(!absolute_path_only.is_newtype());
    assert_eq!(absolute_path_only.field_validator_count(), 1);

    let newtype_each_trailing_comma: DataFieldKorumaAttr =
        syn::parse_str("newtype, each(::demo::ElemValidation),")
            .expect("newtype each with trailing comma should parse");
    assert!(newtype_each_trailing_comma.is_newtype());
    assert_eq!(newtype_each_trailing_comma.element_validator_count(), 1);
    assert!(!newtype_each_trailing_comma.has_field_validators());

    let newtype_each_then_field: DataFieldKorumaAttr =
        syn::parse_str("newtype, each(::demo::ElemValidation), ::demo::FieldValidation")
            .expect("newtype each followed by a field validator should parse");
    assert!(newtype_each_then_field.is_newtype());
    assert_eq!(newtype_each_then_field.element_validator_count(), 1);
    assert_eq!(newtype_each_then_field.field_validator_count(), 1);

    let flat_try_from_options = struct_options_from_attrs(&syn::parse_quote! {
        #[koruma(newtype, try_from,)]
        struct Demo(String);
    });
    let StructMode::Newtype { .. } = flat_try_from_options.mode() else {
        panic!("expected newtype mode");
    };
    assert!(flat_try_from_options.constructors().try_from());

    let direct_with_value: Result<ValidatorAttr, _> =
        syn::parse_str("RangeValidation.with_value(1)");
    assert!(
        direct_with_value
            .expect_err("expected with_value syntax rejection")
            .to_string()
            .contains("the value is supplied automatically by the field or collection element")
    );

    let uppercase_constructor: Result<ValidatorAttr, _> = syn::parse_str("RangeValidation::New(1)");
    let err = uppercase_constructor.expect_err("expected uppercase constructor syntax rejection");
    let err_text = err.to_string();
    assert!(
        err_text.contains("requires a direct validator chain")
            || err_text.contains("expected validator chain"),
        "unexpected error: {err_text}"
    );

    let free_function_call: Result<ValidatorAttr, _> = syn::parse_str("min(1)");
    let err = free_function_call.expect_err("expected free function syntax rejection");
    let err_text = err.to_string();
    assert!(
        err_text.contains("requires a direct validator chain")
            || err_text.contains("expected validator chain"),
        "unexpected error: {err_text}"
    );
}

fn struct_options_from_attrs(item: &syn::ItemStruct) -> crate::StructOptions {
    parse_struct_options(&item.attrs).expect("expected struct options to parse")
}

#[test]
fn parse_field_rejects_duplicate_modifiers() {
    let field: syn::Field = syn::parse_quote! {
        #[koruma(newtype, newtype)]
        wrapped: Wrapper
    };

    let err = parse_field(&field, 0).expect_err("expected duplicate modifier rejection");
    assert!(
        err.to_string()
            .contains("duplicate or conflicting field modifier"),
        "expected duplicate modifier rejection, got: {err}",
    );
}

#[test]
fn field_info_has_validators_covers_element_only_branch() {
    let field: syn::Field = syn::parse_quote! {
        #[koruma(each(PositiveValidation))]
        values: Vec<i32>
    };
    let info = parse_field_info(&field);
    assert!(info.field_validators().is_empty());
    assert!(!info.element_validators().is_empty());
    assert!(info.has_validators());
}

#[test]
fn parse_field_rejects_newtype_with_each() {
    let field: syn::Field = syn::parse_quote! {
        #[koruma(newtype, each(PositiveValidation))]
        wrapped: Wrapper
    };

    let err = parse_field(&field, 0).expect_err("expected newtype + each(...) to be rejected");
    assert!(
        err.to_string()
            .contains("cannot also use `each(...)`; validate elements before wrapping"),
        "expected newtype + each rejection, got: {err}",
    );
}

#[test]
fn parse_field_rejects_nested_and_newtype_combination() {
    let field: syn::Field = syn::parse_quote! {
        #[koruma(nested, newtype)]
        wrapped: Wrapper
    };

    let err = parse_field(&field, 0).expect_err("expected nested + newtype to be rejected");
    assert!(
        err.to_string()
            .contains("duplicate or conflicting field modifier"),
        "expected nested + newtype rejection, got: {err}",
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

#[test]
fn parsed_data_field_accessors_and_labels_cover_helpers() {
    let label = ValidatorLabel::new(quote::format_ident!("valid_label"))
        .expect("lower-snake label should be accepted");
    assert_eq!(label.ident().to_string(), "valid_label");

    let validator: ValidatorAttr = syn::parse_quote!(RangeValidation.min(0));
    let unlabeled = crate::ParsedValidatorUse::unlabeled(validator.clone());
    assert!(unlabeled.label().is_none());
    assert!(unlabeled.label_span().is_none());

    let labeled = crate::ParsedValidatorUse::labeled(quote::format_ident!("range"), validator)
        .expect("valid label should parse");
    assert_eq!(
        labeled.label().map(ToString::to_string),
        Some("range".to_owned())
    );
    assert!(labeled.label_span().is_some());

    let attr: DataFieldKorumaAttr =
        syn::parse_quote!(full(RequiredValidation), each(unwrapped(ItemValidation)));
    let DataFieldKorumaItem::FieldValidation(field_spec) = &attr.items()[0] else {
        panic!("expected field validation item");
    };
    assert!(field_spec.validator().target().marker_span().is_some());
    assert!(field_spec.validator().target().is_full());

    let DataFieldKorumaItem::ElementValidation(element_spec) = &attr.items()[1] else {
        panic!("expected element validation item");
    };
    assert_eq!(element_spec.marker().to_string(), "each");
    assert!(
        element_spec.validators()[0]
            .target()
            .marker_span()
            .is_some()
    );
    assert!(!element_spec.validators()[0].target().is_full());

    let skip_attr: DataFieldKorumaAttr = syn::parse_quote!(skip);
    assert!(skip_attr.is_skip());
    assert_eq!(
        skip_attr.items()[0].modifier(),
        Some(FieldModifierKind::Skip)
    );

    let nested_attr: DataFieldKorumaAttr = syn::parse_quote!(nested);
    assert!(nested_attr.is_nested());
    assert_eq!(
        nested_attr.items()[0].modifier(),
        Some(FieldModifierKind::Nested)
    );

    let field: syn::Field = syn::parse_quote! {
        #[koruma(skip)]
        skipped: String
    };
    let skipped = parse_field(&field, 3).expect("expected skip field to parse");
    assert_eq!(skipped.source().name().to_string(), "skipped");
    assert!(skipped.participating().is_none());

    let field: syn::Field = syn::parse_quote! {
        #[koruma(RangeValidation)]
        value: i32
    };
    let parsed = parse_field(&field, 4).expect("expected field to parse");
    assert_eq!(parsed.source().index(), 4);
    let participating = parsed.participating().expect("field should participate");
    assert_eq!(participating.source().name().to_string(), "value");
    assert_eq!(participating.index(), 4);
}

#[test]
fn data_field_parser_reports_target_and_each_edge_cases() {
    for (source, expected) in [
        ("full()", "must contain exactly one validator"),
        ("unwrapped()", "must contain exactly one validator"),
        (
            "full(RequiredValidation, OtherValidation)",
            "accepts exactly one validator",
        ),
        ("full", "reserved koruma target selector"),
        ("each", "`each` is only valid as `each(...)`"),
        ("each()", "`each(...)` must contain at least one validator"),
        (
            "skip(RangeValidation)",
            "parenthesized `skip` is not valid in a derive data field",
        ),
    ] {
        let err = match syn::parse_str::<DataFieldKorumaAttr>(source) {
            Ok(_) => panic!("expected `{source}` to fail"),
            Err(err) => err,
        };
        assert!(
            err.to_string().contains(expected),
            "expected `{expected}` for `{source}`, got: {err}",
        );
    }

    let field: syn::Field = syn::parse_quote! {
        #[koruma(skip, RequiredValidation)]
        skipped: String
    };
    let err = parse_field(&field, 0).expect_err("skip plus validator should fail");
    assert!(
        err.to_string()
            .contains("fields marked `#[koruma(skip)]` cannot also use validators"),
        "unexpected error: {err}",
    );

    let field: syn::Field = syn::parse_quote! {
        #[koruma(nested, RequiredValidation)]
        nested: String
    };
    let err = parse_field(&field, 0).expect_err("nested plus validator should fail");
    assert!(
        err.to_string()
            .contains("fields marked `#[koruma(nested)]` cannot also use validators"),
        "unexpected error: {err}",
    );
}

#[test]
fn struct_options_cover_constructor_combinations_and_errors() {
    let regular_try_new: crate::StructOptions =
        syn::parse_str("try_new").expect("regular try_new should parse");
    let StructMode::Regular = regular_try_new.mode() else {
        panic!("expected regular mode");
    };
    assert!(regular_try_new.constructors().try_new());
    assert!(!regular_try_new.constructors().try_from());

    let regular_try_from: crate::StructOptions =
        syn::parse_str("try_from").expect("regular try_from should parse");
    let StructMode::Regular = regular_try_from.mode() else {
        panic!("expected regular mode");
    };
    assert!(!regular_try_from.constructors().try_new());
    assert!(regular_try_from.constructors().try_from());

    let newtype_try_new: crate::StructOptions =
        syn::parse_str("try_new, newtype").expect("newtype try_new should parse");
    let StructMode::Newtype { marker } = newtype_try_new.mode() else {
        panic!("expected newtype mode");
    };
    assert_eq!(marker.value().to_string(), "newtype");
    assert!(newtype_try_new.constructors().try_new());
    assert!(!newtype_try_new.constructors().try_from());

    let newtype_try_from: crate::StructOptions =
        syn::parse_str("newtype, try_from").expect("newtype try_from should parse");
    let StructMode::Newtype { .. } = newtype_try_from.mode() else {
        panic!("expected newtype mode");
    };
    assert!(!newtype_try_from.constructors().try_new());
    assert!(newtype_try_from.constructors().try_from());

    let newtype_both: crate::StructOptions =
        syn::parse_str("try_new, newtype, try_from").expect("combined newtype should parse");
    let StructMode::Newtype { .. } = newtype_both.mode() else {
        panic!("expected newtype mode");
    };
    assert!(newtype_both.constructors().try_new());
    assert!(newtype_both.constructors().try_from());

    for (source, expected) in [
        (
            "newtype()",
            "parenthesized `newtype` options are unsupported",
        ),
        (
            "try_from, try_from",
            "duplicate struct-level koruma option `try_from`",
        ),
        ("try_new()", "only valid as a bare struct-level"),
        ("try_from()", "only valid as a bare struct-level"),
        ("try_new::path", "only valid as a bare struct-level"),
        ("try_from::path", "only valid as a bare struct-level"),
        ("newtype::path", "reserved koruma struct option"),
        ("skip", "not valid in a derive struct"),
        ("unknown", "unknown struct-level koruma option"),
        (
            "try_new, try_new",
            "duplicate struct-level koruma option `try_new`",
        ),
        (
            "newtype, newtype",
            "duplicate struct-level koruma option `newtype`",
        ),
    ] {
        let err = match syn::parse_str::<crate::StructOptions>(source) {
            Ok(_) => panic!("expected `{source}` to fail"),
            Err(err) => err,
        };
        assert!(
            err.to_string().contains(expected),
            "expected `{expected}` for `{source}`, got: {err}",
        );
    }
}

#[test]
fn validator_struct_parser_covers_setter_merge_paths_and_accessors() {
    let input: syn::ItemStruct = syn::parse_quote! {
        struct Validator {
            #[koruma(setter(into))]
            title: String,
            #[koruma(setter(required))]
            count: usize,
            #[koruma(setter(default = 42))]
            limit: usize,
            #[koruma(skip_capture)]
            actual: Option<String>,
        }
    };
    let spec = parse_validator_struct(&input).expect("expected validator struct to parse");
    assert_eq!(spec.value_index(), 3);
    assert_eq!(spec.value_spec().capture(), crate::CapturePolicy::Skip);

    let title = &spec.fields()[0];
    assert_eq!(title.name().to_string(), "title");
    let title_ty = title.ty();
    assert_eq!(quote::quote!(#title_ty).to_string(), "String");
    let ValidatorFieldRole::Setter(title_setter) = title.role() else {
        panic!("expected setter role");
    };
    assert_eq!(title_setter.method().to_string(), "title");
    assert!(title_setter.input().accepts_into());
    assert!(matches!(title_setter.presence(), SetterPresence::Optional));

    let count = &spec.fields()[1];
    let ValidatorFieldRole::Setter(count_setter) = count.role() else {
        panic!("expected setter role");
    };
    assert!(matches!(count_setter.presence(), SetterPresence::Required));

    let limit = &spec.fields()[2];
    let ValidatorFieldRole::Setter(limit_setter) = limit.role() else {
        panic!("expected setter role");
    };
    let SetterPresence::Defaulted(SetterDefault::Expr(expr)) = limit_setter.presence() else {
        panic!("expected expression default");
    };
    assert_eq!(quote::quote!(#expr).to_string(), "42");

    for (input, expected) in [
        (
            syn::parse_quote! {
                struct Bad {
                    #[koruma(setter(into), setter(into))]
                    title: String,
                    #[koruma(value)]
                    actual: Option<String>,
                }
            },
            "duplicate `setter(into)` option",
        ),
        (
            syn::parse_quote! {
                struct Bad {
                    #[koruma(setter(name = first), setter(name = second))]
                    title: String,
                    #[koruma(value)]
                    actual: Option<String>,
                }
            },
            "duplicate `setter(name = ...)` option",
        ),
        (
            syn::parse_quote! {
                struct Bad {
                    #[koruma(setter(required), setter(required))]
                    title: String,
                    #[koruma(value)]
                    actual: Option<String>,
                }
            },
            "duplicate `setter(required)` option",
        ),
        (
            syn::parse_quote! {
                struct Bad {
                    #[koruma(setter(default), setter(default = 1))]
                    title: String,
                    #[koruma(value)]
                    actual: Option<String>,
                }
            },
            "duplicate `setter(default)` option",
        ),
    ] {
        let err = parse_validator_struct(&input).expect_err("expected duplicate merge error");
        assert!(
            err.to_string().contains(expected),
            "expected `{expected}`, got: {err}",
        );
    }
}

#[test]
fn validator_struct_value_and_setter_parser_errors_cover_remaining_branches() {
    let input: syn::ItemStruct = syn::parse_quote! {
        struct Bad {
            #[koruma(value())]
            actual: Option<String>,
        }
    };
    let err = parse_validator_struct(&input).expect_err("parenthesized value marker should fail");
    assert!(
        err.to_string()
            .contains("parenthesized `value` markers are unsupported; use `skip_capture`"),
        "unexpected error: {err}",
    );

    let input: syn::ItemStruct = syn::parse_quote! {
        struct Bad {
            #[koruma(setter(into, into))]
            title: String,
            #[koruma(value)]
            actual: Option<String>,
        }
    };
    let err = parse_validator_struct(&input).expect_err("duplicate parser option should fail");
    assert!(
        err.to_string().contains("duplicate `setter(into)` option"),
        "unexpected error: {err}",
    );
}

#[test]
fn validator_attr_parsing_covers_grouped_calls_and_accessors() {
    let grouped: ValidatorAttr = syn::parse_str("(RangeValidation::<i32>.min(0).max(10))")
        .expect("grouped validator chains should parse");
    assert_eq!(grouped.path_name(), "RangeValidation");
    assert_eq!(grouped.path().segments.len(), 1);
    let explicit_type = grouped.explicit_type().expect("expected explicit type");
    assert_eq!(quote::quote!(#explicit_type).to_string(), "i32");
    let first_arg = grouped.setter_calls()[0].args()[0].as_expr();
    assert_eq!(quote::quote!(#first_arg).to_string(), "0");

    let direct_call: Result<ValidatorAttr, _> = syn::parse_str("(RangeValidation::min)(0)");
    assert!(
        direct_call
            .expect_err("expected grouped associated setter starters to be rejected")
            .to_string()
            .contains("expected validator chain")
    );

    for (source, expected) in [
        ("", "validator syntax requires a dot validator chain"),
        ("make().min(1)", "expected validator chain"),
        ("(make())(1)", "expected validator chain"),
    ] {
        let err = match syn::parse_str::<ValidatorAttr>(source) {
            Ok(_) => panic!("expected `{source}` to fail"),
            Err(err) => err,
        };
        assert!(
            err.to_string().contains(expected),
            "unexpected error for `{source}`: {err}",
        );
    }
}

#[test]
fn struct_options_reject_repeated_attrs() {
    let duplicate_newtype: syn::ItemStruct = syn::parse_quote! {
        #[koruma(newtype)]
        #[koruma(newtype)]
        struct Demo(String);
    };
    assert!(
        parse_struct_options(&duplicate_newtype.attrs)
            .expect_err("expected repeated attribute error")
            .to_string()
            .contains("only one struct-level `#[koruma(...)]` attribute is allowed")
    );

    let duplicate_try_from: syn::ItemStruct = syn::parse_quote! {
        #[koruma(newtype, try_from)]
        #[koruma(newtype, try_from)]
        struct Demo(String);
    };
    assert!(
        parse_struct_options(&duplicate_try_from.attrs)
            .expect_err("expected repeated attribute error")
            .to_string()
            .contains("only one struct-level `#[koruma(...)]` attribute is allowed")
    );
}

#[test]
fn value_field_info_wrappers_and_empty_marker_errors_are_covered() {
    let input: syn::ItemStruct = syn::parse_quote! {
        struct Validator {
            #[koruma(value)]
            actual: String,
        }
    };

    let spec = parse_validator_struct(&input).expect("expected valid validator struct");
    assert_eq!(spec.value_field().name().to_string(), "actual");

    let bad_input: syn::ItemStruct = syn::parse_quote! {
        struct Validator {
            #[koruma()]
            actual: String,
        }
    };

    assert!(
        parse_validator_struct(&bad_input)
            .expect_err("expected empty marker error")
            .to_string()
            .contains(
                "validator fields must contain `value`, `skip_capture`, `setter`, or `setter(...)`",
            )
    );
}

#[cfg(feature = "internal-showcase")]
#[test]
fn showcase_attr_errors_are_reported() {
    use crate::{ShowcaseAttr, ShowcaseInputType, ShowcaseModule, find_showcase_attr};

    let unknown: Result<ShowcaseAttr, _> = syn::parse_str(
        r#"name = "n", description = "d", create = |input: &str| input, nope = "x""#,
    );
    assert!(
        unknown
            .expect_err("expected parse error")
            .to_string()
            .contains("unknown showcase attribute")
    );

    let missing_description: Result<ShowcaseAttr, _> =
        syn::parse_str(r#"name = "n", create = |input: &str| input, input_type = Text"#);
    assert!(
        missing_description
            .expect_err("expected parse error")
            .to_string()
            .contains("showcase requires `description` attribute")
    );

    let missing_input_type: Result<ShowcaseAttr, _> =
        syn::parse_str(r#"name = "n", description = "d", create = |input: &str| input"#);
    assert!(
        missing_input_type
            .expect_err("expected parse error")
            .to_string()
            .contains("showcase requires `input_type` attribute")
    );

    let invalid_input_type: Result<ShowcaseAttr, _> = syn::parse_str(
        r#"name = "n", description = "d", create = |input: &str| input, input_type = Boolean"#,
    );
    assert!(
        invalid_input_type
            .expect_err("expected parse error")
            .to_string()
            .contains("showcase `input_type` must be `Text` or `Numeric`")
    );

    let input: syn::ItemStruct = syn::parse_quote! {
        #[showcase(name = "N", description = "D", create = |input: &str| input, input_type = Text)]
        struct Demo;
    };
    let parsed = find_showcase_attr(&input)
        .expect("valid showcase attr")
        .expect("showcase attr should be present");
    assert_eq!(parsed.input_type, ShowcaseInputType::Text);
    assert_eq!(parsed.module, None);

    let input_with_module: syn::ItemStruct = syn::parse_quote! {
        #[showcase(name = "N", description = "D", create = |input: &str| input, input_type = Numeric, module = "format")]
        struct DemoWithModule;
    };
    let parsed = find_showcase_attr(&input_with_module)
        .expect("valid showcase attr with module")
        .expect("showcase attr with module should be present");
    assert_eq!(parsed.input_type, ShowcaseInputType::Numeric);
    assert_eq!(parsed.module, Some(ShowcaseModule::Format));

    for (module_name, expected_module) in [
        ("string", ShowcaseModule::String),
        ("numeric", ShowcaseModule::Numeric),
        ("collection", ShowcaseModule::Collection),
        ("general", ShowcaseModule::General),
    ] {
        let attr: ShowcaseAttr = syn::parse_str(&format!(
            r#"name = "N", description = "D", create = |input: &str| input, input_type = Text, module = "{module_name}""#
        ))
        .expect("showcase module should parse");
        assert_eq!(attr.input_type, ShowcaseInputType::Text);
        assert_eq!(attr.module, Some(expected_module));
    }

    for (source, expected) in [
        (
            r#"description = "d", create = |input: &str| input, input_type = Text"#,
            "showcase requires `name` attribute",
        ),
        (
            r#"name = "n", description = "d", input_type = Text"#,
            "showcase requires `create` attribute",
        ),
    ] {
        let err = syn::parse_str::<ShowcaseAttr>(source)
            .expect_err("expected missing showcase field error");
        assert!(
            err.to_string().contains(expected),
            "expected `{expected}`, got: {err}",
        );
    }

    let invalid_input: syn::ItemStruct = syn::parse_quote! {
        #[showcase(name = "N", description = "D", create = |input: &str| input, input_type = Text, modul = "oops")]
        struct BadDemo;
    };
    assert!(
        find_showcase_attr(&invalid_input)
            .expect_err("expected showcase attr parse error")
            .to_string()
            .contains("unknown showcase attribute: modul")
    );

    let invalid_module: syn::ItemStruct = syn::parse_quote! {
        #[showcase(name = "N", description = "D", create = |input: &str| input, input_type = Text, module = "oops")]
        struct BadDemoModule;
    };
    assert!(
        find_showcase_attr(&invalid_module)
            .expect_err("expected showcase attr parse error for module")
            .to_string()
            .contains("showcase `module` must be one of")
    );
}
