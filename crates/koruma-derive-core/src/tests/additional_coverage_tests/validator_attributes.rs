use super::support::*;

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
