use super::support::*;

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
