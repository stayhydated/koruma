#[test]
fn test_koruma_expansion_single_validator() {
    let input: DeriveInput = syn::parse_quote! {
        pub struct Item {
            #[koruma(NumberRangeValidation.min(0).max(100))]
            pub age: i32,
        }
    };

    let expanded = expand_koruma(input).unwrap();
    assert_snapshot!(pretty_print(expanded));
}

#[test]
fn test_koruma_expansion_single_builder_chain_validator() {
    let input: DeriveInput = syn::parse_quote! {
        pub struct Item {
            #[koruma(NumberRangeValidation::<_>.min(0).max(100))]
            pub age: i32,
        }
    };

    let expanded = expand_koruma(input).unwrap();
    assert_snapshot!(pretty_print(expanded));
}

#[test]
fn test_koruma_expansion_completion_probe_uses_ra_marker() {
    let input: DeriveInput = syn::parse_quote! {
        pub struct CompletionProbe {
            #[koruma(StringLengthValidation.)]
            pub name: String,

            #[koruma(NumberRangeValidation::<_>.)]
            pub age: i32,
        }
    };

    let expanded = expand_koruma(input).unwrap();
    let compact = compact_ws(&pretty_print(expanded));
    assert!(compact.contains("StringLengthValidation::__koruma_builder()"));
    assert!(compact.contains("NumberRangeValidation::<i32>::__koruma_builder()"));
    assert!(compact.contains("let_=__koruma_builder.raCompletionMarker();"));
    assert!(!compact.contains("__koruma_ra_completion_marker"));
}

#[test]
fn test_koruma_expansion_multiple_validators() {
    let input: DeriveInput = syn::parse_quote! {
        pub struct MultiValidatorItem {
            #[koruma(NumberRangeValidation.min(0).max(100), EvenNumberValidation)]
            pub value: i32,
        }
    };

    let expanded = expand_koruma(input).unwrap();
    assert_snapshot!(pretty_print(expanded));
}

#[test]
fn test_koruma_expansion_labeled_field_validators() {
    let input: DeriveInput = syn::parse_quote! {
        pub struct LabeledItem {
            #[koruma(
                min_len = LengthValidation::<_>.min(3),
                max_len = LengthValidation::<_>.max(30),
            )]
            pub value: String,
        }
    };

    let expanded = expand_koruma(input).unwrap();
    let printed = pretty_print(expanded);
    assert!(printed.contains("pub fn min_len"));
    assert!(printed.contains("pub fn max_len"));
    assert!(printed.contains("MinLen"));
    assert!(printed.contains("MaxLen"));
    assert_snapshot!(printed);
}

#[test]
fn test_koruma_expansion_labeled_element_validators() {
    let input: DeriveInput = syn::parse_quote! {
        pub struct LabeledTags {
            #[koruma(each(
                tag_prefix = PrefixValidation::<_>.prefix("tag:"),
            ))]
            pub tags: Vec<String>,
        }
    };

    let expanded = expand_koruma(input).unwrap();
    let printed = pretty_print(expanded);
    assert!(printed.contains("pub fn tag_prefix"));
    assert!(printed.contains("TagPrefix"));
    assert_snapshot!(printed);
}

#[test]
fn test_koruma_expansion_generic_validator() {
    let input: DeriveInput = syn::parse_quote! {
        pub struct GenericItem {
            #[koruma(GenericRangeValidation::<_>.min(0.0).max(100.0))]
            pub score: f64,
        }
    };

    let expanded = expand_koruma(input).unwrap();
    assert_snapshot!(pretty_print(expanded));
}

#[test]
fn test_koruma_expansion_each() {
    let input: DeriveInput = syn::parse_quote! {
        pub struct Order {
            #[koruma(each(GenericRangeValidation::<_>.min(0.0).max(100.0)))]
            pub scores: Vec<f64>,
        }
    };

    let expanded = expand_koruma(input).unwrap();
    assert_snapshot!(pretty_print(expanded));
}

#[test]
fn test_koruma_expansion_multiple_fields() {
    let input: DeriveInput = syn::parse_quote! {
        pub struct Item {
            #[koruma(NumberRangeValidation.min(0).max(100))]
            pub age: i32,

            #[koruma(StringLengthValidation.min(1).max(67))]
            pub name: String,
        }
    };

    let expanded = expand_koruma(input).unwrap();
    assert_snapshot!(pretty_print(expanded));
}


#[test]
fn test_koruma_expansion_validator_no_args() {
    // Validator with no arguments (like EvenNumberValidation)
    let input: DeriveInput = syn::parse_quote! {
        pub struct Item {
            #[koruma(EvenNumberValidation)]
            pub value: i32,
        }
    };

    let expanded = expand_koruma(input).unwrap();
    assert_snapshot!(pretty_print(expanded));
}

