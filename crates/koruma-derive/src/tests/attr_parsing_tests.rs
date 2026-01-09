//! Tests for ValidatorAttr and KorumaAttr parsing.

use crate::expand::parse::*;
use quote::quote;

#[test]
fn test_validator_attr_parse_simple() {
    let attr: ValidatorAttr = syn::parse_quote!(RangeValidation);
    assert_eq!(attr.validator.to_string(), "RangeValidation");
    assert!(!attr.infer_type);
    assert!(attr.args.is_empty());
}

#[test]
fn test_validator_attr_parse_with_args() {
    let attr: ValidatorAttr = syn::parse_quote!(RangeValidation(min = 0, max = 100));
    assert_eq!(attr.validator.to_string(), "RangeValidation");
    assert!(!attr.infer_type);
    assert_eq!(attr.args.len(), 2);
    assert_eq!(attr.args[0].0.to_string(), "min");
    assert_eq!(attr.args[1].0.to_string(), "max");
}

#[test]
fn test_validator_attr_parse_generic() {
    // Turbofish syntax: ::<_>
    let attr: ValidatorAttr = syn::parse_quote!(GenericRange::<_>(min = 0.0, max = 1.0));
    assert_eq!(attr.validator.to_string(), "GenericRange");
    assert!(attr.infer_type);
    assert_eq!(attr.args.len(), 2);
}

#[test]
fn test_koruma_attr_parse_skip() {
    let attr: KorumaAttr = syn::parse_quote!(skip);
    assert!(attr.is_skip);
    assert!(attr.field_validators.is_empty());
    assert!(attr.element_validators.is_empty());
}

#[test]
fn test_koruma_attr_parse_each() {
    let attr: KorumaAttr = syn::parse_quote!(each(RangeValidation(min = 0, max = 100)));
    assert!(!attr.is_skip);
    assert!(attr.field_validators.is_empty());
    assert_eq!(attr.element_validators.len(), 1);
}

#[test]
fn test_koruma_attr_parse_multiple_validators() {
    // Turbofish syntax: ::<_>
    let attr: KorumaAttr = syn::parse_quote!(ValidatorA(x = 1), ValidatorB, ValidatorC::<_>(y = 2));
    assert!(!attr.is_skip);
    assert_eq!(attr.field_validators.len(), 3);
    assert!(attr.element_validators.is_empty());
    assert!(!attr.field_validators[0].infer_type);
    assert!(!attr.field_validators[1].infer_type);
    assert!(attr.field_validators[2].infer_type);
}

#[test]
fn test_koruma_attr_parse_combined_field_and_each() {
    // Combined: field validator + each(element validators) with turbofish
    let attr: KorumaAttr = syn::parse_quote!(
        LenValidator(min = 1, max = 10),
        each(RangeValidation::<_>(min = 0, max = 100))
    );
    assert!(!attr.is_skip);
    assert_eq!(attr.field_validators.len(), 1);
    assert_eq!(
        attr.field_validators[0].validator.to_string(),
        "LenValidator"
    );
    assert_eq!(attr.element_validators.len(), 1);
    assert_eq!(
        attr.element_validators[0].validator.to_string(),
        "RangeValidation"
    );
    assert!(attr.element_validators[0].infer_type);
}

#[test]
fn test_koruma_attr_parse_each_then_field() {
    // each() can come before field validators too
    let attr: KorumaAttr = syn::parse_quote!(
        each(RangeValidation(min = 0, max = 100)),
        LenValidator(min = 1)
    );
    assert!(!attr.is_skip);
    assert_eq!(attr.field_validators.len(), 1);
    assert_eq!(attr.element_validators.len(), 1);
}

#[test]
fn test_validator_attr_parse_nested_generic() {
    // Nested generics with turbofish: ::<Option<_>>
    let attr: ValidatorAttr = syn::parse_quote!(RequiredValidation::<Option<_>>);
    assert_eq!(attr.validator.to_string(), "RequiredValidation");
    assert!(!attr.infer_type);
    assert!(attr.explicit_type.is_some());

    let explicit_ty = attr.explicit_type.unwrap();
    let ty_str = quote::quote!(#explicit_ty).to_string();
    assert!(
        ty_str.contains("Option"),
        "expected Option<_>, got: {}",
        ty_str
    );
}

#[test]
fn test_validator_attr_parse_nested_generic_concrete() {
    // Nested generics with concrete types: ::<Vec<String>>
    let attr: ValidatorAttr = syn::parse_quote!(SomeValidator::<Vec<String>>);
    assert_eq!(attr.validator.to_string(), "SomeValidator");
    assert!(!attr.infer_type);
    assert!(attr.explicit_type.is_some());

    let explicit_ty = attr.explicit_type.unwrap();
    let ty_str = quote::quote!(#explicit_ty).to_string();
    assert!(
        ty_str.contains("Vec") && ty_str.contains("String"),
        "expected Vec<String>, got: {}",
        ty_str
    );
}

#[test]
fn test_validator_attr_parse_deeply_nested_generic() {
    // Deeply nested generics: ::<Option<Vec<_>>>
    let attr: ValidatorAttr = syn::parse_quote!(DeepValidator::<Option<Vec<_>>>);
    assert_eq!(attr.validator.to_string(), "DeepValidator");
    assert!(!attr.infer_type);
    assert!(attr.explicit_type.is_some());
}

#[test]
fn test_validator_attr_parse_option_infer_type() {
    // ::<Option<_>> syntax for full Option type (no unwrapping)
    let attr: ValidatorAttr = syn::parse_quote!(RequiredValidation::<Option<_>>);
    assert_eq!(attr.validator.to_string(), "RequiredValidation");
    assert!(!attr.infer_type);
    assert!(attr.explicit_type.is_some());
    let explicit_ty = attr.explicit_type.unwrap();
    let ty_str = quote::quote!(#explicit_ty).to_string();
    assert!(
        ty_str.contains("Option"),
        "expected Option<_>, got: {}",
        ty_str
    );
}

#[test]
fn test_validator_attr_parse_old_syntax_error() {
    // Old <_> syntax without :: should give helpful error
    let result: Result<ValidatorAttr, _> = syn::parse_str("Validator<_>");
    assert!(result.is_err(), "expected error for old syntax without ::");
    let err = result.err().unwrap().to_string();
    assert!(
        err.contains("turbofish"),
        "expected helpful error about turbofish, got: {}",
        err
    );
}
