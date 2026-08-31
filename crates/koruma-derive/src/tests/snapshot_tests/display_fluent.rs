#[test]
fn test_koruma_all_display_expansion() {
    let input: DeriveInput = syn::parse_quote! {
        pub struct DisplayItem {
            #[koruma(RangeValidation.min(0).max(10), EvenValidation)]
            pub value: i32,
            #[koruma(each(RangeValidation.min(0).max(10)))]
            pub values: Vec<i32>,
        }
    };

    let expanded = expand_koruma_all_display(input).unwrap();
    let rendered = compact_ws(&pretty_print(expanded));
    assert!(rendered.contains("DisplayforDisplayItemValueKorumaValidatorRef"));
    assert!(rendered.contains("DisplayforDisplayItemValuesElementKorumaValidatorRef"));
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
    assert!(rendered.contains("DisplayNewtypeItemWrappedKorumaValidatorRef::Inner(inner)"));
}

#[test]
fn test_koruma_all_display_expansion_uses_labeled_variant_names() {
    let input: DeriveInput = syn::parse_quote! {
        pub struct DisplayQualifiedValidators {
            #[koruma(
                low_range = foo::RangeValidation.min(0).max(10),
                high_range = bar::RangeValidation.min(11).max(20),
            )]
            pub value: i32,
        }
    };

    let expanded = expand_koruma_all_display(input).unwrap();
    let compact = compact_ws(&pretty_print(expanded));
    assert!(compact.contains("DisplayQualifiedValidatorsValueKorumaValidatorRef::LowRange(v)"));
    assert!(compact.contains("DisplayQualifiedValidatorsValueKorumaValidatorRef::HighRange(v)"));
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
fn test_koruma_all_display_handles_skipped_fields() {
    let input: DeriveInput = syn::parse_quote! {
        pub struct DisplayWithSkippedField {
            pub plain: i32,
            #[koruma(RangeValidation.min(0).max(10))]
            pub value: i32,
        }
    };

    let expanded = expand_koruma_all_display(input).unwrap();
    let rendered = compact_ws(&pretty_print(expanded));
    assert!(rendered.contains("DisplayforDisplayWithSkippedFieldValueKorumaValidatorRef"));
}

#[test]
fn test_koruma_all_display_borrowed_types_carry_lifetimes() {
    let input: DeriveInput = syn::parse_quote! {
        pub struct DisplayBorrowed<'a> {
            #[koruma(StartsWithValidation::<_>.prefix("user:"))]
            pub value: &'a str,
            #[koruma(each(StartsWithValidation::<_>.prefix("tag:")))]
            pub values: &'a [&'a str],
        }
    };

    let expanded = expand_koruma_all_display(input).unwrap();
    let compact = compact_ws(&pretty_print(expanded));
    assert!(compact.contains(
        "impl<'koruma,'a>::std::fmt::DisplayforDisplayBorrowedValueKorumaValidatorRef<'koruma,'a>"
    ));
    assert!(
        compact.contains(
            "impl<'koruma,'a>::std::fmt::DisplayforDisplayBorrowedValuesElementKorumaValidatorRef<'koruma,'a>"
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


#[cfg(feature = "fluent")]
#[test]
fn test_koruma_all_fluent_expansion() {
    let input: DeriveInput = syn::parse_quote! {
        pub struct FluentItem {
            #[koruma(RangeValidation.min(0).max(10), EvenValidation)]
            pub value: i32,
            #[koruma(each(RangeValidation.min(0).max(10)))]
            pub values: Vec<i32>,
        }
    };

    let expanded = expand_koruma_all_fluent(input).unwrap();
    let rendered = compact_ws(&pretty_print(expanded));
    assert!(rendered.contains("FluentMessageforFluentItemValueKorumaValidatorRef"));
    assert!(rendered.contains("FluentMessageforFluentItemValuesElementKorumaValidatorRef"));
}

#[cfg(feature = "fluent")]
#[test]
fn test_koruma_all_fluent_borrowed_types_carry_lifetimes() {
    let input: DeriveInput = syn::parse_quote! {
        pub struct FluentBorrowed<'a> {
            #[koruma(StartsWithValidation::<_>.prefix("user:"))]
            pub value: &'a str,
            #[koruma(each(StartsWithValidation::<_>.prefix("tag:")))]
            pub values: &'a [&'a str],
        }
    };

    let expanded = expand_koruma_all_fluent(input).unwrap();
    let compact = compact_ws(&pretty_print(expanded));
    assert!(
        compact.contains(
            "impl<'koruma,'a>::es_fluent::FluentMessageforFluentBorrowedValueKorumaValidatorRef<'koruma,'a>"
        )
    );
    assert!(compact.contains(
        "impl<'koruma,'a>::es_fluent::FluentMessageforFluentBorrowedValuesElementKorumaValidatorRef<'koruma,'a>"
    ));
}

#[cfg(feature = "fluent")]
#[test]
fn test_koruma_all_fluent_handles_skipped_fields() {
    let input: DeriveInput = syn::parse_quote! {
        pub struct FluentWithSkippedField {
            pub plain: i32,
            #[koruma(RangeValidation.min(0).max(10))]
            pub value: i32,
        }
    };

    let expanded = expand_koruma_all_fluent(input).unwrap();
    let rendered = compact_ws(&pretty_print(expanded));
    assert!(rendered.contains("FluentMessageforFluentWithSkippedFieldValueKorumaValidatorRef"));
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
            #[koruma(newtype, RequiredValidation)]
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
