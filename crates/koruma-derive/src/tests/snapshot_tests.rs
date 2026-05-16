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
            #[koruma(NumberRangeValidation::builder().min(0).max(100))]
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
            #[koruma(NumberRangeValidation::builder().min(0).max(100), EvenNumberValidation::builder())]
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
            #[koruma(GenericRangeValidation::<_>::builder().min(0.0).max(100.0))]
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
            #[koruma(each(GenericRangeValidation::<_>::builder().min(0.0).max(100.0)))]
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
            #[koruma(NumberRangeValidation::builder().min(0).max(100))]
            pub age: i32,

            #[koruma(StringLengthValidation::builder().min(1).max(67))]
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
fn test_validator_expansion_skip_capture() {
    let input: ItemStruct = syn::parse_quote! {
        #[derive(Clone, Debug)]
        pub struct PresenceOnlyValidation<T> {
            #[koruma(value, skip_capture)]
            actual: Option<T>,
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
            #[koruma(EvenNumberValidation::builder())]
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
            #[koruma(each(RangeValidation::builder().min(0).max(100), EvenValidation::builder()))]
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
            #[koruma(RangeValidation::builder().min(0).max(100))]
            pub age: i32,

            #[koruma(each(LengthValidation::builder().min(1).max(50)))]
            pub tags: Vec<String>,

            #[koruma(RangeValidation::builder().min(0).max(10), EvenValidation::builder())]
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
            #[koruma(StringLengthValidation::builder().min(1).max(50))]
            pub username: String,

            #[koruma(StringLengthValidation::builder().min(1).max(200))]
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
            #[koruma(GenericRange::<_>::builder().min(0).max(100))]
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
            #[koruma(VecLenValidation::<i32>::builder().min(1).max(10), each(RangeValidation::<_>::builder().min(0).max(100)))]
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
            #[koruma(each(RangeValidation::<_>::builder().min(0).max(100)))]
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
            #[koruma(RangeValidation::builder().min(0).max(150))]
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
        pub struct Username(#[koruma(NonEmptyStringValidation::builder())] pub String);
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
        pub struct Username(#[koruma(NonEmptyStringValidation::builder())] pub String);
    };

    let expanded = expand_koruma(input).unwrap();
    let rendered = pretty_print(expanded);
    assert_snapshot!(rendered);
}

#[test]
fn test_koruma_all_display_expansion() {
    let input: DeriveInput = syn::parse_quote! {
        pub struct DisplayItem {
            #[koruma(RangeValidation::builder().min(0).max(10), EvenValidation::builder())]
            pub value: i32,
            #[koruma(each(RangeValidation::builder().min(0).max(10)))]
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
            #[koruma(newtype, RequiredValidation::builder())]
            pub wrapped: WrappedValue,
        }
    };

    let expanded = expand_koruma_all_display(input).unwrap();
    let rendered = pretty_print(expanded);
    assert!(rendered.contains("DisplayNewtypeItemWrappedKorumaValidator::Inner(inner)"));
}

#[test]
fn test_koruma_all_display_expansion_uses_path_aware_variant_names() {
    let input: DeriveInput = syn::parse_quote! {
        pub struct DisplayQualifiedValidators {
            #[koruma(foo::RangeValidation::builder().min(0).max(10), bar::RangeValidation::builder().min(11).max(20))]
            pub value: i32,
        }
    };

    let expanded = expand_koruma_all_display(input).unwrap();
    let compact = compact_ws(&pretty_print(expanded));
    assert!(
        compact.contains("DisplayQualifiedValidatorsValueKorumaValidator::FooRangeValidation(v)")
    );
    assert!(
        compact.contains("DisplayQualifiedValidatorsValueKorumaValidator::BarRangeValidation(v)")
    );
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
    assert!(compact.contains("inner:Option<<WrappedValueaskoruma::ValidateExt>::Error>"));
    assert!(compact.contains("pubfnwrapped(&self)->Option<&<WrappedValueaskoruma::ValidateExt>::Error>{self.wrapped.inner.as_ref()}"));
    assert!(
        !compact.contains("implstd::ops::DerefforOptionalNewtypeFieldWrappedKorumaValidationError")
    );
}

#[test]
fn test_koruma_expansion_newtype_with_full_and_unwrapped_validators() {
    let input: DeriveInput = syn::parse_quote! {
        pub struct RichNewtypeField {
            #[koruma(newtype, RequiredValidation::<Option<_>>::builder(), GenericRange::<_>::builder().min(0).max(10), PlainValidation::builder().min(1))]
            pub wrapped: Option<WrappedValue>,
        }
    };

    let expanded = expand_koruma(input).unwrap();
    let compact = compact_ws(&pretty_print(expanded));
    assert!(compact.contains("__koruma_assert_validate_wrapped_required_validation_newtype_field"));
    assert!(compact.contains("koruma::BuilderWithValueRef::with_value_ref("));
    assert!(compact.contains("PlainValidation::builder().min(1)"));
    assert!(compact.contains("inner:Option<<WrappedValueaskoruma::ValidateExt>::Error>"));
    assert!(compact.contains("pubfninner(&self)->Option<&<WrappedValueaskoruma::ValidateExt>::Error>{self.inner.as_ref()}"));
    assert!(compact.contains("error.wrapped.inner=Some(newtype_err);"));
    assert!(compact.contains("inner:None"));
}

#[test]
fn test_koruma_expansion_newtype_non_optional_with_validators_uses_direct_inner_validation() {
    let input: DeriveInput = syn::parse_quote! {
        pub struct DirectNewtypeField {
            #[koruma(newtype, GenericRange::<_>::builder().min(0).max(10), PlainValidation::builder().min(1))]
            pub wrapped: WrappedValue,
        }
    };

    let expanded = expand_koruma(input).unwrap();
    let compact = compact_ws(&pretty_print(expanded));
    assert!(!compact.contains("ifletSome(ref__newtype_value)=self.wrapped"));
    assert!(compact.contains("let__newtype_value=&self.wrapped;"));
    assert!(compact.contains("ifletErr(newtype_err)=__newtype_value.validate()"));
}

#[test]
fn test_koruma_expansion_qualified_validators_generate_distinct_members() {
    let input: DeriveInput = syn::parse_quote! {
        pub struct QualifiedValidators {
            #[koruma(foo::RangeValidation::builder().min(0).max(10), bar::RangeValidation::builder().min(11).max(20))]
            pub value: i32,
        }
    };

    let expanded = expand_koruma(input).unwrap();
    let compact = compact_ws(&pretty_print(expanded));
    assert!(compact.contains("foo_range_validation:Option<foo::RangeValidation>"));
    assert!(compact.contains("bar_range_validation:Option<bar::RangeValidation>"));
    assert!(compact.contains("FooRangeValidation(foo::RangeValidation)"));
    assert!(compact.contains("BarRangeValidation(bar::RangeValidation)"));
}

#[test]
fn test_koruma_expansion_optional_field_with_full_and_unwrapped_validators() {
    let input: DeriveInput = syn::parse_quote! {
        pub struct OptionalMixedValidators {
            #[koruma(RequiredValidation::<Option<_>>::builder(), GenericRange::<_>::builder().min(0).max(10))]
            pub value: Option<i32>,
        }
    };

    let expanded = expand_koruma(input).unwrap();
    let compact = compact_ws(&pretty_print(expanded));
    assert!(compact.contains("ifletSome(ref__field_value)=self.value"));
    assert!(compact.contains("RequiredValidation::<Option<i32>>::builder()"));
}

#[test]
fn test_koruma_expansion_optional_field_with_concrete_full_type_validator() {
    let input: DeriveInput = syn::parse_quote! {
        pub struct OptionalConcreteFullTypeValidator {
            #[koruma(RequiredValidation::<Option<String>>::builder())]
            pub value: Option<String>,
        }
    };

    let expanded = expand_koruma(input).unwrap();
    let compact = compact_ws(&pretty_print(expanded));
    assert!(compact.contains("required_validation:Option<RequiredValidation<Option<String>>>"));
    assert!(compact.contains(
        "BuilderWithValueRef::with_value_ref(RequiredValidation::builder(),&self.value,)"
    ));
    assert!(compact.contains("validator.validate(&self.value)"));
    assert!(!compact.contains("ifletSome(ref__field_value)=self.value"));
}

#[test]
fn test_koruma_expansion_each_optional_element_with_full_type_validator() {
    let input: DeriveInput = syn::parse_quote! {
        pub struct OptionalElementValidators {
            #[koruma(each(RequiredValidation::<Option<_>>::builder()))]
            pub values: Vec<Option<i32>>,
        }
    };

    let expanded = expand_koruma(input).unwrap();
    let compact = compact_ws(&pretty_print(expanded));
    assert!(compact.contains("RequiredValidation::<Option<i32>>::builder()"));
    assert!(compact.contains(
        "BuilderWithValueRef::with_value_ref(RequiredValidation::<Option<i32>>::builder(),item,)"
    ));
    assert!(
        compact.contains(
            "__koruma_assert_validate_values_required_validation_element(&validator,item,)"
        )
    );
    assert!(!compact.contains(
        "__koruma_assert_validate_values_required_validation_element(&validator,__item_value,)"
    ));
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
            #[koruma(each(GenericRange::<Vec<_>>::builder().min(0).max(10)))]
            pub values: Vec<Option<i32>>,
        }
    };

    let expanded = expand_koruma(input).unwrap();
    let compact = compact_ws(&pretty_print(expanded));
    assert!(compact.contains("ifletSome(__item_value)=item"));
    assert!(compact.contains("generic_range:Option<GenericRange<Vec<i32>>>"));
    assert!(compact.contains("GenericRange::<Vec<i32>>::builder()"));
}

#[test]
fn test_koruma_expansion_option_vec_each_uses_inner_collection() {
    let input: DeriveInput = syn::parse_quote! {
        pub struct OptionalVecElementValidators {
            #[koruma(each(GenericRange::<_>::builder().min(0).max(10)))]
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
            #[koruma(each(GenericRange::<_>::builder().min(0).max(10)))]
            pub values: &'a [i32],
        }
    };

    let expanded = expand_koruma(input).unwrap();
    let compact = compact_ws(&pretty_print(expanded));
    assert!(compact.contains("for(idx,__item_value)inself.values.iter().enumerate()"));
    assert!(compact.contains("GenericRange::<i32>::builder()"));
}

#[test]
fn test_koruma_expansion_borrowed_field_carries_lifetimes() {
    let input: DeriveInput = syn::parse_quote! {
        pub struct BorrowedField<'a> {
            #[koruma(StartsWithValidation::<_>::builder().prefix("user:"))]
            pub name: &'a str,
        }
    };

    let expanded = expand_koruma(input).unwrap();
    let compact = compact_ws(&pretty_print(expanded));
    assert!(compact.contains(
        "pubstructBorrowedFieldNameKorumaValidationError<'a>{starts_with_validation:Option<StartsWithValidation<&'astr>>"
    ));
    assert!(compact.contains("pubstructBorrowedFieldKorumaValidationError<'a>{"));
    assert!(compact.contains("StartsWithValidation::<&'astr>::builder()"));
}

