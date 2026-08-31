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
