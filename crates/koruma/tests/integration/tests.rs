//! Test cases for koruma validation.

use koruma::{Validate, ValidationError};

use super::fixtures::{GenericItem, Item, MultiValidatorItem, Order, UserProfile};
use super::validators::GenericRangeValidation;

#[test]
fn test_valid_item() {
    let item = Item {
        age: 25,
        name: "Alice".to_string(),
        internal_id: 123,
    };

    assert!(item.validate().is_ok());
}

#[test]
fn test_invalid_age_with_value() {
    let item = Item {
        age: 150, // Out of range
        name: "Bob".to_string(),
        internal_id: 456,
    };

    let err = item.validate().unwrap_err();
    assert!(err.age().number_range_validation().is_some());
    assert!(err.name().string_length_validation().is_none());
    assert!(err.has_errors());

    // The error contains the actual value that failed
    let age_err = err.age().number_range_validation().unwrap();
    assert_eq!(age_err.actual, 150);
}

#[test]
fn test_invalid_name_with_value() {
    let item = Item {
        age: 30,
        name: "".to_string(), // Too short
        internal_id: 789,
    };

    let err = item.validate().unwrap_err();
    assert!(err.age().number_range_validation().is_none());
    assert!(err.name().string_length_validation().is_some());

    // The error contains the actual value that failed
    let name_err = err.name().string_length_validation().unwrap();
    assert_eq!(name_err.input, "".to_string());
}

#[test]
fn test_multiple_field_errors() {
    let item = Item {
        age: -5,              // Out of range
        name: "".to_string(), // Too short
        internal_id: 0,
    };

    let err = item.validate().unwrap_err();
    assert!(err.age().number_range_validation().is_some());
    assert!(err.name().string_length_validation().is_some());

    // Both errors contain their respective values
    assert_eq!(err.age().number_range_validation().unwrap().actual, -5);
    assert_eq!(
        err.name().string_length_validation().unwrap().input,
        "".to_string()
    );

    // Both errors are collected, not just the first one
    assert!(!err.is_empty());
}

#[test]
fn test_generic_validator_i32() {
    let validator = GenericRangeValidation::<i32>::builder()
        .min(0)
        .max(100)
        .with_value(50)
        .build();

    assert!(validator.validate(&50).is_ok());
    assert!(validator.validate(&150).is_err());
    assert_eq!(validator.actual, 50);
}

#[test]
fn test_generic_validator_f64() {
    let validator = GenericRangeValidation::<f64>::builder()
        .min(0.0)
        .max(1.0)
        .with_value(0.5)
        .build();

    assert!(validator.validate(&0.5).is_ok());
    assert!(validator.validate(&1.5).is_err());
    assert_eq!(validator.actual, 0.5);
}

#[test]
fn test_generic_item_valid() {
    let item = GenericItem {
        score: 50.0,
        points: 500,
    };

    assert!(item.validate().is_ok());
}

#[test]
fn test_generic_item_invalid_score() {
    let item = GenericItem {
        score: 150.0, // Out of range (max 100.0)
        points: 500,
    };

    let err = item.validate().unwrap_err();
    assert!(err.score().generic_range_validation().is_some());
    assert!(err.points().generic_range_validation().is_none());

    // The error contains the actual value
    let score_err = err.score().generic_range_validation().unwrap();
    assert_eq!(score_err.actual, 150.0);
}

#[test]
fn test_generic_item_invalid_points() {
    let item = GenericItem {
        score: 50.0,
        points: 2000, // Out of range (max 1000)
    };

    let err = item.validate().unwrap_err();
    assert!(err.score().generic_range_validation().is_none());
    assert!(err.points().generic_range_validation().is_some());

    // The error contains the actual value
    let points_err = err.points().generic_range_validation().unwrap();
    assert_eq!(points_err.actual, 2000);
}

// Tests for multiple validators per field
#[test]
fn test_multi_validator_valid() {
    let item = MultiValidatorItem { value: 50 }; // In range AND even
    assert!(item.validate().is_ok());
}

#[test]
fn test_multi_validator_out_of_range() {
    let item = MultiValidatorItem { value: 150 }; // Out of range, but even
    let err = item.validate().unwrap_err();

    assert!(err.value().number_range_validation().is_some());
    assert!(err.value().even_number_validation().is_none()); // 150 is even
}

#[test]
fn test_multi_validator_odd() {
    let item = MultiValidatorItem { value: 51 }; // In range, but odd
    let err = item.validate().unwrap_err();

    assert!(err.value().number_range_validation().is_none()); // 51 is in range
    assert!(err.value().even_number_validation().is_some());
}

