//! Error case tests for the expand module.

use crate::expand::*;
use syn::{DeriveInput, ItemStruct};

#[test]
fn test_validator_error_missing_value_field() {
    let input: ItemStruct = syn::parse_quote! {
        pub struct BadValidator {
            #[koruma(setter)]
            min: i32,
            #[koruma(setter)]
            max: i32,
            // Missing an unmarked or explicit value field.
        }
    };

    let result = expand_validator(input);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("requires a value field"));
}

#[test]
fn test_validator_error_public_value_field() {
    let input: ItemStruct = syn::parse_quote! {
        pub struct BadValidator {
            min: i32,
            max: i32,
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
    assert!(err.to_string().contains("requires exactly one value field"));
}

#[test]
fn test_validator_error_on_duplicate_value_marker_same_field() {
    let input: ItemStruct = syn::parse_quote! {
        pub struct BadValidator {
            #[koruma(value, value)]
            actual: i32,
        }
    };

    let result = expand_validator(input);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("has multiple value markers"));
}

#[test]
fn test_validator_error_capture_skip_requires_option_value_field() {
    let input: ItemStruct = syn::parse_quote! {
        pub struct BadValidator {
            #[koruma(skip_capture)]
            actual: String,
        }
    };

    let result = expand_validator(input);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.to_string()
            .contains("`#[koruma(skip_capture)]` requires an `Option<T>` field")
    );
}

#[test]
fn test_validator_error_value_field_rejects_setter_metadata() {
    let input: ItemStruct = syn::parse_quote! {
        pub struct BadValidator {
            #[koruma(value, setter(required))]
            actual: Option<i32>,
        }
    };

    let result = expand_validator(input);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.to_string()
            .contains("validator value fields cannot also use `#[koruma(setter(...))]`")
    );
}

#[test]
fn test_validator_error_unknown_setter_option() {
    let input: ItemStruct = syn::parse_quote! {
        pub struct BadValidator {
            #[koruma(setter(skip))]
            min: i32,
            #[koruma(value)]
            actual: Option<i32>,
        }
    };

    let result = expand_validator(input);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.to_string()
            .contains("unsupported `#[koruma(setter(skip))]` option")
    );
}

#[test]
fn test_validator_error_required_and_default_setter_options_conflict() {
    let input: ItemStruct = syn::parse_quote! {
        pub struct BadValidator {
            #[koruma(setter(required, default = 0))]
            min: i32,
            #[koruma(value)]
            actual: Option<i32>,
        }
    };

    let result = expand_validator(input);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.to_string()
            .contains("`required` and `default` cannot be combined")
    );
}

#[test]
fn test_validator_error_duplicate_setter_options() {
    for (input, expected) in [
        (
            syn::parse_quote! {
                pub struct BadValidator {
                    #[koruma(setter(name = min_value, name = minimum))]
                    min: i32,
                    #[koruma(value)]
                    actual: Option<i32>,
                }
            },
            "duplicate `setter(name = ...)` option",
        ),
        (
            syn::parse_quote! {
                pub struct BadValidator {
                    #[koruma(setter(default, default = 1))]
                    min: i32,
                    #[koruma(value)]
                    actual: Option<i32>,
                }
            },
            "duplicate `setter(default)` option",
        ),
        (
            syn::parse_quote! {
                pub struct BadValidator {
                    #[koruma(setter(required, required))]
                    min: i32,
                    #[koruma(value)]
                    actual: Option<i32>,
                }
            },
            "duplicate `setter(required)` option",
        ),
    ] {
        let err = expand_validator(input).expect_err("expected duplicate setter option error");
        assert!(
            err.to_string().contains(expected),
            "expected `{expected}`, got: {err}"
        );
    }
}

#[test]
fn test_validator_error_rejects_multiple_field_attrs() {
    let input: ItemStruct = syn::parse_quote! {
        pub struct BadValidator {
            #[koruma(setter(default))]
            #[koruma(setter(default = 1))]
            min: i32,
            #[koruma(value)]
            actual: Option<i32>,
        }
    };

    let err = expand_validator(input).expect_err("expected repeated attribute error");
    assert!(
        err.to_string()
            .contains("only one validator-field `#[koruma(...)]` attribute is allowed"),
        "expected repeated validator-field attribute error, got: {err}"
    );
}

