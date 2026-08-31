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
