//! Test cases for koruma validation.

use koruma::{Validate, ValidationError};

use super::fixtures::{
    Address, AddressWrapper, BorrowedOrder, BorrowedTags, BorrowedUsername,
    BorrowedUsernameExplicitInfer, Company, ContainsNewtype, Customer, CustomerWithOptionalAddress,
    DirectPasswordConfirmation, DirectSyntaxItem, Employee, ExplicitRequiredProfile, GenericItem,
    Item, MultiAttrItem, MultiValidatorItem, NonCloneSecret, NonCloneValidatorItem,
    OptionalBorrowedOrder, OptionalElementMixedValidators, OptionalElementPresenceOrder,
    OptionalOrder, Order, OrderWithLenCheck, PasswordConfirmation, PositiveNumber,
    PresenceOnlyNonClone, QualifiedPathProfile, StaticSecretConfirmation, UserProfile,
};
use super::validators::{GenericRangeValidation, PrefixBytesValidation};

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
    assert_eq!(age_errors.count(), 1);

    // Multiple validators - both fail
    let item = MultiValidatorItem { value: 151 };
    let err = item.validate().unwrap_err();
    let value_errors = err.value().all();
    assert_eq!(value_errors.count(), 2);

    // Multiple validators - one fails
    let item = MultiValidatorItem { value: 150 }; // even but out of range
    let err = item.validate().unwrap_err();
    let value_errors = err.value().all();
    assert_eq!(value_errors.count(), 1);
}

// Tests for multiple separate #[koruma(...)] attributes per field
#[test]
fn test_multi_attr_valid() {
    let item = MultiAttrItem { value: 50 }; // In range AND even
    assert!(item.validate().is_ok());
}

#[test]
fn test_multi_attr_out_of_range() {
    let item = MultiAttrItem { value: 150 }; // Out of range, but even
    let err = item.validate().unwrap_err();

    assert!(err.value().number_range_validation().is_some());
    assert!(err.value().even_number_validation().is_none()); // 150 is even
}

#[test]
fn test_multi_attr_odd() {
    let item = MultiAttrItem { value: 51 }; // In range, but odd
    let err = item.validate().unwrap_err();

    assert!(err.value().number_range_validation().is_none()); // 51 is in range
    assert!(err.value().even_number_validation().is_some());
}

#[test]
fn test_required_validation_supports_non_clone_option_payload() {
    let valid = PresenceOnlyNonClone {
        token: Some(NonCloneSecret {
            raw: "secret".to_string(),
        }),
    };
    assert!(valid.validate().is_ok());
    assert_eq!(
        valid.token.as_ref().map(|secret| secret.raw.as_str()),
        Some("secret")
    );

    let missing = PresenceOnlyNonClone { token: None };
    let err = missing.validate().unwrap_err();
    let validation = err.token().required_validation().unwrap();
    assert!(validation.actual().is_none());
}

#[test]
fn test_multi_attr_both_fail() {
    let item = MultiAttrItem { value: 151 }; // Out of range AND odd
    let err = item.validate().unwrap_err();

    // Both validators should fail (from separate #[koruma] attributes)
    assert!(err.value().number_range_validation().is_some());
    assert!(err.value().even_number_validation().is_some());

    // Check the actual values
    assert_eq!(
        *err.value().number_range_validation().unwrap().actual(),
        151
    );
    assert_eq!(*err.value().even_number_validation().unwrap().actual(), 151);
}

#[test]
fn test_multi_attr_all_validators() {
    // Multiple separate attributes - both fail
    let item = MultiAttrItem { value: 151 };
    let err = item.validate().unwrap_err();
    let value_errors = err.value().all();
    assert_eq!(value_errors.count(), 2);

    // Multiple separate attributes - one fails
    let item = MultiAttrItem { value: 150 }; // even but out of range
    let err = item.validate().unwrap_err();
    let value_errors = err.value().all();
    assert_eq!(value_errors.count(), 1);
}