#[test]
fn test_koruma_expansion_multi_generic_explicit_type_infers_per_slot() {
    let input: DeriveInput = syn::parse_quote! {
        pub struct MultiGenericInference {
            #[koruma(MultiGenericValidation::<std::collections::HashMap<_, _>>::builder())]
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
            #[koruma(MatchesValidation::builder().matches(password))]
            pub confirm: String,
        }
    };

    let expanded = expand_koruma(input).unwrap();
    let compact = compact_ws(&pretty_print(expanded));
    assert!(compact.contains(".matches(self.password.clone())"));
}

#[test]
fn test_koruma_expansion_builder_chain_uses_supplied_setters() {
    let input: DeriveInput = syn::parse_quote! {
        pub struct BuilderSyntaxItem {
            #[koruma(GenericRangeValidation::<_>::builder().min(0.0).max(100.0))]
            pub score: f64,
        }
    };

    let expanded = expand_koruma(input).unwrap();
    let compact = compact_ws(&pretty_print(expanded));
    assert!(compact.contains("GenericRangeValidation::<f64>::builder().min(0.0).max(100.0)"));
}

#[test]
fn test_koruma_expansion_builder_chain_field_arg_ident_transforms_to_self_clone() {
    let input: DeriveInput = syn::parse_quote! {
        pub struct BuilderMatchesValidationInput {
            pub password: String,
            #[koruma(MatchesValidation::builder().matches(password))]
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
            #[koruma(MatchesValidation::builder().matches(STATIC_MATCH))]
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
            #[koruma(RangeValidation::builder().min(0).max(10))]
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
fn test_koruma_all_display_borrowed_types_carry_lifetimes() {
    let input: DeriveInput = syn::parse_quote! {
        pub struct DisplayBorrowed<'a> {
            #[koruma(StartsWithValidation::<_>::builder().prefix("user:"))]
            pub value: &'a str,
            #[koruma(each(StartsWithValidation::<_>::builder().prefix("tag:")))]
            pub values: &'a [&'a str],
        }
    };

    let expanded = expand_koruma_all_display(input).unwrap();
    let compact = compact_ws(&pretty_print(expanded));
    assert!(
        compact.contains("impl<'a>::std::fmt::DisplayforDisplayBorrowedValueKorumaValidator<'a>")
    );
    assert!(
        compact.contains(
            "impl<'a>::std::fmt::DisplayforDisplayBorrowedValuesElementKorumaValidator<'a>"
        )
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
            input_type = Text,
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
    assert!(compact.contains("input_type:::koruma::showcase::InputType::Text"));
    assert!(compact.contains("module:\"general\""));
    assert!(compact.contains("showcase_validation_builder::State"));
    assert!(compact.contains("S::Actual:koruma::bon::IsUnset"));
    assert!(compact.contains("DynValidatorforShowcaseValidation"));
    assert!(compact.contains("whereU:Default"));
    assert!(compact.contains("Self:::std::marker::Send+::std::marker::Sync"));
    assert!(compact.contains("Self:::koruma::Validate<Option<T>>"));
    assert!(compact.contains("Self:::std::fmt::Display"));
    assert!(!compact.contains("feature=\"internal-showcase\""));
    assert!(!compact.contains("feature=\"fmt\""));
    assert!(!compact.contains("feature=\"fluent\""));
}

#[cfg(feature = "fluent")]
#[test]
fn test_koruma_all_fluent_expansion() {
    let input: DeriveInput = syn::parse_quote! {
        pub struct FluentItem {
            #[koruma(RangeValidation::builder().min(0).max(10), EvenValidation::builder())]
            pub value: i32,
            #[koruma(each(RangeValidation::builder().min(0).max(10)))]
            pub values: Vec<i32>,
        }
    };

    let expanded = expand_koruma_all_fluent(input).unwrap();
    let rendered = pretty_print(expanded);
    assert!(
        rendered.contains("impl ::es_fluent::FluentMessage for FluentItemValueKorumaValidator")
    );
    assert!(
        rendered
            .contains("impl ::es_fluent::FluentMessage for FluentItemValuesElementKorumaValidator")
    );
}

#[cfg(feature = "fluent")]
#[test]
fn test_koruma_all_fluent_borrowed_types_carry_lifetimes() {
    let input: DeriveInput = syn::parse_quote! {
        pub struct FluentBorrowed<'a> {
            #[koruma(StartsWithValidation::<_>::builder().prefix("user:"))]
            pub value: &'a str,
            #[koruma(each(StartsWithValidation::<_>::builder().prefix("tag:")))]
            pub values: &'a [&'a str],
        }
    };

    let expanded = expand_koruma_all_fluent(input).unwrap();
    let compact = compact_ws(&pretty_print(expanded));
    assert!(
        compact.contains(
            "impl<'a>::es_fluent::FluentMessageforFluentBorrowedValueKorumaValidator<'a>"
        )
    );
    assert!(compact.contains(
        "impl<'a>::es_fluent::FluentMessageforFluentBorrowedValuesElementKorumaValidator<'a>"
    ));
}

