use crate::{
    FieldInfo, KorumaAttr, ParseFieldResult, ValidatorAttr, contains_infer_type,
    expr_as_simple_ident, find_value_field, find_value_field_info, find_value_field_info_strict,
    first_generic_arg, is_option_infer_type, option_inner_type, parse_field, parse_struct_options,
    substitute_infer_type, substitute_infer_type_from_source, type_to_ident, vec_inner_type,
};

fn parse_field_info(field: &syn::Field) -> FieldInfo {
    match parse_field(field, 0) {
        ParseFieldResult::Valid(info) => *info,
        other => panic!("expected ParseFieldResult::Valid, got {other:?}"),
    }
}

#[test]
fn validator_attr_helpers_and_error_paths() {
    let plain: ValidatorAttr = syn::parse_quote!(RangeValidation);
    assert!(!plain.has_args());
    assert!(!plain.uses_type_inference());
    assert!(!plain.has_explicit_type());

    let with_builder_methods: ValidatorAttr = syn::parse_quote!(RangeValidation::min(0).max(10));
    assert!(with_builder_methods.has_args());
    let builder_calls = with_builder_methods.setter_calls();
    assert_eq!(builder_calls.len(), 2);
    assert_eq!(builder_calls[0].method.to_string(), "min");
    assert_eq!(builder_calls[1].method.to_string(), "max");

    let infer: ValidatorAttr = syn::parse_quote!(GenericValidation::<_>);
    assert!(infer.uses_type_inference());
    assert!(!infer.has_explicit_type());

    let explicit: ValidatorAttr = syn::parse_quote!(GenericValidation::<i32>);
    assert!(!explicit.uses_type_inference());
    assert!(explicit.has_explicit_type());

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

    let removed_shorthand: Result<ValidatorAttr, _> = syn::parse_str("GenericValidation<_>");
    assert!(
        removed_shorthand
            .expect_err("expected shorthand syntax to be rejected")
            .to_string()
            .contains("requires a direct validator chain")
    );

    let builder_with_build: Result<ValidatorAttr, _> =
        syn::parse_str("GenericValidation::min(1).build()");
    assert!(
        builder_with_build
            .expect_err("expected validator chains to reject .build()")
            .to_string()
            .contains("injects builder creation, value capture, and `.build()` automatically")
    );

    let parenthesized_path: Result<ValidatorAttr, _> = syn::parse_str("std::ops::Fn(i32)");
    assert!(parenthesized_path.is_err());

    let direct_parenthesized: Result<ValidatorAttr, _> = syn::parse_str("Fn(i32)");
    assert!(direct_parenthesized.is_err());
}

#[test]
fn koruma_attr_helpers_and_newtype_parsing_paths() {
    let attr: KorumaAttr =
        syn::parse_quote!(RangeValidation::min(0).max(10), each(PositiveValidation));
    assert!(attr.has_validators());
    assert!(!attr.is_modifier());

    let skip: KorumaAttr = syn::parse_quote!(skip);
    assert!(!skip.has_validators());
    assert!(skip.is_modifier());

    let nested: KorumaAttr = syn::parse_quote!(nested);
    assert!(!nested.has_validators());
    assert!(nested.is_modifier());

    let newtype_only: KorumaAttr = syn::parse_quote!(newtype);
    assert!(newtype_only.is_newtype);
    assert!(newtype_only.is_modifier());

    let newtype_with_validators: KorumaAttr = syn::parse_quote!(
        newtype,
        each(PositiveValidation),
        RangeValidation::min(0).max(1)
    );
    assert!(newtype_with_validators.is_newtype);
    assert!(newtype_with_validators.has_validators());
    assert_eq!(newtype_with_validators.field_validators.len(), 1);
    assert_eq!(newtype_with_validators.element_validators.len(), 1);
}