#[test]
fn test_validator_error_generated_setter_method_collisions() {
    for (input, expected) in [
        (
            syn::parse_quote! {
                pub struct BadValidator {
                    #[koruma(setter(name = same))]
                    min: i32,
                    #[koruma(setter(name = same))]
                    max: i32,
                    #[koruma(value)]
                    actual: Option<i32>,
                }
            },
            "setter method `same` collides",
        ),
        (
            syn::parse_quote! {
                pub struct BadValidator {
                    #[koruma(setter(name = with_value))]
                    min: i32,
                    #[koruma(value)]
                    actual: Option<i32>,
                }
            },
            "setter method name `with_value` is reserved",
        ),
        (
            syn::parse_quote! {
                pub struct BadValidator {
                    #[koruma(setter(name = __koruma_builder))]
                    min: i32,
                    #[koruma(value)]
                    actual: Option<i32>,
                }
            },
            "setter method name `__koruma_builder` is reserved",
        ),
        (
            syn::parse_quote! {
                pub struct BadValidator {
                    maybe_min: i32,
                    min: Option<i32>,
                    #[koruma(value)]
                    actual: Option<i32>,
                }
            },
            "setter method name `maybe_min` is reserved",
        ),
    ] {
        let err = expand_validator(input).expect_err("expected setter method collision");
        assert!(
            err.to_string().contains(expected),
            "expected `{expected}`, got: {err}"
        );
    }
}

#[test]
fn test_validator_error_generated_builder_name_collisions() {
    for (input, expected) in [
        (
            syn::parse_quote! {
                pub struct BadValidator {
                    _state: i32,
                    #[koruma(value)]
                    actual: Option<i32>,
                }
            },
            "field name `_state` is reserved",
        ),
        (
            syn::parse_quote! {
                pub struct BadValidator<__KorumaMinState> {
                    min: i32,
                    #[koruma(value)]
                    actual: Option<i32>,
                }
            },
            "generated required-state generic `__KorumaMinState` collides",
        ),
    ] {
        let err = expand_validator(input).expect_err("expected generated-name collision");
        assert!(
            err.to_string().contains(expected),
            "expected `{expected}`, got: {err}"
        );
    }
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
            #[koruma(RangeValidation::min(0).max(100), RangeValidation::min(10).max(50))]
            pub value: i32,
        }
    };

    let result = expand_koruma(input);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("add explicit validator labels"),
        "expected label-required duplicate-name error, got: {}",
        err
    );
}

#[test]
fn test_koruma_error_on_repeated_field_attrs() {
    let input: DeriveInput = syn::parse_quote! {
        pub struct RepeatedFieldAttrs {
            #[koruma(RangeValidation::min(0).max(100))]
            #[koruma(RangeValidation::min(10).max(50))]
            pub value: i32,
        }
    };

    let result = expand_koruma(input);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.to_string()
            .contains("only one field-level `#[koruma(...)]` attribute is allowed"),
        "expected repeated field attribute error, got: {}",
        err
    );
}

#[test]
fn test_koruma_error_on_duplicate_element_validator() {
    let input: DeriveInput = syn::parse_quote! {
        pub struct DuplicateElementValidator {
            #[koruma(each(RangeValidation::min(0).max(100), RangeValidation::min(10).max(50)))]
            pub values: Vec<i32>,
        }
    };

    let result = expand_koruma(input);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("add explicit validator labels"),
        "expected label-required duplicate-name error, got: {}",
        err
    );
}

#[test]
fn test_koruma_error_on_explicit_option_element_validator_for_non_optional_target() {
    let input: DeriveInput = syn::parse_quote! {
        pub struct NonOptionalExplicitElementFullTypeValidator {
            #[koruma(each(GenericValidation::<Option<String>>))]
            pub values: Vec<String>,
        }
    };

    let result = expand_koruma(input);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains(
            "explicit `Option<...>` validator type arguments require an optional validation target"
        ),
        "expected non-optional option-target diagnostic, got: {}",
        err
    );
    assert!(
        err.to_string()
            .contains("element validators on field `values`"),
        "expected element context in diagnostic, got: {}",
        err
    );
}

#[test]
fn test_koruma_error_on_explicit_option_validator_for_non_optional_target() {
    let input: DeriveInput = syn::parse_quote! {
        pub struct NonOptionalExplicitFullTypeValidator {
            #[koruma(GenericValidation::<Option<String>>)]
            pub value: String,
        }
    };

    let result = expand_koruma(input);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains(
            "explicit `Option<...>` validator type arguments require an optional validation target"
        ),
        "expected non-optional option-target diagnostic, got: {}",
        err
    );
    assert!(
        err.to_string().contains("field `value`"),
        "expected field name in diagnostic, got: {}",
        err
    );
}