#[cfg(feature = "fluent")]
#[test]
fn test_koruma_all_fluent_handles_skipped_fields() {
    let input: DeriveInput = syn::parse_quote! {
        pub struct FluentWithSkippedField {
            pub plain: i32,
            #[koruma(RangeValidation::builder().min(0).max(10))]
            pub value: i32,
        }
    };

    let expanded = expand_koruma_all_fluent(input).unwrap();
    let rendered = pretty_print(expanded);
    assert!(rendered.contains(
        "impl ::es_fluent::FluentMessage for FluentWithSkippedFieldValueKorumaValidator"
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
    assert!(!rendered.contains("impl ::es_fluent::FluentMessage for"));
}

#[cfg(feature = "fluent")]
#[test]
fn test_koruma_all_fluent_expansion_newtype_inner_delegate() {
    let input: DeriveInput = syn::parse_quote! {
        pub struct FluentNewtypeItem {
            #[koruma(newtype, RequiredValidation::builder())]
            pub wrapped: WrappedValue,
        }
    };

    let expanded = expand_koruma_all_fluent(input).unwrap();
    let rendered = pretty_print(expanded);
    assert!(rendered.contains("if !self.inner().is_empty()"));
    assert!(rendered.contains("messages.push(self.inner().to_fluent_string_with(localize))"));
}

#[cfg(feature = "fluent")]
#[test]
fn test_koruma_all_fluent_expansion_optional_newtype_inner_delegate() {
    let input: DeriveInput = syn::parse_quote! {
        pub struct FluentOptionalNewtypeItem {
            #[koruma(newtype)]
            pub wrapped: Option<WrappedValue>,
        }
    };

    let expanded = expand_koruma_all_fluent(input).unwrap();
    let rendered = pretty_print(expanded);
    assert!(rendered.contains("if let Some(inner) = self.inner()"));
    assert!(rendered.contains("messages.push(inner.to_fluent_string_with(localize))"));
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
        pub struct Email(#[koruma(NonEmptyStringValidation::builder())] pub String);
    };

    let expanded = expand_koruma(input).unwrap();
    let rendered = pretty_print(expanded);
    assert_snapshot!(rendered);
}

#[test]
fn test_koruma_expansion_try_from_bare_tuple_struct() {
    let input: DeriveInput = syn::parse_quote! {
        #[koruma(newtype(try_from))]
        pub struct Email(pub String);
    };

    let expanded = expand_koruma(input).unwrap();
    let compact = compact_ws(&pretty_print(expanded));
    assert!(compact.contains("implTryFrom<String>forEmail"));
    assert!(compact.contains("ifletErr(newtype_err)=self.0.validate()"));
    assert!(compact.contains("error._0.inner=newtype_err;"));
    assert!(compact.contains("implcore::ops::DerefforEmailKorumaValidationError"));
}

#[test]
fn test_koruma_expansion_try_from_named_field_struct() {
    let input: DeriveInput = syn::parse_quote! {
        #[koruma(newtype(try_from))]
        pub struct Username {
            #[koruma(NonEmptyStringValidation::builder())]
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
        pub struct Wrapper<T>(#[koruma(GenericRange::<_>::builder().min(0).max(100))] pub T);
    };

    let expanded = expand_koruma(input).unwrap();
    let rendered = pretty_print(expanded);
    assert_snapshot!(rendered);
}

#[test]
fn test_koruma_expansion_try_from_generic_with_bounds() {
    let input: DeriveInput = syn::parse_quote! {
        #[koruma(newtype(try_from))]
        pub struct BoundedWrapper<T: Clone>(#[koruma(GenericRange::<_>::builder().min(0).max(100))] pub T);
    };

    let expanded = expand_koruma(input).unwrap();
    let rendered = pretty_print(expanded);
    assert_snapshot!(rendered);
}

#[test]
fn test_koruma_expansion_try_from_generic_with_where_clause() {
    let input: DeriveInput = syn::parse_quote! {
        #[koruma(newtype(try_from))]
        pub struct WhereWrapper<T>(#[koruma(GenericRange::<_>::builder().min(0).max(100))] pub T) where T: Default;
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
            #[koruma(newtype, RequiredValidation::<Option<_>>::builder())]
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
            #[koruma(NonEmptyStringValidation::builder())]
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
