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
