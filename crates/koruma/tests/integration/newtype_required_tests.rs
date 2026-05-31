// Tests for newtype fields with RequiredValidation

use koruma::ValidationError;

use super::fixtures::{ContainsNewtypeWithValidators, ContainsRequiredNewtype, PositiveNumber};

#[test]
fn test_required_newtype_valid() {
    let item = ContainsRequiredNewtype {
        age: Some(PositiveNumber { value: 25 }),
    };

    assert!(item.validate().is_ok());
}

#[test]
fn test_required_newtype_none() {
    let item = ContainsRequiredNewtype { age: None };

    let err = item.validate().unwrap_err();

    // Should have required_validation error
    assert!(err.age().required_validation().is_some());
    // Should NOT have inner error because the value was None
    assert!(err.age().inner().is_none());
    // Can use all() to get all failed validators
    assert_eq!(err.age().all().count(), 1);
}

#[test]
fn test_required_newtype_invalid_inner() {
    let item = ContainsRequiredNewtype {
        age: Some(PositiveNumber { value: -5 }), // Invalid: negative number
    };

    let err = item.validate().unwrap_err();

    // Should NOT have required_validation error because value is Some
    assert!(err.age().required_validation().is_none());
    // Should have inner error from the newtype validation
    let inner = err.age().inner().expect("expected inner newtype error");
    assert!(inner.number_range_validation().is_some());
    assert!(!inner.is_empty());
}

#[test]
fn test_newtype_with_validators_all_method() {
    let item = ContainsRequiredNewtype { age: None };

    let err = item.validate().unwrap_err();

    // The all() method returns all failed validators
    let all_validators: Vec<_> = err.age().all().collect();
    assert_eq!(all_validators.len(), 1);

    // Verify the validator is present
    assert_eq!(all_validators.len(), 1);
}

#[test]
fn test_newtype_with_validators_valid() {
    let item = ContainsNewtypeWithValidators {
        age: Some(PositiveNumber { value: 50 }),
        name: "Test".to_string(),
    };

    assert!(item.validate().is_ok());
}

#[test]
fn test_newtype_with_validators_is_empty() {
    // Test that is_empty() works correctly for newtype fields with validators
    let item = ContainsRequiredNewtype { age: None };

    let err = item.validate().unwrap_err();
    assert!(!err.is_empty());
    assert!(err.age().has_errors());

    // Valid case should be empty
    let valid_item = ContainsRequiredNewtype {
        age: Some(PositiveNumber { value: 10 }),
    };
    assert!(valid_item.validate().is_ok());
}
