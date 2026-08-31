#[test]
fn test_koruma_expansion_try_new() {
    // Struct with #[koruma(try_new)] generates a try_new constructor
    let input: DeriveInput = syn::parse_quote! {
        #[koruma(try_new)]
        pub struct Person {
            #[koruma(RangeValidation.min(0).max(150))]
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
fn test_koruma_expansion_try_from_tuple_struct() {
    let input: DeriveInput = syn::parse_quote! {
        #[koruma(newtype, try_from)]
        pub struct Email(#[koruma(NonEmptyStringValidation)] pub String);
    };

    let expanded = expand_koruma(input).unwrap();
    let rendered = pretty_print(expanded);
    assert_snapshot!(rendered);
}

#[test]
fn test_koruma_expansion_try_from_bare_tuple_struct() {
    let input: DeriveInput = syn::parse_quote! {
        #[koruma(newtype, try_from)]
        pub struct Email(pub String);
    };

    let expanded = expand_koruma(input).unwrap();
    let compact = compact_ws(&pretty_print(expanded));
    assert!(compact.contains("implTryFrom<String>forEmail"));
    assert!(compact.contains("NewtypeValueforEmail"));
    assert!(compact.contains("fnvalidate_inner"));
    assert!(compact.contains("implcore::ops::DerefforEmailKorumaValidationError"));
}

#[test]
fn test_koruma_expansion_try_from_named_field_struct() {
    let input: DeriveInput = syn::parse_quote! {
        #[koruma(newtype, try_from)]
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
fn test_koruma_expansion_regular_try_from_named_field_struct() {
    let input: DeriveInput = syn::parse_quote! {
        #[koruma(try_from)]
        pub struct EmailAddress {
            #[koruma(EmailValidation)]
            pub value: String,
        }
    };

    let expanded = expand_koruma(input).unwrap();
    let compact = compact_ws(&pretty_print(expanded));
    assert!(compact.contains("implTryFrom<String>forEmailAddress"));
    assert!(compact.contains("letinstance=Self{value:value};"));
    assert!(compact.contains("instance.validate()?;"));
    assert!(!compact.contains("implcore::ops::DerefforEmailAddressKorumaValidationError"));
}

#[test]
fn test_koruma_expansion_regular_try_from_tuple_struct() {
    let input: DeriveInput = syn::parse_quote! {
        #[koruma(try_from)]
        pub struct EmailAddress(#[koruma(EmailValidation)] pub String);
    };

    let expanded = expand_koruma(input).unwrap();
    let compact = compact_ws(&pretty_print(expanded));
    assert!(compact.contains("implTryFrom<String>forEmailAddress"));
    assert!(compact.contains("letinstance=Self(value);"));
    assert!(compact.contains("instance.validate()?;"));
    assert!(!compact.contains("implcore::ops::DerefforEmailAddressKorumaValidationError"));
}

#[test]
fn test_koruma_expansion_newtype_try_new_and_try_from() {
    let input: DeriveInput = syn::parse_quote! {
        #[koruma(newtype, try_new, try_from)]
        pub struct Username(#[koruma(NonEmptyStringValidation)] pub String);
    };

    let expanded = expand_koruma(input).unwrap();
    let compact = compact_ws(&pretty_print(expanded));
    assert!(
        compact.contains("pubfntry_new(_0:String)->Result<Self,UsernameKorumaValidationError>")
    );
    assert!(compact.contains("implTryFrom<String>forUsername"));
    assert!(compact.contains("implcore::ops::DerefforUsernameKorumaValidationError"));
}

#[test]
fn test_koruma_expansion_try_from_generic() {
    let input: DeriveInput = syn::parse_quote! {
        #[koruma(newtype, try_from)]
        pub struct Wrapper<T>(#[koruma(GenericRange::<_>.min(0).max(100))] pub T);
    };

    let expanded = expand_koruma(input).unwrap();
    let rendered = pretty_print(expanded);
    assert_snapshot!(rendered);
}

#[test]
fn test_koruma_expansion_try_from_generic_with_bounds() {
    let input: DeriveInput = syn::parse_quote! {
        #[koruma(newtype, try_from)]
        pub struct BoundedWrapper<T: Clone>(#[koruma(GenericRange::<_>.min(0).max(100))] pub T);
    };

    let expanded = expand_koruma(input).unwrap();
    let rendered = pretty_print(expanded);
    assert_snapshot!(rendered);
}

#[test]
fn test_koruma_expansion_try_from_generic_with_where_clause() {
    let input: DeriveInput = syn::parse_quote! {
        #[koruma(newtype, try_from)]
        pub struct WhereWrapper<T>(#[koruma(GenericRange::<_>.min(0).max(100))] pub T) where T: Default;
    };

    let expanded = expand_koruma(input).unwrap();
    let rendered = pretty_print(expanded);
    assert_snapshot!(rendered);
}

#[test]
fn test_koruma_expansion_try_from_option_field() {
    let input: DeriveInput = syn::parse_quote! {
        #[koruma(newtype, try_from)]
        pub struct OptionalWrapper {
            #[koruma(newtype, RequiredValidation::<Option<_>>)]
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
        #[koruma(try_from)]
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
        err.to_string().contains(
            "try_from requires exactly one field; use try_new for multi-field constructors"
        )
    );
}