#[test]
fn test_koruma_error_on_explicit_option_validator_with_unwrapped_target() {
    let input: DeriveInput = syn::parse_quote! {
        pub struct OptionalExplicitFullTypeValidator {
            #[koruma(unwrapped(GenericValidation::<Option<_>>))]
            pub value: Option<String>,
        }
    };

    let result = expand_koruma(input);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains(
            "explicit `Option<...>` validator type arguments require full-option target selection"
        ),
        "expected unwrapped target conflict diagnostic, got: {}",
        err
    );
    assert!(
        err.to_string().contains("field `value`"),
        "expected field context in diagnostic, got: {}",
        err
    );
}

#[test]
fn test_koruma_error_on_explicit_option_element_validator_with_unwrapped_target() {
    let input: DeriveInput = syn::parse_quote! {
        pub struct OptionalExplicitFullElementTypeValidator {
            #[koruma(each(unwrapped(GenericValidation::<Option<_>>)))]
            pub values: Vec<Option<String>>,
        }
    };

    let result = expand_koruma(input);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains(
            "explicit `Option<...>` validator type arguments require full-option target selection"
        ),
        "expected unwrapped target conflict diagnostic, got: {}",
        err
    );
    assert!(
        err.to_string()
            .contains("element validators on field `values`"),
        "expected element context in diagnostic, got: {}",
        err
    );
}

#[test]
fn test_koruma_error_on_direct_chain_with_build_call() {
    let input: DeriveInput = syn::parse_quote! {
        pub struct InvalidDirectSyntax {
            #[koruma(NumberRangeValidation::min(0).max(100).build())]
            pub value: i32,
        }
    };

    let result = expand_koruma(input);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.to_string()
            .contains("injects builder creation, value capture, and `.build()` automatically"),
        "expected validator chain diagnostic, got: {}",
        err
    );
}

#[test]
fn test_koruma_error_on_invalid_validator_call() {
    let input: DeriveInput = syn::parse_quote! {
        pub struct InvalidValidatorSyntax {
            #[koruma(NumberRangeValidation(min = 0, max = 100))]
            pub value: i32,
        }
    };

    let result = expand_koruma(input);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.to_string()
            .contains("requires a direct validator chain"),
        "expected direct-chain diagnostic, got: {}",
        err
    );
}

#[test]
fn test_koruma_error_on_each_non_vec_collection() {
    let input: DeriveInput = syn::parse_quote! {
        pub struct NonVecEach {
            #[koruma(each(RangeValidation::min(0).max(100)))]
            pub values: std::collections::HashSet<i32>,
        }
    };

    let result = expand_koruma(input);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.to_string()
            .contains("`each(...)` supports syntactic `Vec<T>`, slice fields"),
        "expected each(...) collection diagnostic, got: {err}",
    );
    assert!(
        err.to_string().contains("the outer field type"),
        "expected outer field context, got: {err}",
    );
}

#[test]
fn test_koruma_error_on_each_optional_non_collection() {
    let input: DeriveInput = syn::parse_quote! {
        pub struct OptionalNonCollectionEach {
            #[koruma(each(RangeValidation::min(0).max(100)))]
            pub values: Option<std::collections::HashSet<i32>>,
        }
    };

    let result = expand_koruma(input);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.to_string()
            .contains("the unwrapped optional collection"),
        "expected unwrapped optional context, got: {err}",
    );
}

#[test]
fn test_koruma_error_on_struct_level_newtype_with_wrong_field_count() {
    let input: DeriveInput = syn::parse_quote! {
        #[koruma(newtype)]
        pub struct BadNewtype {
            #[koruma(RangeValidation::min(0).max(10))]
            pub a: i32,
            #[koruma(RangeValidation::min(0).max(10))]
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
            #[koruma(RangeValidation::min(0).max(10))]
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
            #[koruma(RequiredValidation::<Option<_>>)]
            pub child: Option<Child>,
        }
    };

    let result = expand_koruma(input);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.to_string()
            .contains("only one field-level `#[koruma(...)]` attribute is allowed"),
        "expected repeated field attribute error, got: {err}"
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
            .contains("only one field-level `#[koruma(...)]` attribute is allowed"),
        "expected repeated field attribute error, got: {err}"
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
            .contains("only one field-level `#[koruma(...)]` attribute is allowed"),
        "expected repeated field attribute error, got: {err}"
    );
}
