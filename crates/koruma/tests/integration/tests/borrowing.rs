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
fn test_validator_capture_hook_preserves_lifetime_and_const_generics() {
    let builder = PrefixBytesValidation::prefix(b"ab");
    let validator = koruma::__private::BuildValidator::build_validator(
        koruma::__private::CaptureValueRef::capture_value_ref(builder, b"abcd"),
    );

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
