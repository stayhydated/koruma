#[test]
fn test_validator_metadata_reports_static_and_runtime_params() {
    let descriptor = <NumberRangeValidation as ValidatorMetadata<i32>>::validator_descriptor();
    assert!(descriptor.type_name().ends_with("NumberRangeValidation"));
    assert_eq!(descriptor.params().len(), 2);
    assert_eq!(descriptor.params()[0].name(), "min");
    assert_eq!(descriptor.params()[0].type_name(), "i32");
    assert!(descriptor.params()[0].required());
    assert_eq!(descriptor.params()[1].name(), "max");

    let validator = NumberRangeValidation::min(10)
        .max(20)
        .with_value(30)
        .build();
    let params = validator.validator_params();
    assert_eq!(params.len(), 2);
    assert_eq!(params[0].name(), "min");
    assert_eq!(params[0].value(), &ValidatorParamValue::I64(10));
    assert_eq!(params[1].name(), "max");
    assert_eq!(params[1].value(), &ValidatorParamValue::I64(20));
}

#[test]
fn test_generic_validator_metadata_uses_opaque_params_without_extra_bounds() {
    let validator = GenericRangeValidation::<String>::min("a".to_string())
        .max("m".to_string())
        .with_value("z".to_string())
        .build();

    let params = validator.validator_params();
    assert_eq!(params.len(), 2);
    assert_eq!(params[0].name(), "min");
    assert!(matches!(
        params[0].value(),
        ValidatorParamValue::Opaque { type_name } if type_name.ends_with("String")
    ));
}

#[test]
fn test_validation_issues_reports_field_failures() {
    let item = Item {
        age: 150,
        name: "".to_string(),
        internal_id: 123,
    };
    let err = item.validate().unwrap_err();
    let issues = err.issues();

    assert_eq!(issues.len(), 2);
    assert_eq!(
        issues[0].field_name(),
        Some(ValidationFieldName::new("age"))
    );
    assert_eq!(issues[0].field_name_str(), Some("age"));
    assert_eq!(issues[0].scope(), ValidationIssueScope::Field);
    assert!(
        issues[0]
            .validator()
            .is_some_and(|validator| validator.ends_with("NumberRangeValidation"))
    );
    assert_eq!(issues[0].label(), None);
    assert!(issues[0].message().contains("NumberRangeValidation"));
    assert!(issues[0].params().is_empty());

    assert_eq!(
        issues[1].field_name(),
        Some(ValidationFieldName::new("name"))
    );
    assert_eq!(issues[1].scope(), ValidationIssueScope::Field);
    assert!(
        issues[1]
            .validator()
            .is_some_and(|validator| validator.ends_with("StringLengthValidation"))
    );
}
#[test]
fn test_validation_issues_reports_element_failures() {
    let order = Order {
        scores: vec![50.0, 150.0],
    };
    let err = order.validate().unwrap_err();
    let issues = err.issues();

    assert_eq!(issues.len(), 1);
    assert_eq!(
        issues[0].field_name(),
        Some(ValidationFieldName::new("scores"))
    );
    assert_eq!(issues[0].scope(), ValidationIssueScope::Element);
    assert_eq!(issues[0].element_index(), Some(1));
    assert!(
        issues[0]
            .validator()
            .is_some_and(|validator| validator.ends_with("GenericRangeValidation<f64>"))
    );
}