#[test]
fn test_all_borrows_non_clone_failed_validators() {
    let item = NonCloneValidatorItem { value: 150 };
    let err = item.validate().unwrap_err();
    let failures: Vec<_> = err.value().all().collect();

    assert_eq!(failures.len(), 1);
    assert_eq!(
        failures[0].to_string(),
        "value must be between 0 and 100, got 150"
    );
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
fn test_borrowed_direct_field_valid() {
    let user = BorrowedUsername {
        username: "user:alice",
    };
    assert!(user.validate().is_ok());
}

#[test]
fn test_validator_builder_preserves_lifetime_and_const_generics_for_with_value() {
    let validator = PrefixBytesValidation::prefix(b"ab")
        .with_value(*b"abcd")
        .build();

    assert!(validator.validate(b"abcd"));
    assert!(!validator.validate(b"zzzz"));
    assert_eq!(validator.actual(), b"abcd");
}

#[test]
fn test_validator_builder_preserves_lifetime_and_const_generics_for_with_value_ref() {
    let builder = PrefixBytesValidation::prefix(b"ab");
    let validator = koruma::BuilderWithValueRef::with_value_ref(builder, b"abcd").build();

    assert!(validator.validate(b"abcd"));
    assert!(!validator.validate(b"zzzz"));
    assert_eq!(validator.actual(), b"abcd");
}

#[test]
fn test_borrowed_direct_field_invalid() {
    let user = BorrowedUsername { username: "guest" };

    let err = user.validate().unwrap_err();
    let validator = err
        .username()
        .starts_with_validation()
        .expect("expected username prefix validation error");

    assert_eq!(*validator.actual(), "guest");
    let failures: Vec<_> = err.username().all().collect();
    assert_eq!(failures[0].to_string(), "Must start with 'user:'");
}

#[test]
fn test_borrowed_direct_field_explicit_reference_infer_invalid() {
    let user = BorrowedUsernameExplicitInfer { username: "guest" };

    let err = user.validate().unwrap_err();
    let validator = err
        .username()
        .starts_with_validation()
        .expect("expected username prefix validation error");

    assert_eq!(*validator.actual(), "guest");
    let failures: Vec<_> = err.username().all().collect();
    assert_eq!(failures[0].to_string(), "Must start with 'user:'");
}

#[test]
fn test_each_borrowed_str_items_invalid() {
    let tags = ["tag:one", "bad"];
    let value = BorrowedTags { tags: &tags };

    let err = value.validate().unwrap_err();
    let tag_errors = err.tags().element_errors();

    assert_eq!(tag_errors.len(), 1);
    assert_eq!(tag_errors[0].0, 1);
    assert_eq!(
        *tag_errors[0]
            .1
            .starts_with_validation()
            .expect("expected failing borrowed element validator")
            .actual(),
        "bad"
    );
    let failures: Vec<_> = tag_errors[0].1.all().collect();
    assert_eq!(failures[0].to_string(), "Must start with 'tag:'");
}

#[test]
fn test_cross_field_arg_uses_explicit_field_access() {
    let confirmation = PasswordConfirmation {
        password: "secret".to_string(),
        confirm: "different".to_string(),
    };

    let err = confirmation.validate().unwrap_err();
    let validator = err
        .confirm()
        .matches_string_validation()
        .expect("expected confirm mismatch");
    assert_eq!(validator.expected, "secret");
    assert_eq!(validator.actual(), "different");
}

#[test]
fn test_non_field_arg_identifier_is_not_rewritten() {
    let confirmation = StaticSecretConfirmation {
        confirm: "wrong".to_string(),
    };

    let err = confirmation.validate().unwrap_err();
    let validator = err
        .confirm()
        .matches_static_str_validation()
        .expect("expected shared-secret mismatch");
    assert_eq!(validator.expected, "shared-secret");
    assert_eq!(validator.actual(), "wrong");
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
fn test_explicit_full_type_optional_field_none_fails() {
    let profile = ExplicitRequiredProfile { bio: None };

    let err = profile.validate().unwrap_err();
    assert!(err.bio().required_validation().is_some());
}

#[test]
fn test_explicit_full_type_optional_field_some_passes() {
    let profile = ExplicitRequiredProfile {
        bio: Some("I love concrete Option types".to_string()),
    };

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
    assert_eq!(bio_err.input().as_str(), ""); // Direct value, no Option!

    // Age should fail
    assert!(err.age().number_range_validation().is_some());
    let age_err = err.age().number_range_validation().unwrap();
    assert_eq!(*age_err.actual(), 200); // Direct value, no Option!
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

// Tests for COMBINED collection-level + per-element validation
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

#[test]
fn test_nested_valid() {
    let customer = Customer {
        name: "Alice".to_string(),
        address: Address {
            street: "123 Main St".to_string(),
            city: "Springfield".to_string(),
            zip_code: "12345".to_string(),
        },
    };
    assert!(customer.validate().is_ok());
}

#[test]
fn test_nested_parent_invalid() {
    let customer = Customer {
        name: "".to_string(), // Invalid: empty name
        address: Address {
            street: "123 Main St".to_string(),
            city: "Springfield".to_string(),
            zip_code: "12345".to_string(),
        },
    };
    let err = customer.validate().unwrap_err();

    // Parent field has error
    assert!(err.name().string_length_validation().is_some());
    // Nested struct is valid, so no nested error
    assert!(err.address().is_none());
}

#[test]
fn test_nested_child_invalid() {
    let customer = Customer {
        name: "Alice".to_string(),
        address: Address {
            street: "".to_string(), // Invalid: empty street
            city: "Springfield".to_string(),
            zip_code: "12345".to_string(),
        },
    };
    let err = customer.validate().unwrap_err();

    // Parent field is valid
    assert!(err.name().string_length_validation().is_none());
    // Nested struct has error - access nested error struct
    let nested_err = err.address().expect("should have nested error");
    assert!(nested_err.street().string_length_validation().is_some());
    assert!(nested_err.city().string_length_validation().is_none());
}

#[test]
fn test_nested_both_invalid() {
    let customer = Customer {
        name: "".to_string(), // Invalid
        address: Address {
            street: "123 Main St".to_string(),
            city: "".to_string(),      // Invalid: empty city
            zip_code: "X".to_string(), // Invalid: too short (min 2)
        },
    };
    let err = customer.validate().unwrap_err();

    // Parent field has error
    assert!(err.name().string_length_validation().is_some());
    // Nested struct has multiple errors
    let nested_err = err.address().expect("should have nested error");
    assert!(nested_err.street().string_length_validation().is_none());
    assert!(nested_err.city().string_length_validation().is_some());
    assert!(nested_err.zip_code().string_length_validation().is_some());
}

#[test]
fn test_optional_nested_none_skips_validation() {
    let customer = CustomerWithOptionalAddress {
        name: "Bob".to_string(),
        shipping_address: None, // No address - validation skipped
    };
    assert!(customer.validate().is_ok());
}

#[test]
fn test_optional_nested_some_valid() {
    let customer = CustomerWithOptionalAddress {
        name: "Bob".to_string(),
        shipping_address: Some(Address {
            street: "456 Oak Ave".to_string(),
            city: "Shelbyville".to_string(),
            zip_code: "67890".to_string(),
        }),
    };
    assert!(customer.validate().is_ok());
}

#[test]
fn test_optional_nested_some_invalid() {
    let customer = CustomerWithOptionalAddress {
        name: "Bob".to_string(),
        shipping_address: Some(Address {
            street: "".to_string(), // Invalid
            city: "Shelbyville".to_string(),
            zip_code: "67890".to_string(),
        }),
    };
    let err = customer.validate().unwrap_err();

    assert!(err.name().string_length_validation().is_none());
    let nested_err = err.shipping_address().expect("should have nested error");
    assert!(nested_err.street().string_length_validation().is_some());
}

#[test]
fn test_deeply_nested_valid() {
    let employee = Employee {
        employee_name: "Charlie".to_string(),
        employer: Company {
            company_name: "Acme Corp".to_string(),
            headquarters: Address {
                street: "789 Industrial Blvd".to_string(),
                city: "Metropolis".to_string(),
                zip_code: "11111".to_string(),
            },
        },
    };
    assert!(employee.validate().is_ok());
}

#[test]
fn test_deeply_nested_innermost_invalid() {
    let employee = Employee {
        employee_name: "Charlie".to_string(),
        employer: Company {
            company_name: "Acme Corp".to_string(),
            headquarters: Address {
                street: "789 Industrial Blvd".to_string(),
                city: "".to_string(), // Invalid at deepest level
                zip_code: "11111".to_string(),
            },
        },
    };
    let err = employee.validate().unwrap_err();

    // Navigate through the nested errors
    let company_err = err.employer().expect("should have employer error");
    let address_err = company_err
        .headquarters()
        .expect("should have headquarters error");
    assert!(address_err.city().string_length_validation().is_some());
}

#[test]
fn test_deeply_nested_multiple_levels_invalid() {
    let employee = Employee {
        employee_name: "".to_string(), // Invalid at top level
        employer: Company {
            company_name: "".to_string(), // Invalid at middle level
            headquarters: Address {
                street: "".to_string(), // Invalid at deepest level
                city: "Metropolis".to_string(),
                zip_code: "11111".to_string(),
            },
        },
    };
    let err = employee.validate().unwrap_err();

    // Top level error
    assert!(err.employee_name().string_length_validation().is_some());

    // Middle level error
    let company_err = err.employer().expect("should have employer error");
    assert!(
        company_err
            .company_name()
            .string_length_validation()
            .is_some()
    );

    // Deepest level error
    let address_err = company_err
        .headquarters()
        .expect("should have headquarters error");
    assert!(address_err.street().string_length_validation().is_some());
}

// ============================================================================
// Newtype struct validation tests
// ============================================================================

#[test]
fn test_newtype_with_validators_valid() {
    let num = PositiveNumber { value: 50 };
    assert!(num.validate().is_ok());
}

#[test]
fn test_newtype_with_validators_invalid() {
    let num = PositiveNumber { value: -10 };
    let err = num.validate().unwrap_err();

    // Can access via the field getter
    assert!(err.value().number_range_validation().is_some());

    // Can also access via Deref - the error struct derefs to the field error struct
    assert!(err.number_range_validation().is_some());
    assert_eq!(err.all().count(), 1);
}

#[test]
fn test_newtype_nested_valid() {
    let wrapper = AddressWrapper {
        inner: Address {
            street: "123 Main St".to_string(),
            city: "Springfield".to_string(),
            zip_code: "12345".to_string(),
        },
    };
    assert!(wrapper.validate().is_ok());
}

#[test]
fn test_newtype_nested_invalid() {
    let wrapper = AddressWrapper {
        inner: Address {
            street: "".to_string(), // Invalid
            city: "Springfield".to_string(),
            zip_code: "12345".to_string(),
        },
    };
    let err = wrapper.validate().unwrap_err();

    // Can access via Deref - the error struct derefs to the inner error struct
    // So we can call methods on AddressKorumaValidationError directly
    assert!(err.street().string_length_validation().is_some());
}

#[test]
fn test_field_level_newtype_valid() {
    let container = ContainsNewtype {
        name: "Test".to_string(),
        number: PositiveNumber { value: 50 },
    };
    assert!(container.validate().is_ok());
}

#[test]
fn test_field_level_newtype_invalid() {
    let container = ContainsNewtype {
        name: "Test".to_string(),
        number: PositiveNumber { value: -10 }, // Invalid
    };
    let err = container.validate().unwrap_err();

    // Name is valid
    assert!(err.name().string_length_validation().is_none());

    // Number is invalid - can access via Deref!
    // err.number() returns &ContainsNewtypeNumberKorumaValidationError which derefs to
    // PositiveNumberKorumaValidationError which derefs to PositiveNumberValueKorumaValidationError
    assert_eq!(err.number().all().count(), 1);
    assert!(err.number().number_range_validation().is_some());
}
