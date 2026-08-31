use super::support::*;

#[test]
fn test_find_value_field_finds_marked_field() {
    let input: ItemStruct = syn::parse_quote! {
        pub struct Test {
            min: i32,
            max: i32,
            #[koruma(value)]
            checked: Option<i32>,
        }
    };

    let result = parse_validator_struct(&input).expect("expected validator struct parse");
    assert_eq!(result.value_field().name().to_string(), "checked");
}

#[test]
fn test_parse_validator_struct_errors_when_missing_value() {
    let input: ItemStruct = syn::parse_quote! {
        pub struct Test {
            #[koruma(setter)]
            min: i32,
            #[koruma(setter)]
            max: i32,
            #[koruma(setter)]
            checked: Option<i32>,
        }
    };

    assert!(
        parse_validator_struct(&input)
            .expect_err("expected missing value field")
            .to_string()
            .contains("requires a value field")
    );
}

#[test]
fn test_parse_field_with_single_validator() {
    let field: syn::Field = syn::parse_quote! {
        #[koruma(RangeValidation.min(0).max(100))]
        pub age: i32
    };

    let ParsedDataField::Participating(info) =
        parse_field(&field, 0).expect("expected field parse")
    else {
        panic!("expected validated field")
    };
    assert_eq!(info.name().to_string(), "age");
    assert_eq!(info.field_validators().len(), 1);
    assert_eq!(
        info.field_validators()[0].validator().name().to_string(),
        "RangeValidation"
    );
    assert!(!info.field_validators()[0].validator().uses_type_inference());
    assert_eq!(
        info.field_validators()[0].validator().setter_calls().len(),
        2
    );
    assert!(info.element_validators().is_empty());
}

#[test]
fn test_parse_field_with_generic_validator() {
    let field: syn::Field = syn::parse_quote! {
        #[koruma(GenericRange::<_>.min(0.0).max(1.0))]
        pub score: f64
    };

    let ParsedDataField::Participating(info) =
        parse_field(&field, 0).expect("expected field parse")
    else {
        panic!("expected validated field")
    };
    assert!(info.field_validators()[0].validator().uses_type_inference());
}

#[test]
fn test_parse_field_with_each() {
    let field: syn::Field = syn::parse_quote! {
        #[koruma(each(RangeValidation.min(0).max(100)))]
        pub scores: Vec<i32>
    };

    let ParsedDataField::Participating(info) =
        parse_field(&field, 0).expect("expected field parse")
    else {
        panic!("expected validated field")
    };
    assert!(info.field_validators().is_empty());
    assert_eq!(info.element_validators().len(), 1);
}

#[test]
fn test_parse_field_with_skip_returns_skip() {
    let field: syn::Field = syn::parse_quote! {
        #[koruma(skip)]
        pub internal: u64
    };

    assert!(matches!(
        parse_field(&field, 0).expect("expected field parse"),
        ParsedDataField::Skipped { .. }
    ));
}

#[test]
fn test_parse_field_without_koruma_returns_skip() {
    let field: syn::Field = syn::parse_quote! {
        pub normal_field: String
    };

    assert!(matches!(
        parse_field(&field, 0).expect("expected field parse"),
        ParsedDataField::Unannotated(_)
    ));
}
