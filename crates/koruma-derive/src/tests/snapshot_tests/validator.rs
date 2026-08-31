#[test]
fn test_validator_expansion_simple() {
    let input: ItemStruct = syn::parse_quote! {
        #[derive(Clone, Debug)]
        pub struct NumberRangeValidation {
            min: i32,
            max: i32,
            #[koruma(value)]
            actual: Option<i32>,
        }
    };

    let expanded = expand_validator(input).unwrap();
    assert_snapshot!(pretty_print(expanded));
}

#[test]
fn test_validator_expansion_generic() {
    let input: ItemStruct = syn::parse_quote! {
        #[derive(Clone, Debug)]
        pub struct GenericRangeValidation<T> {
            pub min: T,
            pub max: T,
            #[koruma(value)]
            actual: Option<T>,
        }
    };

    let expanded = expand_validator(input).unwrap();
    assert_snapshot!(pretty_print(expanded));
}


#[test]
fn test_validator_expansion_non_option_value() {
    // Value field that is NOT wrapped in Option
    let input: ItemStruct = syn::parse_quote! {
        #[derive(Clone, Debug)]
        pub struct DirectValueValidation {
            min: i32,
            #[koruma(value)]
            actual: i32,
        }
    };

    let expanded = expand_validator(input).unwrap();
    assert_snapshot!(pretty_print(expanded));
}

#[test]
fn test_validator_expansion_capture_skip_policy() {
    let input: ItemStruct = syn::parse_quote! {
        #[derive(Clone, Debug)]
        pub struct PresenceOnlyValidation<T> {
            #[koruma(skip_capture)]
            actual: Option<T>,
        }
    };

    let expanded = expand_validator(input).unwrap();
    assert_snapshot!(pretty_print(expanded));
}


#[test]
#[cfg(feature = "internal-showcase")]
fn test_validator_expansion_showcase_with_generics_and_where_clause() {
    let input: ItemStruct = syn::parse_quote! {
        #[derive(Clone, Debug)]
        #[showcase(
            name = "Demo",
            description = "Demo description",
            input_type = Text,
            create = |input: &str| {
                let _ = input;
                ::anyhow::Result::Ok(ShowcaseValidation::<String, usize, 2>::with_value("x".to_string()).build())
            }
        )]
        pub struct ShowcaseValidation<'a, T: Clone, U, const N: usize>
        where
            U: Default,
        {
            pub marker: &'a str,
            pub extra: U,
            #[koruma(value)]
            actual: Option<T>,
        }
    };

    let expanded = expand_validator(input).unwrap();
    let compact = compact_ws(&pretty_print(expanded));
    assert!(compact.contains("input_type:::renamed_koruma::showcase::InputType::Text"));
    assert!(compact.contains("module:::renamed_koruma::showcase::ValidatorModule::General"));
    assert!(compact.contains("ShowcaseValidationBuilder<"));
    assert!(!compact.contains("::renamed_koruma::bon"));
    assert!(compact.contains("DynValidatorforShowcaseValidation"));
    assert!(compact.contains("whereU:Default"));
    assert!(compact.contains("Self:::std::marker::Send+::std::marker::Sync"));
    assert!(compact.contains("Self:::renamed_koruma::Validate<Option<T>>"));
    assert!(compact.contains("Self:::std::fmt::Display"));
    assert!(!compact.contains("feature=\"internal-showcase\""));
    assert!(!compact.contains("feature=\"fmt\""));
    assert!(!compact.contains("feature=\"fluent\""));
}

#[test]
fn test_validator_expansion_respects_setter_metadata() {
    let input: ItemStruct = syn::parse_quote! {
        #[derive(Clone, Debug)]
        pub struct SetterMetadataValidation<T> {
            #[koruma(setter(into, name = label))]
            pub title: String,
            #[koruma(setter(into))]
            pub optional_title: Option<String>,
            #[koruma(setter(required))]
            pub required_limit: Option<usize>,
            pub optional_limit: Option<usize>,
            #[koruma(setter(default = 10))]
            pub defaulted: usize,
            #[koruma(setter(default = Some(3)))]
            pub defaulted_optional: Option<usize>,
            #[koruma(value)]
            actual: Option<T>,
        }
    };

    let expanded = expand_validator(input).unwrap();
    let rendered = pretty_print(expanded);
    let compact = compact_ws(&rendered);

    assert!(compact.contains("pubfnlabel(value:impl::std::convert::Into<String>,)"));
    assert!(compact.contains("pubfnoptional_title(value:impl::std::convert::Into<String>,)"));
    assert!(compact.contains("pubfnmaybe_optional_title(value:::std::option::Option<String>,)"));
    assert!(compact.contains("pubfnrequired_limit(value:Option<usize>,)"));
    assert!(!compact.contains("maybe_required_limit"));
    assert!(compact.contains("pubfnoptional_limit(value:usize,)"));
    assert!(compact.contains("pubfnmaybe_optional_limit(value:::std::option::Option<usize>,)"));
    assert!(compact.contains("pubfndefaulted(value:usize,)"));
    assert!(compact.contains("pubfndefaulted_optional(value:Option<usize>,)"));
    assert!(!compact.contains("maybe_defaulted_optional"));
    assert_snapshot!(rendered);
}
