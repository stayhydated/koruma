#[test]
fn test_koruma_expansion_optional_field() {
    // Optional field should generate if-let pattern and skip validation when None
    let input: DeriveInput = syn::parse_quote! {
        pub struct UserProfile {
            #[koruma(StringLengthValidation.min(1).max(50))]
            pub username: String,

            #[koruma(StringLengthValidation.min(1).max(200))]
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
            #[koruma(GenericRange::<_>.min(0).max(100))]
            pub score: Option<i32>,
        }
    };

    let expanded = expand_koruma(input).unwrap();
    assert_snapshot!(pretty_print(expanded));
}

#[test]
fn test_koruma_expansion_full_optional_field_target_path() {
    let input: DeriveInput = syn::parse_quote! {
        pub struct OptionalPresence {
            #[koruma(RequiredValidation::<Option<_>>)]
            pub value: Option<String>,
        }
    };

    let expanded = expand_koruma(input).unwrap();
    assert_snapshot!(pretty_print(expanded));
}

#[test]
fn test_koruma_expansion_optional_element_target_path() {
    let input: DeriveInput = syn::parse_quote! {
        pub struct OptionalScores {
            #[koruma(each(GenericRangeValidation::<_>.min(0).max(10)))]
            pub values: Vec<Option<i32>>,
        }
    };

    let expanded = expand_koruma(input).unwrap();
    assert_snapshot!(pretty_print(expanded));
}

#[test]
fn test_koruma_expansion_full_optional_element_target_path() {
    let input: DeriveInput = syn::parse_quote! {
        pub struct OptionalElementPresence {
            #[koruma(each(RequiredValidation::<Option<_>>))]
            pub values: Vec<Option<i32>>,
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
            #[koruma(VecLenValidation::<i32>.min(1).max(10), each(RangeValidation::<_>.min(0).max(100)))]
            pub scores: Vec<i32>,
        }
    };

    let expanded = expand_koruma(input).unwrap();
    assert_snapshot!(pretty_print(expanded));
}

#[test]
fn test_koruma_expansion_only_element_validators() {
    // Only element validators (no field-level validators)
    let input: DeriveInput = syn::parse_quote! {
        pub struct Scores {
            #[koruma(each(RangeValidation::<_>.min(0).max(100)))]
            pub values: Vec<i32>,
        }
    };

    let expanded = expand_koruma(input).unwrap();
    assert_snapshot!(pretty_print(expanded));
}


#[test]
fn test_koruma_expansion_optional_field_with_full_and_unwrapped_validators() {
    let input: DeriveInput = syn::parse_quote! {
        pub struct OptionalMixedValidators {
            #[koruma(RequiredValidation::<Option<_>>, GenericRange::<_>.min(0).max(10))]
            pub value: Option<i32>,
        }
    };

    let expanded = expand_koruma(input).unwrap();
    let compact = compact_ws(&pretty_print(expanded));
    assert!(compact.contains("ifletSome(ref__field_value)=self.value"));
    assert!(compact.contains("RequiredValidation::<Option<i32>>"));
}

#[test]
fn test_koruma_expansion_optional_field_with_concrete_full_type_validator() {
    let input: DeriveInput = syn::parse_quote! {
        pub struct OptionalConcreteFullTypeValidator {
            #[koruma(RequiredValidation::<Option<String>>)]
            pub value: Option<String>,
        }
    };

    let expanded = expand_koruma(input).unwrap();
    let compact = compact_ws(&pretty_print(expanded));
    assert!(compact.contains("required_validation:Option<RequiredValidation<Option<String>>>"));
    assert!(compact.contains("let__koruma_builder=RequiredValidation::__koruma_builder();"));
    assert!(compact.contains("__private::assert_field_validator_ready::<_,Option<String>,RequiredValidation<Option<String>>,>(&__koruma_builder);"));
    assert!(
        compact.contains(
            "__private::CaptureValueRef::capture_value_ref(__koruma_builder,&self.value,)"
        )
    );
    assert!(compact.contains("::renamed_koruma::Validate<Option<String>,>"));
    assert!(compact.contains("::validate(&validator,&self.value)"));
    assert!(!compact.contains("ifletSome(ref__field_value)=self.value"));
}

#[test]
fn test_koruma_expansion_each_optional_element_with_full_type_validator() {
    let input: DeriveInput = syn::parse_quote! {
        pub struct OptionalElementValidators {
            #[koruma(each(RequiredValidation::<Option<_>>))]
            pub values: Vec<Option<i32>>,
        }
    };

    let expanded = expand_koruma(input).unwrap();
    let compact = compact_ws(&pretty_print(expanded));
    assert!(compact.contains("RequiredValidation<Option<i32>>"));
    assert!(
        compact.contains("__private::CaptureValueRef::capture_value_ref(__koruma_builder,item,)")
    );
    assert!(
        compact.contains("as::renamed_koruma::Validate<Option<i32>,>>::validate(&validator,item)")
    );
    assert!(!compact.contains("__koruma_assert_validate_values_required_validation_element"));
}


#[test]
fn test_koruma_expansion_vec_option_each_with_explicit_infer_type() {
    let input: DeriveInput = syn::parse_quote! {
        pub struct VecOptionElementValidators {
            #[koruma(each(GenericRange::<Vec<_>>.min(0).max(10)))]
            pub values: Vec<Option<i32>>,
        }
    };

    let expanded = expand_koruma(input).unwrap();
    let compact = compact_ws(&pretty_print(expanded));
    assert!(compact.contains("ifletSome(__item_value)=item"));
    assert!(compact.contains("generic_range:Option<GenericRange<Vec<i32>>>"));
    assert!(compact.contains("GenericRange::<Vec<i32>>"));
}

#[test]
fn test_koruma_expansion_option_vec_each_uses_inner_collection() {
    let input: DeriveInput = syn::parse_quote! {
        pub struct OptionalVecElementValidators {
            #[koruma(each(GenericRange::<_>.min(0).max(10)))]
            pub values: Option<Vec<i32>>,
        }
    };

    let expanded = expand_koruma(input).unwrap();
    let compact = compact_ws(&pretty_print(expanded));
    assert!(compact.contains("ifletSome(ref__collection_value)=self.values"));
    assert!(compact.contains("for(idx,__item_value)in__collection_value.iter().enumerate()"));
    assert!(compact.contains("GenericRange::<i32>"));
}
#[test]
fn test_koruma_expansion_slice_each_uses_element_type() {
    let input: DeriveInput = syn::parse_quote! {
        pub struct BorrowedSliceElementValidators<'a> {
            #[koruma(each(GenericRange::<_>.min(0).max(10)))]
            pub values: &'a [i32],
        }
    };

    let expanded = expand_koruma(input).unwrap();
    let compact = compact_ws(&pretty_print(expanded));
    assert!(compact.contains("for(idx,__item_value)inself.values.iter().enumerate()"));
    assert!(compact.contains("GenericRange::<i32>"));
}
