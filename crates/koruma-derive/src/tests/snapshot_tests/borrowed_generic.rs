#[test]
fn test_koruma_expansion_borrowed_field_carries_lifetimes() {
    let input: DeriveInput = syn::parse_quote! {
        pub struct BorrowedField<'a> {
            #[koruma(StartsWithValidation::<_>.prefix("user:"))]
            pub name: &'a str,
        }
    };

    let expanded = expand_koruma(input).unwrap();
    let compact = compact_ws(&pretty_print(expanded));
    assert!(compact.contains(
        "pubstructBorrowedFieldNameKorumaValidationError<'a>{starts_with_validation:Option<StartsWithValidation<&'astr>>"
    ));
    assert!(compact.contains("pubstructBorrowedFieldKorumaValidationError<'a>{"));
    assert!(compact.contains("StartsWithValidation::<&'astr>"));
}

#[test]
fn test_koruma_expansion_borrowed_explicit_reference_infer() {
    let input: DeriveInput = syn::parse_quote! {
        pub struct BorrowedField<'a> {
            #[koruma(StartsWithValidation::<&_>.prefix("user:"))]
            pub name: &'a str,
        }
    };

    let expanded = expand_koruma(input).unwrap();
    let compact = compact_ws(&pretty_print(expanded));
    assert!(compact.contains(
        "pubstructBorrowedFieldNameKorumaValidationError<'a>{starts_with_validation:Option<StartsWithValidation<&'astr>>"
    ));
    assert!(compact.contains("StartsWithValidation::<&'astr>"));
}

#[test]
fn test_koruma_expansion_multi_generic_explicit_type_infers_per_slot() {
    let input: DeriveInput = syn::parse_quote! {
        pub struct MultiGenericInference {
            #[koruma(MultiGenericValidation::<std::collections::HashMap<_, _>>)]
            pub values: std::collections::HashMap<String, i32>,
        }
    };

    let expanded = expand_koruma(input).unwrap();
    let compact = compact_ws(&pretty_print(expanded));
    assert!(compact.contains("MultiGenericValidation<std::collections::HashMap<String,i32>>"));
}