#[test]
fn field_info_and_parse_field_result_helpers() {
    let field: syn::Field = syn::parse_quote! {
        #[koruma(RangeValidation::min(0).max(10), each(PositiveValidation))]
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
    assert!(valid_result.is_valid());
    assert!(valid_result.valid().is_some());

    let skip_field: syn::Field = syn::parse_quote! { plain: i32 };
    let skip_result = parse_field(&skip_field, 0);
    assert!(skip_result.is_skip());
    assert!(skip_result.valid().is_none());

    let generic_field: syn::Field = syn::parse_quote! {
        #[koruma(RangeValidation::<_>)]
        broken: i32
    };
    let generic_result = parse_field(&generic_field, 0);
    assert!(generic_result.is_valid());
    let generic_info = generic_result.valid().expect("expected parsed field info");
    assert!(generic_info.validation.field_validators[0].infer_type);

    let explicit_field: syn::Field = syn::parse_quote! {
        #[koruma(RangeValidation::<i32>::min(0).max(10))]
        constrained: i32
    };
    let explicit_result = parse_field(&explicit_field, 0);
    assert!(explicit_result.is_valid());
    let explicit_info = explicit_result.valid().expect("expected parsed field info");
    assert!(
        explicit_info.validation.field_validators[0]
            .explicit_type
            .is_some()
    );

    let valid_result_for_error = parse_field(&field, 0);
    assert!(valid_result_for_error.error().is_none());
}

#[test]
fn parse_field_allows_distinct_fully_qualified_validators() {
    let field: syn::Field = syn::parse_quote! {
        #[koruma(foo::RangeValidation::min(0).max(10), bar::RangeValidation::min(11).max(20))]
        value: i32
    };

    let info = parse_field_info(&field);
    let validator_paths: Vec<_> = info
        .validation
        .field_validators
        .iter()
        .map(ValidatorAttr::path_name)
        .collect();
    assert_eq!(
        validator_paths,
        vec!["foo::RangeValidation", "bar::RangeValidation"]
    );
}

#[test]
fn find_value_field_returns_none_without_marker() {
    let input: syn::ItemStruct = syn::parse_quote! {
        struct Validator {
            actual: i32,
        }
    };
    assert!(find_value_field(&input).is_none());

    let tuple_input: syn::ItemStruct = syn::parse_quote! {
        struct TupleValidator(i32);
    };
    assert!(find_value_field(&tuple_input).is_none());
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

    let option_concrete: syn::Type = syn::parse_quote!(Option<String>);
    assert!(!is_option_infer_type(&option_concrete));
    let option_infer: syn::Type = syn::parse_quote!(Option<_>);
    assert!(is_option_infer_type(&option_infer));

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

    let named_type: syn::Type = syn::parse_quote!(Age);
    assert_eq!(
        type_to_ident(&named_type).map(|ident| ident.to_string()),
        Some("Age".to_string())
    );
}

#[test]
fn koruma_attr_newtype_parser_handles_trailing_commas() {
    let with_trailing_commas: KorumaAttr = syn::parse_str(
        "newtype, each(RangeValidation::min(0).max(1), PositiveValidation,), RequiredValidation,",
    )
    .expect("newtype parser should accept commas");
    assert!(with_trailing_commas.is_newtype);
    assert_eq!(with_trailing_commas.field_validators.len(), 1);
    assert_eq!(with_trailing_commas.element_validators.len(), 2);

    let plain_with_each: KorumaAttr = syn::parse_str(
        "each(RangeValidation::min(0).max(1), PositiveValidation,), RequiredValidation,",
    )
    .expect("plain parser should accept commas");
    assert!(!plain_with_each.is_newtype);
    assert_eq!(plain_with_each.field_validators.len(), 1);
    assert_eq!(plain_with_each.element_validators.len(), 2);
}

#[test]
fn parser_edge_cases_cover_remaining_parse_lines() {
    let chain_with_spaces: ValidatorAttr = syn::parse_str("RangeValidation :: < _ > :: min(0)")
        .expect("expected direct validator syntax to parse");
    assert!(chain_with_spaces.infer_type);
    assert_eq!(chain_with_spaces.setter_calls().len(), 1);

    // Parenthesized path arguments branch.
    let parenthesized_path: Result<ValidatorAttr, _> = syn::parse_str("Fn(i32)");
    assert!(parenthesized_path.is_err());

    // `newtype, each(...), ::Path` exercises comma continuation and non-ident validator path
    // in the newtype parser loop.
    let newtype_with_each_and_path: KorumaAttr =
        syn::parse_str("newtype, each(::demo::ElemValidation), ::demo::FieldValidation")
            .expect("newtype attr with `each` and absolute path should parse");
    assert!(newtype_with_each_and_path.is_newtype);
    assert_eq!(newtype_with_each_and_path.element_validators.len(), 1);
    assert_eq!(newtype_with_each_and_path.field_validators.len(), 1);

    // `newtype` followed by a path without a comma currently falls back to regular parsing.
    let newtype_without_comma_falls_through: KorumaAttr =
        syn::parse_str("newtype::demo::FieldValidation")
            .expect("fallback parser should still parse remaining validator path");
    assert!(!newtype_without_comma_falls_through.is_newtype);
    assert_eq!(
        newtype_without_comma_falls_through.field_validators.len(),
        1
    );

    // Non-ident path in the non-newtype parser loop.
    let absolute_path_only: KorumaAttr =
        syn::parse_str("::demo::FieldValidation").expect("absolute validator path should parse");
    assert!(!absolute_path_only.is_newtype);
    assert_eq!(absolute_path_only.field_validators.len(), 1);

    let newtype_each_trailing_comma: KorumaAttr =
        syn::parse_str("newtype, each(::demo::ElemValidation),")
            .expect("newtype each with trailing comma should parse");
    assert!(newtype_each_trailing_comma.is_newtype);
    assert_eq!(newtype_each_trailing_comma.element_validators.len(), 1);
    assert!(newtype_each_trailing_comma.field_validators.is_empty());

    let newtype_each_then_field: KorumaAttr =
        syn::parse_str("newtype, each(::demo::ElemValidation), ::demo::FieldValidation")
            .expect("newtype each followed by a field validator should parse");
    assert!(newtype_each_then_field.is_newtype);
    assert_eq!(newtype_each_then_field.element_validators.len(), 1);
    assert_eq!(newtype_each_then_field.field_validators.len(), 1);

    let newtype_options_with_trailing_comma = struct_options_from_attrs(&syn::parse_quote! {
        #[koruma(newtype(try_from,))]
        struct Demo(String);
    });
    assert!(newtype_options_with_trailing_comma.newtype);
    assert!(newtype_options_with_trailing_comma.try_from);

    let legacy_builder: Result<ValidatorAttr, _> = syn::parse_str("RangeValidation::builder()");
    assert!(
        legacy_builder
            .expect_err("expected legacy builder syntax rejection")
            .to_string()
            .contains("legacy validator `::builder()` syntax is not supported")
    );

    let direct_with_value: Result<ValidatorAttr, _> =
        syn::parse_str("RangeValidation::with_value(1)");
    assert!(
        direct_with_value
            .expect_err("expected with_value syntax rejection")
            .to_string()
            .contains("chains should stop before `.with_value(...)`")
    );

    let uppercase_constructor: Result<ValidatorAttr, _> = syn::parse_str("RangeValidation::New(1)");
    assert!(
        uppercase_constructor
            .expect_err("expected uppercase constructor syntax rejection")
            .to_string()
            .contains("requires a direct validator chain")
    );

    let free_function_call: Result<ValidatorAttr, _> = syn::parse_str("min(1)");
    assert!(
        free_function_call
            .expect_err("expected free function syntax rejection")
            .to_string()
            .contains("requires a direct validator chain")
    );
}

fn struct_options_from_attrs(item: &syn::ItemStruct) -> crate::StructOptions {
    parse_struct_options(&item.attrs).expect("expected struct options to parse")
}

#[test]
fn field_info_has_validators_covers_element_only_branch() {
    let field: syn::Field = syn::parse_quote! {
        #[koruma(each(PositiveValidation))]
        values: Vec<i32>
    };
    let info = parse_field_info(&field);
    assert!(info.validation.field_validators.is_empty());
    assert!(!info.validation.element_validators.is_empty());
    assert!(info.has_validators());
}

#[test]
fn parse_field_rejects_newtype_with_each_across_attributes() {
    let field: syn::Field = syn::parse_quote! {
        #[koruma(newtype)]
        #[koruma(each(PositiveValidation))]
        wrapped: Wrapper
    };

    let err = parse_field(&field, 0)
        .error()
        .expect("expected newtype + each(...) to be rejected");
    assert!(
        err.to_string()
            .contains("cannot also use `each(...)`; element validation is not supported"),
        "expected newtype + each rejection, got: {err}",
    );
}

#[test]
fn parse_field_rejects_nested_and_newtype_combination() {
    let field: syn::Field = syn::parse_quote! {
        #[koruma(nested)]
        #[koruma(newtype)]
        wrapped: Wrapper
    };

    let err = parse_field(&field, 0)
        .error()
        .expect("expected nested + newtype to be rejected");
    assert!(
        err.to_string()
            .contains("cannot combine `#[koruma(nested)]` and `#[koruma(newtype)]`"),
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

    let option_concrete: syn::Type = syn::parse_quote!(Option<u32>);
    assert!(!is_option_infer_type(&option_concrete));

    let option_infer: syn::Type = syn::parse_quote!(Option<_>);
    assert!(is_option_infer_type(&option_infer));

    // Option with lifetime arg only - exercises the for loop without Type match
    let option_lifetime: syn::Type = syn::parse_quote!(Option<'static>);
    assert!(!is_option_infer_type(&option_lifetime));

    let ty_with_lifetime_and_infer: syn::Type = syn::parse_quote!(Wrapper<'static, _>);
    let infer_target: syn::Type = syn::parse_quote!(usize);
    let substituted = substitute_infer_type(&ty_with_lifetime_and_infer, &infer_target);
    assert_eq!(
        quote::quote!(#substituted).to_string(),
        "Wrapper < 'static , usize >"
    );
}

#[test]
fn struct_options_report_duplicate_newtype_and_try_from() {
    let duplicate_newtype: syn::ItemStruct = syn::parse_quote! {
        #[koruma(newtype)]
        #[koruma(newtype)]
        struct Demo(String);
    };
    assert!(
        parse_struct_options(&duplicate_newtype.attrs)
            .expect_err("expected duplicate newtype error")
            .to_string()
            .contains("duplicate struct-level koruma option `newtype`")
    );

    let duplicate_try_from: syn::ItemStruct = syn::parse_quote! {
        #[koruma(newtype(try_from))]
        #[koruma(newtype(try_from))]
        struct Demo(String);
    };
    assert!(
        parse_struct_options(&duplicate_try_from.attrs)
            .expect_err("expected duplicate newtype error")
            .to_string()
            .contains("duplicate struct-level koruma option `newtype`")
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

    let info = find_value_field_info(&input).expect("expected value field info");
    assert_eq!(info.name.to_string(), "actual");

    let bad_input: syn::ItemStruct = syn::parse_quote! {
        struct Validator {
            #[koruma()]
            actual: String,
        }
    };

    assert!(
        find_value_field_info_strict(&bad_input)
            .expect_err("expected empty marker error")
            .to_string()
            .contains("validator fields only support")
    );
    assert!(find_value_field_info(&bad_input).is_none());
}

#[cfg(feature = "internal-showcase")]
#[test]
fn showcase_attr_errors_are_reported() {
    use crate::{ShowcaseAttr, find_showcase_attr};

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
    assert!(
        find_showcase_attr(&input)
            .expect("valid showcase attr")
            .is_some()
    );

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
}
