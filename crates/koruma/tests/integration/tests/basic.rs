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
    assert_eq!(*age_err.actual(), 150);
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
    assert_eq!(name_err.input().as_str(), "");
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
    assert_eq!(*err.age().number_range_validation().unwrap().actual(), -5);
    assert_eq!(
        err.name()
            .string_length_validation()
            .unwrap()
            .input()
            .as_str(),
        ""
    );

    // Both errors are collected, not just the first one
    assert!(!err.is_empty());
}
