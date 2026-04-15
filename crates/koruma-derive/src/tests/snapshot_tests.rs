//! Snapshot tests for the expand module.
//!
//! These tests verify the generated TokenStream output using insta snapshots.

use crate::expand::*;

use insta::assert_snapshot;
use proc_macro2::TokenStream as TokenStream2;
use syn::{DeriveInput, ItemStruct};

/// Helper to format TokenStream as pretty-printed Rust code
fn pretty_print(tokens: TokenStream2) -> String {
    let file = syn::parse_file(&tokens.to_string()).unwrap();
    prettyplease::unparse(&file)
}

fn compact_ws(input: &str) -> String {
    input.chars().filter(|c| !c.is_whitespace()).collect()
}

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
            actual: i32,
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
    // Note: VecLenValidation<T> expects T to be the inner type, so we use explicit <i32>
    // instead of <_> (which would give Vec<i32>).
    let input: DeriveInput = syn::parse_quote! {
        pub struct OrderWithLenCheck {
            #[koruma(VecLenValidation<i32>(min = 1, max = 10), each(RangeValidation<_>(min = 0, max = 100)))]
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

#[test]
fn test_koruma_expansion_try_new() {
    // Struct with #[koruma(try_new)] generates a try_new constructor
    let input: DeriveInput = syn::parse_quote! {
        #[koruma(try_new)]
        pub struct Person {
            #[koruma(RangeValidation(min = 0, max = 150))]
            pub age: i32,
            pub name: String,
        }
    };

    let expanded = expand_koruma(input).unwrap();
    assert_snapshot!(pretty_print(expanded));
}

#[test]
fn test_koruma_expansion_try_new_tuple_struct() {
    // Tuple struct with #[koruma(try_new)] generates a try_new constructor with tuple initialization
    let input: DeriveInput = syn::parse_quote! {
        #[koruma(try_new)]
        pub struct Username(#[koruma(NonEmptyStringValidation)] pub String);
    };

    let expanded = expand_koruma(input).unwrap();
    let rendered = pretty_print(expanded);
    assert_snapshot!(rendered);
}

#[test]
fn test_koruma_expansion_try_new_newtype_tuple_struct() {
    // Tuple struct with both #[koruma(try_new, newtype)] - the main feature being tested
    let input: DeriveInput = syn::parse_quote! {
        #[koruma(try_new, newtype)]
        pub struct Username(#[koruma(NonEmptyStringValidation)] pub String);
    };

    let expanded = expand_koruma(input).unwrap();
    let rendered = pretty_print(expanded);
    assert_snapshot!(rendered);
}

#[test]
fn test_koruma_all_display_expansion() {
    let input: DeriveInput = syn::parse_quote! {
        pub struct DisplayItem {
            #[koruma(RangeValidation(min = 0, max = 10), EvenValidation)]
            pub value: i32,
            #[koruma(each(RangeValidation(min = 0, max = 10)))]
            pub values: Vec<i32>,
        }
    };

    let expanded = expand_koruma_all_display(input).unwrap();
    let rendered = pretty_print(expanded);
    assert!(rendered.contains("impl ::std::fmt::Display for DisplayItemValueKorumaValidator"));
    assert!(
        rendered.contains("impl ::std::fmt::Display for DisplayItemValuesElementKorumaValidator")
    );
}

#[test]
fn test_koruma_all_display_expansion_newtype_inner_arm() {
    let input: DeriveInput = syn::parse_quote! {
        pub struct DisplayNewtypeItem {
            #[koruma(newtype, RequiredValidation)]
            pub wrapped: WrappedValue,
        }
    };

    let expanded = expand_koruma_all_display(input).unwrap();
    let rendered = pretty_print(expanded);
    assert!(rendered.contains("DisplayNewtypeItemWrappedKorumaValidator::Inner(inner)"));
}

#[test]
fn test_koruma_all_display_rejects_non_struct() {
    let input: DeriveInput = syn::parse_quote! {
        enum NotAStruct {
            A,
        }
    };
    assert!(expand_koruma_all_display(input).is_err());
}

#[test]
fn test_koruma_expansion_struct_newtype_optional_nested_no_deref_impl() {
    let input: DeriveInput = syn::parse_quote! {
        #[koruma(newtype)]
        pub struct OptionalNestedWrapper {
            #[koruma(nested)]
            pub inner: Option<InnerValue>,
        }
    };

    let expanded = expand_koruma(input).unwrap();
    let rendered = pretty_print(expanded);
    let compact = compact_ws(&rendered);
    assert!(compact.contains("implkoruma::NewtypeValidationforOptionalNestedWrapper{}"));
    assert!(!compact.contains("implcore::ops::DerefforOptionalNestedWrapperKorumaValidationError"));
}

#[test]
fn test_koruma_expansion_newtype_optional_without_field_validators() {
    let input: DeriveInput = syn::parse_quote! {
        pub struct OptionalNewtypeField {
            #[koruma(newtype)]
            pub wrapped: Option<WrappedValue>,
        }
    };

    let expanded = expand_koruma(input).unwrap();
    let compact = compact_ws(&pretty_print(expanded));
    assert!(compact.contains("ifletSome(ref__newtype_value)=self.wrapped"));
}

#[test]
fn test_koruma_expansion_newtype_with_full_and_unwrapped_validators() {
    let input: DeriveInput = syn::parse_quote! {
        pub struct RichNewtypeField {
            #[koruma(newtype, RequiredValidation<Option<_>>, GenericRange<_>(min = 0, max = 10), PlainValidation(min = 1))]
            pub wrapped: Option<WrappedValue>,
        }
    };

    let expanded = expand_koruma(input).unwrap();
    let compact = compact_ws(&pretty_print(expanded));
    assert!(compact.contains("__koruma_assert_validate_wrapped_required_validation_newtype_field"));
    assert!(compact.contains(".with_value(__newtype_value.clone())"));
    assert!(compact.contains("letvalidator=PlainValidation::builder()"));
}

#[test]
fn test_koruma_expansion_optional_field_with_full_and_unwrapped_validators() {
    let input: DeriveInput = syn::parse_quote! {
        pub struct OptionalMixedValidators {
            #[koruma(RequiredValidation<Option<_>>, GenericRange<_>(min = 0, max = 10))]
            pub value: Option<i32>,
        }
    };

    let expanded = expand_koruma(input).unwrap();
    let compact = compact_ws(&pretty_print(expanded));
    assert!(compact.contains("ifletSome(ref__field_value)=self.value"));
    assert!(compact.contains("RequiredValidation::<Option<i32>>::builder()"));
}

#[test]
fn test_koruma_expansion_non_optional_field_with_full_and_unwrapped_validators() {
    let input: DeriveInput = syn::parse_quote! {
        pub struct NonOptionalMixedValidators {
            #[koruma(RequiredValidation<Option<_>>, GenericRange<_>(min = 0, max = 10))]
            pub value: i32,
        }
    };

    let expanded = expand_koruma(input).unwrap();
    let compact = compact_ws(&pretty_print(expanded));
    assert!(compact.contains("let__field_value=&self.value;"));
}

#[test]
fn test_koruma_expansion_non_optional_field_with_only_full_type_validator() {
    let input: DeriveInput = syn::parse_quote! {
        pub struct NonOptionalFullOnlyValidator {
            #[koruma(RequiredValidation<Option<_>>)]
            pub value: i32,
        }
    };

    let expanded = expand_koruma(input).unwrap();
    let compact = compact_ws(&pretty_print(expanded));
    assert!(compact.contains("RequiredValidation::<Option<i32>>::builder()"));
}

#[test]
fn test_koruma_expansion_empty_koruma_attr_still_expands() {
    let input: DeriveInput = syn::parse_quote! {
        pub struct EmptyKorumaAttrField {
            #[koruma()]
            pub value: i32,
        }
    };

    let expanded = expand_koruma(input).unwrap();
    let rendered = pretty_print(expanded);
    assert!(rendered.contains("struct EmptyKorumaAttrFieldKorumaValidationError"));
}

#[test]
fn test_koruma_expansion_vec_option_each_with_explicit_infer_type() {
    let input: DeriveInput = syn::parse_quote! {
        pub struct VecOptionElementValidators {
            #[koruma(each(GenericRange<Vec<_>>(min = 0, max = 10)))]
            pub values: Vec<Option<i32>>,
        }
    };

    let expanded = expand_koruma(input).unwrap();
    let compact = compact_ws(&pretty_print(expanded));
    assert!(compact.contains("ifletSome(ref__item_value)=item"));
    assert!(compact.contains("generic_range:Option<GenericRange<Vec<i32>>>"));
    assert!(compact.contains("GenericRange::<Vec<i32>>::builder()"));
}

#[test]
fn test_koruma_expansion_option_vec_each_uses_inner_collection() {
    let input: DeriveInput = syn::parse_quote! {
        pub struct OptionalVecElementValidators {
            #[koruma(each(GenericRange<_>(min = 0, max = 10)))]
            pub values: Option<Vec<i32>>,
        }
    };

    let expanded = expand_koruma(input).unwrap();
    let compact = compact_ws(&pretty_print(expanded));
    assert!(compact.contains("ifletSome(ref__collection_value)=self.values"));
    assert!(compact.contains("for(idx,__item_value)in__collection_value.iter().enumerate()"));
    assert!(compact.contains("GenericRange::<i32>::builder()"));
}

#[test]
fn test_koruma_expansion_slice_each_uses_element_type() {
    let input: DeriveInput = syn::parse_quote! {
        pub struct BorrowedSliceElementValidators<'a> {
            #[koruma(each(GenericRange<_>(min = 0, max = 10)))]
            pub values: &'a [i32],
        }
    };

    let expanded = expand_koruma(input).unwrap();
    let compact = compact_ws(&pretty_print(expanded));
    assert!(compact.contains("for(idx,__item_value)inself.values.iter().enumerate()"));
    assert!(compact.contains("GenericRange::<i32>::builder()"));
}

#[test]
fn test_koruma_expansion_multi_generic_explicit_type_infers_per_slot() {
    let input: DeriveInput = syn::parse_quote! {
        pub struct MultiGenericInference {
            #[koruma(MultiGenericValidation<std::collections::HashMap<_, _>>)]
            pub values: std::collections::HashMap<String, i32>,
        }
    };

    let expanded = expand_koruma(input).unwrap();
    let compact = compact_ws(&pretty_print(expanded));
    assert!(compact.contains("MultiGenericValidation<std::collections::HashMap<String,i32>>"));
}

#[test]
fn test_koruma_expansion_field_arg_ident_transforms_to_self_clone() {
    let input: DeriveInput = syn::parse_quote! {
        pub struct MatchesValidationInput {
            pub password: String,
            #[koruma(MatchesValidation(matches = password))]
            pub confirm: String,
        }
    };

    let expanded = expand_koruma(input).unwrap();
    let compact = compact_ws(&pretty_print(expanded));
    assert!(compact.contains(".matches(self.password.clone())"));
}

#[test]
fn test_koruma_expansion_non_field_arg_ident_is_left_alone() {
    let input: DeriveInput = syn::parse_quote! {
        pub struct ConstantArgInput {
            #[koruma(MatchesValidation(matches = STATIC_MATCH))]
            pub confirm: String,
        }
    };

    let expanded = expand_koruma(input).unwrap();
    let compact = compact_ws(&pretty_print(expanded));
    assert!(compact.contains(".matches(STATIC_MATCH)"));
    assert!(!compact.contains("self.STATIC_MATCH"));
}

#[test]
fn test_koruma_expansion_struct_newtype_nested_deref_has_no_expect() {
    let input: DeriveInput = syn::parse_quote! {
        #[koruma(newtype)]
        pub struct NestedWrapper {
            #[koruma(nested)]
            pub inner: InnerValue,
        }
    };

    let expanded = expand_koruma(input).unwrap();
    let compact = compact_ws(&pretty_print(expanded));
    assert!(compact.contains("implcore::ops::DerefforNestedWrapperKorumaValidationError"));
    assert!(compact.contains("fnderef(&self)->&Self::Target{&self.inner}"));
    assert!(!compact.contains("expect(\"newtypeerrorshouldhaveinnererror\")"));
}

#[test]
fn test_koruma_all_display_handles_skipped_fields() {
    let input: DeriveInput = syn::parse_quote! {
        pub struct DisplayWithSkippedField {
            pub plain: i32,
            #[koruma(RangeValidation(min = 0, max = 10))]
            pub value: i32,
        }
    };

    let expanded = expand_koruma_all_display(input).unwrap();
    let rendered = pretty_print(expanded);
    assert!(
        rendered
            .contains("impl ::std::fmt::Display for DisplayWithSkippedFieldValueKorumaValidator")
    );
}

#[test]
fn test_koruma_all_display_all_fields_skipped() {
    let input: DeriveInput = syn::parse_quote! {
        pub struct DisplayAllSkipped {
            pub a: i32,
            pub b: String,
        }
    };

    let expanded = expand_koruma_all_display(input).unwrap();
    let rendered = pretty_print(expanded);
    assert!(!rendered.contains("impl ::std::fmt::Display for"));
}

#[test]
#[cfg(feature = "internal-showcase")]
fn test_validator_expansion_showcase_with_generics_and_where_clause() {
    let input: ItemStruct = syn::parse_quote! {
        #[derive(Clone, Debug)]
        #[showcase(
            name = "Demo",
            description = "Demo description",
            create = |input: &str| {
                let _ = input;
                ::anyhow::Result::Ok(ShowcaseValidation::<String, usize, 2>::builder().with_value("x".to_string()).build())
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
    assert!(compact.contains("module:\"general\""));
    assert!(compact.contains("showcase_validation_builder::State"));
    assert!(compact.contains("S::Actual:koruma::bon::IsUnset"));
    assert!(compact.contains("DynValidatorforShowcaseValidation"));
}

#[cfg(feature = "fluent")]
#[test]
fn test_koruma_all_fluent_expansion() {
    let input: DeriveInput = syn::parse_quote! {
        pub struct FluentItem {
            #[koruma(RangeValidation(min = 0, max = 10), EvenValidation)]
            pub value: i32,
            #[koruma(each(RangeValidation(min = 0, max = 10)))]
            pub values: Vec<i32>,
        }
    };

    let expanded = expand_koruma_all_fluent(input).unwrap();
    let rendered = pretty_print(expanded);
    assert!(
        rendered.contains("impl ::es_fluent::ToFluentString for FluentItemValueKorumaValidator")
    );
    assert!(
        rendered.contains(
            "impl ::es_fluent::ToFluentString for FluentItemValuesElementKorumaValidator"
        )
    );
}

#[cfg(feature = "fluent")]
#[test]
fn test_koruma_all_fluent_handles_skipped_fields() {
    let input: DeriveInput = syn::parse_quote! {
        pub struct FluentWithSkippedField {
            pub plain: i32,
            #[koruma(RangeValidation(min = 0, max = 10))]
            pub value: i32,
        }
    };

    let expanded = expand_koruma_all_fluent(input).unwrap();
    let rendered = pretty_print(expanded);
    assert!(rendered.contains(
        "impl ::es_fluent::ToFluentString for FluentWithSkippedFieldValueKorumaValidator"
    ));
}

#[cfg(feature = "fluent")]
#[test]
fn test_koruma_all_fluent_all_fields_skipped() {
    let input: DeriveInput = syn::parse_quote! {
        pub struct FluentAllSkipped {
            pub a: i32,
            pub b: String,
        }
    };

    let expanded = expand_koruma_all_fluent(input).unwrap();
    let rendered = pretty_print(expanded);
    assert!(!rendered.contains("impl ::es_fluent::ToFluentString for"));
}

#[cfg(feature = "fluent")]
#[test]
fn test_koruma_all_fluent_expansion_newtype_inner_delegate() {
    let input: DeriveInput = syn::parse_quote! {
        pub struct FluentNewtypeItem {
            #[koruma(newtype, RequiredValidation)]
            pub wrapped: WrappedValue,
        }
    };

    let expanded = expand_koruma_all_fluent(input).unwrap();
    let rendered = pretty_print(expanded);
    assert!(rendered.contains("self.inner().to_fluent_string()"));
}

#[cfg(feature = "fluent")]
#[test]
fn test_koruma_all_fluent_rejects_non_struct() {
    let input: DeriveInput = syn::parse_quote! {
        enum NotAStruct {
            A,
        }
    };
    assert!(expand_koruma_all_fluent(input).is_err());
}

#[test]
fn test_koruma_expansion_try_from_tuple_struct() {
    let input: DeriveInput = syn::parse_quote! {
        #[koruma(newtype(try_from))]
        pub struct Email(#[koruma(NonEmptyStringValidation)] pub String);
    };

    let expanded = expand_koruma(input).unwrap();
    let rendered = pretty_print(expanded);
    assert_snapshot!(rendered);
}

#[test]
fn test_koruma_expansion_try_from_named_field_struct() {
    let input: DeriveInput = syn::parse_quote! {
        #[koruma(newtype(try_from))]
        pub struct Username {
            #[koruma(NonEmptyStringValidation)]
            pub value: String,
        }
    };

    let expanded = expand_koruma(input).unwrap();
    let rendered = pretty_print(expanded);
    assert_snapshot!(rendered);
}

#[test]
fn test_koruma_expansion_try_from_generic() {
    let input: DeriveInput = syn::parse_quote! {
        #[koruma(newtype(try_from))]
        pub struct Wrapper<T>(#[koruma(GenericRange<_>(min = 0, max = 100))] pub T);
    };

    let expanded = expand_koruma(input).unwrap();
    let rendered = pretty_print(expanded);
    assert_snapshot!(rendered);
}

#[test]
fn test_koruma_expansion_try_from_generic_with_bounds() {
    let input: DeriveInput = syn::parse_quote! {
        #[koruma(newtype(try_from))]
        pub struct BoundedWrapper<T: Clone>(#[koruma(GenericRange<_>(min = 0, max = 100))] pub T);
    };

    let expanded = expand_koruma(input).unwrap();
    let rendered = pretty_print(expanded);
    assert_snapshot!(rendered);
}

#[test]
fn test_koruma_expansion_try_from_generic_with_where_clause() {
    let input: DeriveInput = syn::parse_quote! {
        #[koruma(newtype(try_from))]
        pub struct WhereWrapper<T>(#[koruma(GenericRange<_>(min = 0, max = 100))] pub T) where T: Default;
    };

    let expanded = expand_koruma(input).unwrap();
    let rendered = pretty_print(expanded);
    assert_snapshot!(rendered);
}

#[test]
fn test_koruma_expansion_try_from_option_field() {
    let input: DeriveInput = syn::parse_quote! {
        #[koruma(newtype(try_from))]
        pub struct OptionalWrapper {
            #[koruma(newtype, RequiredValidation<Option<_>>)]
            pub inner: Option<InnerValue>,
        }
    };

    let expanded = expand_koruma(input).unwrap();
    let rendered = pretty_print(expanded);
    assert_snapshot!(rendered);
}

#[test]
fn test_koruma_expansion_try_from_requires_single_field() {
    let input: DeriveInput = syn::parse_quote! {
        #[koruma(newtype(try_from))]
        pub struct MultiField {
            #[koruma(NonEmptyStringValidation)]
            pub a: String,
            #[koruma(skip)]
            pub b: i32,
        }
    };

    let result = expand_koruma(input);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.to_string()
            .contains("newtype(try_from) requires exactly one field")
    );
}
