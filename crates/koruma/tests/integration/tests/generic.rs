#[test]
fn test_generic_validator_i32() {
    let validator = GenericRangeValidation::<i32>::min(0)
        .max(100)
        .with_value(50)
        .build();

    assert!(validator.validate(&50));
    assert!(!validator.validate(&150));
    assert_eq!(*validator.actual(), 50);
}

#[test]
fn test_generic_validator_f64() {
    let validator = GenericRangeValidation::<f64>::min(0.0)
        .max(1.0)
        .with_value(0.5)
        .build();

    assert!(validator.validate(&0.5));
    assert!(!validator.validate(&1.5));
    assert_eq!(*validator.actual(), 0.5);
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
    assert_eq!(*score_err.actual(), 150.0);
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
    assert_eq!(*points_err.actual(), 2000);
}

#[test]
fn test_direct_syntax_item_invalid_score() {
    let item = DirectSyntaxItem { score: 150.0 };

    let err = item.validate().unwrap_err();
    assert!(err.score().generic_range_validation().is_some());
    assert_eq!(
        *err.score().generic_range_validation().unwrap().actual(),
        150.0
    );
}

#[test]
fn test_direct_password_confirmation_reuses_field_values() {
    let item = DirectPasswordConfirmation {
        password: "secret".to_string(),
        confirm: "different".to_string(),
    };

    let err = item.validate().unwrap_err();
    let confirm_err = err.confirm().matches_string_validation().unwrap();
    assert_eq!(confirm_err.expected.as_str(), "secret");
    assert_eq!(confirm_err.actual().as_str(), "different");
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
    assert_eq!(
        *err.value().number_range_validation().unwrap().actual(),
        151
    );
    assert_eq!(*err.value().even_number_validation().unwrap().actual(), 151);
}