#[test]
fn test_koruma_expansion_each_multiple_validators() {
    // each() with multiple validators
    let input: DeriveInput = syn::parse_quote! {
        pub struct Order {
            #[koruma(each(RangeValidation.min(0).max(100), EvenValidation))]
            pub values: Vec<i32>,
        }
    };

    let expanded = expand_koruma(input).unwrap();
    assert_snapshot!(pretty_print(expanded));
}

#[test]
fn test_koruma_expansion_mixed_fields() {
    // Mix of regular field, each field, and multiple validators
    let input: DeriveInput = syn::parse_quote! {
        pub struct ComplexItem {
            #[koruma(RangeValidation.min(0).max(100))]
            pub age: i32,

            #[koruma(each(LengthValidation.min(1).max(50)))]
            pub tags: Vec<String>,

            #[koruma(RangeValidation.min(0).max(10), EvenValidation)]
            pub rating: i32,
        }
    };

    let expanded = expand_koruma(input).unwrap();
    assert_snapshot!(pretty_print(expanded));
}


#[test]
fn test_koruma_expansion_labeled_qualified_validators_generate_distinct_members() {
    let input: DeriveInput = syn::parse_quote! {
        pub struct QualifiedValidators {
            #[koruma(
                low_range = foo::RangeValidation.min(0).max(10),
                high_range = bar::RangeValidation.min(11).max(20),
            )]
            pub value: i32,
        }
    };

    let expanded = expand_koruma(input).unwrap();
    let compact = compact_ws(&pretty_print(expanded));
    assert!(compact.contains("low_range:Option<foo::RangeValidation>"));
    assert!(compact.contains("high_range:Option<bar::RangeValidation>"));
    assert!(compact.contains("LowRange(&'korumafoo::RangeValidation)"));
    assert!(compact.contains("HighRange(&'korumabar::RangeValidation)"));
}


#[test]
fn test_koruma_expansion_empty_koruma_attr_is_rejected() {
    let input: DeriveInput = syn::parse_quote! {
        pub struct EmptyKorumaAttrField {
            #[koruma()]
            pub value: i32,
        }
    };

    let err = expand_koruma(input).expect_err("empty koruma attributes should be rejected");
    assert!(
        err.to_string()
            .contains("must contain a modifier, validator, or `each(...)` block"),
        "expected empty attribute rejection, got: {err}"
    );
}


#[test]
fn test_koruma_expansion_field_arg_ident_is_rejected() {
    let input: DeriveInput = syn::parse_quote! {
        pub struct MatchesValidationInput {
            pub password: String,
            #[koruma(MatchesValidation.matches(password))]
            pub confirm: String,
        }
    };

    let err = expand_koruma(input).unwrap_err();
    assert!(
        err.to_string()
            .contains("bare field argument `password` is ambiguous")
    );
}

#[test]
fn test_koruma_expansion_direct_chain_uses_supplied_setters() {
    let input: DeriveInput = syn::parse_quote! {
        pub struct DirectSyntaxItem {
            #[koruma(GenericRangeValidation::<_>.min(0.0).max(100.0))]
            pub score: f64,
        }
    };

    let expanded = expand_koruma(input).unwrap();
    let compact = compact_ws(&pretty_print(expanded));
    assert!(compact.contains("GenericRangeValidation::<f64>::min(0.0).max(100.0)"));
}

#[test]
fn test_koruma_expansion_direct_chain_field_arg_requires_explicit_self() {
    let input: DeriveInput = syn::parse_quote! {
        pub struct BuilderMatchesValidationInput {
            pub password: String,
            #[koruma(MatchesValidation.matches(self.password.clone()))]
            pub confirm: String,
        }
    };

    let expanded = expand_koruma(input).unwrap();
    let compact = compact_ws(&pretty_print(expanded));
    assert!(compact.contains("MatchesValidation::matches(self.password.clone())"));
}

#[test]
fn test_koruma_expansion_non_field_arg_ident_is_left_alone() {
    let input: DeriveInput = syn::parse_quote! {
        pub struct ConstantArgInput {
            #[koruma(MatchesValidation.matches(STATIC_MATCH))]
            pub confirm: String,
        }
    };

    let expanded = expand_koruma(input).unwrap();
    let compact = compact_ws(&pretty_print(expanded));
    assert!(compact.contains("MatchesValidation::matches(STATIC_MATCH)"));
    assert!(!compact.contains("self.STATIC_MATCH"));
}
