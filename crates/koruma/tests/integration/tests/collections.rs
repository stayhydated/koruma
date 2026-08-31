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
    let score_errors = err.scores().element_errors();

    assert_eq!(score_errors.len(), 1);
    assert_eq!(score_errors[0].0, 1); // Index 1 failed

    let element_err = &score_errors[0].1;
    assert!(element_err.generic_range_validation().is_some());
    assert_eq!(
        *element_err.generic_range_validation().unwrap().actual(),
        150.0
    );
}

#[test]
fn test_each_multiple_invalid() {
    let order = Order {
        scores: vec![150.0, 50.0, -10.0], // Index 0 and 2 are invalid
    };
    let err = order.validate().unwrap_err();
    let score_errors = err.scores().element_errors();

    assert_eq!(score_errors.len(), 2);
    assert_eq!(score_errors[0].0, 0); // Index 0 failed
    assert_eq!(score_errors[1].0, 2); // Index 2 failed
}

#[test]
fn test_each_empty_collection() {
    let order = Order { scores: vec![] };
    assert!(order.validate().is_ok());
}

#[test]
fn test_each_optional_collection_none_skips_validation() {
    let order = OptionalOrder { scores: None };
    assert!(order.validate().is_ok());
}

#[test]
fn test_each_optional_collection_some_invalid_element() {
    let order = OptionalOrder {
        scores: Some(vec![50.0, 150.0, 75.0]),
    };

    let err = order.validate().unwrap_err();
    let score_errors = err.scores().element_errors();

    assert_eq!(score_errors.len(), 1);
    assert_eq!(score_errors[0].0, 1);
    assert_eq!(
        *score_errors[0]
            .1
            .generic_range_validation()
            .expect("expected failing element validator")
            .actual(),
        150.0
    );
}

#[test]
fn test_each_optional_elements_full_type_validator_reports_none() {
    let order = OptionalElementPresenceOrder {
        values: vec![Some(1), None, Some(3)],
    };

    let err = order.validate().unwrap_err();
    let value_errors = err.values().element_errors();

    assert_eq!(value_errors.len(), 1);
    assert_eq!(value_errors[0].0, 1);
    assert!(value_errors[0].1.required_validation().is_some());
}

#[test]
fn test_each_optional_elements_split_full_type_and_unwrapped_paths() {
    let order = OptionalElementMixedValidators {
        values: vec![None, Some(20), Some(5)],
    };

    let err = order.validate().unwrap_err();
    let value_errors = err.values().element_errors();

    assert_eq!(value_errors.len(), 2);
    assert_eq!(value_errors[0].0, 0);
    assert!(value_errors[0].1.required_validation().is_some());
    assert!(value_errors[0].1.generic_range_validation().is_none());

    assert_eq!(value_errors[1].0, 1);
    assert!(value_errors[1].1.required_validation().is_none());
    assert!(value_errors[1].1.generic_range_validation().is_some());
    assert_eq!(
        *value_errors[1]
            .1
            .generic_range_validation()
            .expect("expected failing unwrapped element validator")
            .actual(),
        20
    );
}

#[test]
fn test_each_required_elements_full_type_validator_uses_element_reference() {
    let order = RequiredElementFullTypeOrder {
        values: vec![1, 20, 5],
    };

    let err = order.validate().unwrap_err();
    let value_errors = err.values().element_errors();

    assert_eq!(value_errors.len(), 1);
    assert_eq!(value_errors[0].0, 1);
    assert_eq!(
        *value_errors[0]
            .1
            .generic_range_validation()
            .expect("expected failing full-type element validator")
            .actual(),
        20
    );
}

#[test]
fn test_qualified_option_and_vec_paths_keep_validation_behavior() {
    let profile = QualifiedPathProfile {
        bio: Some(String::new()),
        scores: Some(vec![50.0, 150.0, 75.0]),
    };

    let err = profile.validate().unwrap_err();
    let bio_err = err
        .bio()
        .string_length_validation()
        .expect("expected qualified Option<String> field to unwrap and validate");
    assert_eq!(bio_err.input().as_str(), "");

    let score_errors = err.scores().element_errors();
    assert_eq!(score_errors.len(), 1);
    assert_eq!(score_errors[0].0, 1);
    assert_eq!(
        *score_errors[0]
            .1
            .generic_range_validation()
            .expect("expected qualified Option<Vec<_>> field to validate elements")
            .actual(),
        150.0
    );
}