#[test]
fn test_multi_validator_both_fail() {
    let item = MultiValidatorItem { value: 151 }; // Out of range AND odd
    let err = item.validate().unwrap_err();

    // Both validators should fail
    assert!(err.value().number_range_validation().is_some());
    assert!(err.value().even_number_validation().is_some());

    // Check the actual values
    assert_eq!(err.value().number_range_validation().unwrap().actual, 151);
    assert_eq!(err.value().even_number_validation().unwrap().actual, 151);
}

#[test]
fn test_all_validators() {
    // Single validator field
    let item = Item {
        age: 150,
        name: "Valid".to_string(),
        internal_id: 0,
    };
    let err = item.validate().unwrap_err();
    let age_errors = err.age().all();
    assert_eq!(age_errors.len(), 1);

    // Multiple validators - both fail
    let item = MultiValidatorItem { value: 151 };
    let err = item.validate().unwrap_err();
    let value_errors = err.value().all();
    assert_eq!(value_errors.len(), 2);

    // Multiple validators - one fails
    let item = MultiValidatorItem { value: 150 }; // even but out of range
    let err = item.validate().unwrap_err();
    let value_errors = err.value().all();
    assert_eq!(value_errors.len(), 1);
}

// Tests for collection validation with each()
#[test]
fn test_each_valid() {
    let order = Order {
        scores: vec![50.0, 75.0, 100.0],
    };
    assert!(order.validate().is_ok());
}

#[test]
fn test_each_single_invalid() {
    let order = Order {
        scores: vec![50.0, 150.0, 75.0], // 150 is out of range
    };
    let err = order.validate().unwrap_err();
    let score_errors = err.scores();

    assert_eq!(score_errors.len(), 1);
    assert_eq!(score_errors[0].0, 1); // Index 1 failed

    let element_err = &score_errors[0].1;
    assert!(element_err.generic_range_validation().is_some());
    assert_eq!(
        element_err.generic_range_validation().unwrap().actual,
        150.0
    );
}

#[test]
fn test_each_multiple_invalid() {
    let order = Order {
        scores: vec![150.0, 50.0, -10.0], // Index 0 and 2 are invalid
    };
    let err = order.validate().unwrap_err();
    let score_errors = err.scores();

    assert_eq!(score_errors.len(), 2);
    assert_eq!(score_errors[0].0, 0); // Index 0 failed
    assert_eq!(score_errors[1].0, 2); // Index 2 failed
}

#[test]
fn test_each_empty_collection() {
    let order = Order { scores: vec![] };
    assert!(order.validate().is_ok());
}

// Tests for optional field validation
#[test]
fn test_optional_field_none_skips_validation() {
    let profile = UserProfile {
        username: "alice".to_string(),
        bio: None, // Should skip validation
        age: None, // Should skip validation
    };

    // All None fields are skipped, username is valid
    assert!(profile.validate().is_ok());
}

#[test]
fn test_optional_field_some_valid() {
    let profile = UserProfile {
        username: "bob".to_string(),
        bio: Some("I love coding!".to_string()),
        age: Some(25),
    };

    assert!(profile.validate().is_ok());
}

#[test]
fn test_optional_field_some_invalid() {
    let profile = UserProfile {
        username: "charlie".to_string(),
        bio: Some("".to_string()), // Too short (min = 1)
        age: Some(200),            // Out of range (max = 150)
    };

    let err = profile.validate().unwrap_err();

    // Bio should fail
    assert!(err.bio().string_length_validation().is_some());
    let bio_err = err.bio().string_length_validation().unwrap();
    assert_eq!(bio_err.input, "".to_string()); // Direct value, no Option!

    // Age should fail
    assert!(err.age().number_range_validation().is_some());
    let age_err = err.age().number_range_validation().unwrap();
    assert_eq!(age_err.actual, 200); // Direct value, no Option!
}

#[test]
fn test_optional_field_mixed() {
    let profile = UserProfile {
        username: "diana".to_string(),
        bio: None,      // Skip validation
        age: Some(200), // Invalid
    };

    let err = profile.validate().unwrap_err();

    // Bio is None, so no error
    assert!(err.bio().string_length_validation().is_none());

    // Age has a value, and it's invalid
    assert!(err.age().number_range_validation().is_some());
}

#[test]
fn test_required_field_with_optional_fields() {
    let profile = UserProfile {
        username: "".to_string(), // Invalid - too short
        bio: None,
        age: None,
    };

    let err = profile.validate().unwrap_err();

    // Username should fail (required field)
    assert!(err.username().string_length_validation().is_some());

    // Optional fields with None should not have errors
    assert!(err.bio().string_length_validation().is_none());
    assert!(err.age().number_range_validation().is_none());
}
