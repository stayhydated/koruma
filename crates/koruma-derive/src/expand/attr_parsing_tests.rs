//! Tests for ValidatorAttr and KorumaAttr parsing.

use super::*;

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
    let attr: ValidatorAttr = syn::parse_quote!(GenericRange<_>(min = 0.0, max = 1.0));
    assert_eq!(attr.validator.to_string(), "GenericRange");
    assert!(attr.infer_type);
    assert_eq!(attr.args.len(), 2);
}

#[test]
fn test_koruma_attr_parse_skip() {
    let attr: KorumaAttr = syn::parse_quote!(skip);
    assert!(attr.is_skip);
    assert!(!attr.validate_each);
    assert!(attr.validators.is_empty());
}

#[test]
fn test_koruma_attr_parse_each() {
    let attr: KorumaAttr = syn::parse_quote!(each(RangeValidation(min = 0, max = 100)));
    assert!(!attr.is_skip);
    assert!(attr.validate_each);
    assert_eq!(attr.validators.len(), 1);
}

#[test]
fn test_koruma_attr_parse_multiple_validators() {
    let attr: KorumaAttr = syn::parse_quote!(ValidatorA(x = 1), ValidatorB, ValidatorC<_>(y = 2));
    assert!(!attr.is_skip);
    assert!(!attr.validate_each);
    assert_eq!(attr.validators.len(), 3);
    assert!(!attr.validators[0].infer_type);
    assert!(!attr.validators[1].infer_type);
    assert!(attr.validators[2].infer_type);
}