#[test]
fn test_qualified_option_and_vec_paths_skip_when_none() {
    let profile = QualifiedPathProfile {
        bio: None,
        scores: None,
    };

    assert!(profile.validate().is_ok());
}

#[test]
fn test_each_borrowed_slice_valid() {
    let values = [50.0, 75.0, 100.0];
    let order = BorrowedOrder { scores: &values };
    assert!(order.validate().is_ok());
}

#[test]
fn test_each_borrowed_slice_invalid() {
    let values = [50.0, 150.0, 75.0];
    let order = BorrowedOrder { scores: &values };

    let err = order.validate().unwrap_err();
    let score_errors = err.scores().element_errors();

    assert_eq!(score_errors.len(), 1);
    assert_eq!(score_errors[0].0, 1);
    assert_eq!(
        *score_errors[0]
            .1
            .generic_range_validation()
            .expect("expected failing element validator")
            .actual(),
        150.0
    );
}

#[test]
fn test_each_optional_borrowed_slice_none_skips_validation() {
    let order = OptionalBorrowedOrder { scores: None };
    assert!(order.validate().is_ok());
}

#[test]
fn test_each_array_valid() {
    let order = ArrayOrder {
        scores: [50, 75, 100],
    };
    assert!(order.validate().is_ok());
}

#[test]
fn test_each_array_invalid() {
    let order = ArrayOrder {
        scores: [50, 150, 75],
    };

    let err = order.validate().unwrap_err();
    let score_errors = err.scores().element_errors();

    assert_eq!(score_errors.len(), 1);
    assert_eq!(score_errors[0].0, 1);
    assert_eq!(
        *score_errors[0]
            .1
            .generic_range_validation()
            .expect("expected failing element validator")
            .actual(),
        150
    );
}


#[test]
fn test_combined_valid() {
    let order = OrderWithLenCheck {
        scores: vec![50.0, 75.0, 100.0], // len=3 is in [1,5], all values in [0,100]
    };
    assert!(order.validate().is_ok());
}

#[test]
fn test_combined_len_fails() {
    let order = OrderWithLenCheck {
        scores: vec![], // len=0 is NOT in [1,5]
    };
    let err = order.validate().unwrap_err();

    // Length validation should fail
    assert!(err.scores().vec_len_validation().is_some());
    let len_err = err.scores().vec_len_validation().unwrap();
    assert_eq!(len_err.actual_len(), 0);
    assert_eq!(len_err.min, 1);
    assert_eq!(len_err.max, 5);

    // No element errors (no elements to validate)
    assert!(err.scores().element_errors().is_empty());
}

#[test]
fn test_combined_element_fails() {
    let order = OrderWithLenCheck {
        scores: vec![50.0, 150.0, 75.0], // len=3 is ok, but 150 is out of range
    };
    let err = order.validate().unwrap_err();

    // Length validation should pass (len=3 is in [1,5])
    assert!(err.scores().vec_len_validation().is_none());

    // Element validation should fail for index 1
    let element_errors = err.scores().element_errors();
    assert_eq!(element_errors.len(), 1);
    assert_eq!(element_errors[0].0, 1); // Index 1 failed
    assert!(element_errors[0].1.generic_range_validation().is_some());
}

#[test]
fn test_combined_both_fail() {
    let order = OrderWithLenCheck {
        scores: vec![50.0, 150.0, -10.0, 75.0, 200.0, 25.0], // len=6 > 5, and 3 values out of range
    };
    let err = order.validate().unwrap_err();

    // Length validation should fail (len=6 > 5)
    assert!(err.scores().vec_len_validation().is_some());

    // Element validation should also fail for indices 1, 2, 4
    let element_errors = err.scores().element_errors();
    assert_eq!(element_errors.len(), 3);
    assert_eq!(element_errors[0].0, 1); // 150.0 out of range
    assert_eq!(element_errors[1].0, 2); // -10.0 out of range
    assert_eq!(element_errors[2].0, 4); // 200.0 out of range
}

// ============================================================================
// Nested struct validation tests
// ============================================================================
