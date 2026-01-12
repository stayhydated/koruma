//! Error case tests for the expand module.

use crate::expand::*;
use syn::{DeriveInput, ItemStruct};

#[test]
fn test_validator_error_missing_value_field() {
    let input: ItemStruct = syn::parse_quote! {
        pub struct BadValidator {
            min: i32,
            max: i32,
            // Missing #[koruma(value)] field!
        }
    };

    let result = expand_validator(input);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("koruma(value)"));
}

#[test]
fn test_koruma_success_no_validated_fields() {
    let input: DeriveInput = syn::parse_quote! {
        pub struct EmptyStruct {
            // No #[koruma(...)] attributes
            pub normal_field: i32,
        }
    };

    let result = expand_koruma(input);
    assert!(
        result.is_ok(),
        "Should succeed even without validated fields"
    );
}

#[test]
fn test_koruma_error_on_enum() {
    let input: DeriveInput = syn::parse_quote! {
        pub enum NotAStruct {
            VariantA,
            VariantB,
        }
    };

    let result = expand_koruma(input);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("struct"));
}

#[test]
fn test_koruma_error_on_tuple_struct() {
    let input: DeriveInput = syn::parse_quote! {
        pub struct TupleStruct(i32, String);
    };

    let result = expand_koruma(input);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("named fields"));
}

#[test]
fn test_koruma_error_on_duplicate_validator_same_attr() {
    let input: DeriveInput = syn::parse_quote! {
        pub struct DuplicateValidatorSameAttr {
            #[koruma(RangeValidation(min = 0, max = 100), RangeValidation(min = 10, max = 50))]
            pub value: i32,
        }
    };

    let result = expand_koruma(input);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("duplicate validator"),
        "expected 'duplicate validator' error, got: {}",
        err
    );
}

#[test]
fn test_koruma_error_on_duplicate_validator_separate_attrs() {
    let input: DeriveInput = syn::parse_quote! {
        pub struct DuplicateValidatorSeparateAttrs {
            #[koruma(RangeValidation(min = 0, max = 100))]
            #[koruma(RangeValidation(min = 10, max = 50))]
            pub value: i32,
        }
    };

    let result = expand_koruma(input);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("duplicate validator"),
        "expected 'duplicate validator' error, got: {}",
        err
    );
}

#[test]
fn test_koruma_error_on_duplicate_element_validator() {
    let input: DeriveInput = syn::parse_quote! {
        pub struct DuplicateElementValidator {
            #[koruma(each(RangeValidation(min = 0, max = 100)))]
            #[koruma(each(RangeValidation(min = 10, max = 50)))]
            pub values: Vec<i32>,
        }
    };

    let result = expand_koruma(input);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("duplicate element validator"),
        "expected 'duplicate element validator' error, got: {}",
        err
    );
}
