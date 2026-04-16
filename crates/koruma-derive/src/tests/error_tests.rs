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
fn test_validator_error_public_value_field() {
    let input: ItemStruct = syn::parse_quote! {
        pub struct BadValidator {
            min: i32,
            max: i32,
            #[koruma(value)]
            pub actual: i32,
        }
    };

    let result = expand_validator(input);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("must be private"));
    assert!(err.to_string().contains("generated getter"));
}

#[test]
fn test_validator_error_tuple_struct_rejected_early() {
    let input: ItemStruct = syn::parse_quote! {
        pub struct TupleValidator(#[koruma(value)] i32);
    };

    let result = expand_validator(input);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.to_string()
            .contains("koruma::validator only supports structs with named fields")
    );
}

#[test]
fn test_validator_error_on_multiple_value_fields() {
    let input: ItemStruct = syn::parse_quote! {
        pub struct BadValidator {
            #[koruma(value)]
            actual: i32,
            #[koruma(value)]
            expected: i32,
        }
    };

    let result = expand_validator(input);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.to_string()
            .contains("requires exactly one `#[koruma(value)]` field")
    );
}

#[test]
fn test_validator_error_on_duplicate_value_marker_same_field() {
    let input: ItemStruct = syn::parse_quote! {
        pub struct BadValidator {
            #[koruma(value)]
            #[koruma(value)]
            actual: i32,
        }
    };

    let result = expand_validator(input);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.to_string()
            .contains("has multiple `#[koruma(value)]` markers")
    );
}

#[test]
fn test_validator_error_skip_capture_requires_option_value_field() {
    let input: ItemStruct = syn::parse_quote! {
        pub struct BadValidator {
            #[koruma(value, skip_capture)]
            actual: String,
        }
    };

    let result = expand_validator(input);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.to_string()
            .contains("`#[koruma(value, skip_capture)]` currently requires an `Option<T>` field")
    );
}

#[cfg(feature = "internal-showcase")]
#[test]
fn test_validator_error_on_invalid_showcase_attr() {
    let input: ItemStruct = syn::parse_quote! {
        #[showcase(
            name = "Bad Showcase",
            description = "Should fail",
            create = |input: &str| input,
            input_type = Text,
            modul = "broken"
        )]
        pub struct BadShowcaseValidator {
            #[koruma(value)]
            actual: Option<String>,
        }
    };

    let result = expand_validator(input);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.to_string()
            .contains("unknown showcase attribute: modul"),
        "expected showcase parse error, got: {err}"
    );
}

#[cfg(feature = "internal-showcase")]
#[test]
fn test_validator_error_on_missing_showcase_input_type() {
    let input: ItemStruct = syn::parse_quote! {
        #[showcase(
            name = "Bad Showcase",
            description = "Should fail",
            create = |input: &str| input
        )]
        pub struct BadShowcaseValidator {
            #[koruma(value)]
            actual: Option<String>,
        }
    };

    let result = expand_validator(input);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.to_string()
            .contains("showcase requires `input_type` attribute"),
        "expected showcase input_type error, got: {err}"
    );
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

#[test]
fn test_koruma_error_on_each_non_vec_collection() {
    let input: DeriveInput = syn::parse_quote! {
        pub struct NonVecEach {
            #[koruma(each(RangeValidation(min = 0, max = 100)))]
            pub values: std::collections::HashSet<i32>,
        }
    };

    let result = expand_koruma(input);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.to_string()
            .contains("`each(...)` currently only supports `Vec<T>`, slice fields"),
        "expected unsupported each(...) collection error, got: {err}",
    );
}

#[test]
fn test_koruma_error_on_struct_level_newtype_with_wrong_field_count() {
    let input: DeriveInput = syn::parse_quote! {
        #[koruma(newtype)]
        pub struct BadNewtype {
            #[koruma(RangeValidation(min = 0, max = 10))]
            pub a: i32,
            #[koruma(RangeValidation(min = 0, max = 10))]
            pub b: i32,
        }
    };

    let result = expand_koruma(input);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.to_string()
            .contains("newtype structs must have exactly one field"),
        "expected newtype field-count error, got: {err}"
    );
}

#[test]
fn test_koruma_error_on_struct_level_newtype_with_one_validated_and_one_plain_field() {
    let input: DeriveInput = syn::parse_quote! {
        #[koruma(newtype)]
        pub struct BadNewtype {
            #[koruma(RangeValidation(min = 0, max = 10))]
            pub a: i32,
            pub b: i32,
        }
    };

    let result = expand_koruma(input);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.to_string()
            .contains("newtype structs must have exactly one field"),
        "expected newtype field-count error, got: {err}"
    );
}

#[test]
fn test_koruma_error_on_tuple_newtype_with_multiple_fields() {
    // Tuple struct with newtype must have exactly one field
    let input: DeriveInput = syn::parse_quote! {
        #[koruma(newtype)]
        pub struct BadTupleNewtype(pub i32, pub i32);
    };

    let result = expand_koruma(input);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.to_string()
            .contains("newtype structs must have exactly one field"),
        "expected newtype field-count error, got: {err}"
    );
}

#[test]
fn test_koruma_error_on_struct_level_newtype_with_skipped_only_field() {
    let input: DeriveInput = syn::parse_quote! {
        #[koruma(newtype)]
        pub struct SkippedOnlyField(#[koruma(skip)] pub String);
    };

    let result = expand_koruma(input);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.to_string()
            .contains("require their only field to participate in validation"),
        "expected skipped-only-field newtype error, got: {err}"
    );
}

#[test]
fn test_koruma_error_on_nested_field_with_split_validators() {
    let input: DeriveInput = syn::parse_quote! {
        pub struct Parent {
            #[koruma(nested)]
            #[koruma(RequiredValidation<Option<_>>)]
            pub child: Option<Child>,
        }
    };

    let result = expand_koruma(input);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.to_string()
            .contains("cannot also use validators or `each(...)`"),
        "expected nested+validator compatibility error, got: {err}"
    );
}

#[test]
fn test_koruma_error_on_newtype_field_with_each() {
    let input: DeriveInput = syn::parse_quote! {
        pub struct Parent {
            #[koruma(newtype)]
            #[koruma(each(PositiveValidation))]
            pub child: Child,
        }
    };

    let result = expand_koruma(input);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.to_string()
            .contains("cannot also use `each(...)`; element validation is not supported"),
        "expected newtype + each rejection, got: {err}"
    );
}

#[test]
fn test_koruma_error_on_field_with_nested_and_newtype() {
    let input: DeriveInput = syn::parse_quote! {
        pub struct Parent {
            #[koruma(nested)]
            #[koruma(newtype)]
            pub child: Child,
        }
    };

    let result = expand_koruma(input);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.to_string()
            .contains("cannot combine `#[koruma(nested)]` and `#[koruma(newtype)]`"),
        "expected nested + newtype rejection, got: {err}"
    );
}
