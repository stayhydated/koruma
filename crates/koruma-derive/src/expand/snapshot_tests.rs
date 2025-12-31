//! Snapshot tests for the expand module.
//!
//! These tests verify the generated TokenStream output using insta snapshots.

use super::*;
use insta::assert_snapshot;
use syn::{DeriveInput, ItemStruct};

/// Helper to format TokenStream as pretty-printed Rust code
fn pretty_print(tokens: TokenStream2) -> String {
    let file = syn::parse_file(&tokens.to_string()).unwrap();
    prettyplease::unparse(&file)
}

#[test]
fn test_validator_expansion_simple() {
    let input: ItemStruct = syn::parse_quote! {
        #[derive(Clone, Debug)]
        pub struct NumberRangeValidation {
            min: i32,
            max: i32,
            #[koruma(value)]
            pub actual: Option<i32>,
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
            pub actual: Option<T>,
        }
    };

    let expanded = expand_validator(input).unwrap();
    assert_snapshot!(pretty_print(expanded));
}

#[test]
fn test_koruma_expansion_single_validator() {
    let input: DeriveInput = syn::parse_quote! {
        pub struct Item {
            #[koruma(NumberRangeValidation(min = 0, max = 100))]
            pub age: i32,
        }
    };

    let expanded = expand_koruma(input).unwrap();
    assert_snapshot!(pretty_print(expanded));
}

#[test]
fn test_koruma_expansion_multiple_validators() {
    let input: DeriveInput = syn::parse_quote! {
        pub struct MultiValidatorItem {
            #[koruma(NumberRangeValidation(min = 0, max = 100), EvenNumberValidation)]
            pub value: i32,
        }
    };

    let expanded = expand_koruma(input).unwrap();
    assert_snapshot!(pretty_print(expanded));
}

#[test]
fn test_koruma_expansion_generic_validator() {
    let input: DeriveInput = syn::parse_quote! {
        pub struct GenericItem {
            #[koruma(GenericRangeValidation<_>(min = 0.0, max = 100.0))]
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
            #[koruma(each(GenericRangeValidation<_>(min = 0.0, max = 100.0)))]
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
            #[koruma(NumberRangeValidation(min = 0, max = 100))]
            pub age: i32,

            #[koruma(StringLengthValidation(min = 1, max = 67))]
            pub name: String,
        }
    };

    let expanded = expand_koruma(input).unwrap();
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
            pub actual: i32,
        }
    };

    let expanded = expand_validator(input).unwrap();
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
            #[koruma(each(RangeValidation(min = 0, max = 100), EvenValidation))]
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
            #[koruma(RangeValidation(min = 0, max = 100))]
            pub age: i32,

            #[koruma(each(LengthValidation(min = 1, max = 50)))]
            pub tags: Vec<String>,

            #[koruma(RangeValidation(min = 0, max = 10), EvenValidation)]
            pub rating: i32,
        }
    };

    let expanded = expand_koruma(input).unwrap();
    assert_snapshot!(pretty_print(expanded));
}

#[test]
fn test_koruma_expansion_optional_field() {
    // Optional field should generate if-let pattern and skip validation when None
    let input: DeriveInput = syn::parse_quote! {
        pub struct UserProfile {
            #[koruma(StringLengthValidation(min = 1, max = 50))]
            pub username: String,

            #[koruma(StringLengthValidation(min = 1, max = 200))]
            pub bio: Option<String>,
        }
    };

    let expanded = expand_koruma(input).unwrap();
    assert_snapshot!(pretty_print(expanded));
}

#[test]
fn test_koruma_expansion_optional_with_generic() {
    // Optional field with generic validator
    let input: DeriveInput = syn::parse_quote! {
        pub struct Item {
            #[koruma(GenericRange<_>(min = 0, max = 100))]
            pub score: Option<i32>,
        }
    };

    let expanded = expand_koruma(input).unwrap();
    assert_snapshot!(pretty_print(expanded));
}

#[test]
fn test_koruma_expansion_combined_field_and_element_validators() {
    // Combined: field-level validator (for Vec) + element validators (for each element)
    let input: DeriveInput = syn::parse_quote! {
        pub struct OrderWithLenCheck {
            #[koruma(VecLenValidation<_>(min = 1, max = 10), each(RangeValidation<_>(min = 0, max = 100)))]
            pub scores: Vec<i32>,
        }
    };

    let expanded = expand_koruma(input).unwrap();
    assert_snapshot!(pretty_print(expanded));
}

#[test]
fn test_koruma_expansion_only_element_validators() {
    // Only element validators (no field-level validators) - backwards compatible with existing each()
    let input: DeriveInput = syn::parse_quote! {
        pub struct Scores {
            #[koruma(each(RangeValidation<_>(min = 0, max = 100)))]
            pub values: Vec<i32>,
        }
    };

    let expanded = expand_koruma(input).unwrap();
    assert_snapshot!(pretty_print(expanded));
}
