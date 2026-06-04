//! Tests for ValidatorAttr and DataFieldKorumaAttr parsing.

use koruma_derive_core::*;

#[test]
fn test_validator_attr_parse_simple() {
    let attr: ValidatorAttr = syn::parse_quote!(RangeValidation);
    assert_eq!(attr.name().to_string(), "RangeValidation");
    assert_eq!(attr.path_name(), "RangeValidation");
    assert_eq!(attr.codegen_snake_name(), "range_validation");
    assert_eq!(attr.codegen_upper_camel_name(), "RangeValidation");
    assert!(!attr.uses_type_inference());
    assert!(attr.setter_calls().is_empty());
}

#[test]
fn test_validator_attr_parse_with_args() {
    let attr: ValidatorAttr = syn::parse_quote!(RangeValidation::min(0).max(100));
    assert_eq!(attr.name().to_string(), "RangeValidation");
    assert!(!attr.uses_type_inference());
    assert_eq!(attr.setter_calls().len(), 2);
    assert_eq!(attr.setter_calls()[0].method().to_string(), "min");
    assert_eq!(attr.setter_calls()[1].method().to_string(), "max");
}

#[test]
fn test_validator_attr_parse_generic() {
    let attr: ValidatorAttr = syn::parse_quote!(GenericRange::<_>::min(0.0).max(1.0));
    assert_eq!(attr.name().to_string(), "GenericRange");
    assert!(attr.uses_type_inference());
    assert_eq!(attr.setter_calls().len(), 2);
}

#[test]
fn test_validator_attr_parse_direct_chain() {
    let attr: ValidatorAttr =
        syn::parse_quote!(RangeValidation::min(0).max(100).exclusive_max(true));
    assert_eq!(attr.name().to_string(), "RangeValidation");
    assert!(!attr.uses_type_inference());
    assert_eq!(attr.setter_calls().len(), 3);
    assert_eq!(attr.setter_calls()[0].method().to_string(), "min");
    assert_eq!(attr.setter_calls()[1].method().to_string(), "max");
    assert_eq!(attr.setter_calls()[2].method().to_string(), "exclusive_max");
}

#[test]
fn test_validator_attr_parse_direct_chain_with_turbofish_inference() {
    let attr: ValidatorAttr =
        syn::parse_quote!(validators::numeric::RangeValidation::<_>::min(0).max(100));
    assert_eq!(attr.name().to_string(), "RangeValidation");
    assert_eq!(attr.path_name(), "validators::numeric::RangeValidation");
    assert!(attr.uses_type_inference());
    assert_eq!(attr.setter_calls().len(), 2);
}

#[test]
fn test_validator_attr_parse_direct_chain_with_explicit_option_type() {
    let attr: ValidatorAttr = syn::parse_quote!(RequiredValidation::<Option<_>>);
    assert_eq!(attr.name().to_string(), "RequiredValidation");
    assert!(!attr.uses_type_inference());
    assert!(attr.explicit_type().is_some());
    assert!(attr.setter_calls().is_empty());
}

#[test]
fn test_koruma_attr_parse_skip() {
    let attr: DataFieldKorumaAttr = syn::parse_quote!(skip);
    assert!(attr.is_skip());
    assert!(!attr.has_field_validators());
    assert!(!attr.has_element_validators());
}

#[test]
fn test_koruma_attr_parse_each() {
    let attr: DataFieldKorumaAttr = syn::parse_quote!(each(
        RangeValidation::min(0).max(100),
        RequiredValidation::<Option<_>>
    ));
    assert!(!attr.is_skip());
    assert!(!attr.has_field_validators());
    assert_eq!(attr.element_validator_count(), 2);
    let element_validators: Vec<_> = attr.element_validators().collect();
    assert!(element_validators[1].explicit_type().is_some());
}

#[test]
fn test_koruma_attr_parse_labeled_field_validator() {
    let attr: DataFieldKorumaAttr =
        syn::parse_quote!(username_prefix = string::PrefixValidation::<_>::prefix("user:"));

    let DataFieldKorumaItem::FieldValidation(spec) = &attr.items()[0] else {
        panic!("expected field validator");
    };
    assert_eq!(
        spec.validator().label().map(ToString::to_string),
        Some("username_prefix".to_string())
    );
    assert_eq!(
        spec.validator().validator().name().to_string(),
        "PrefixValidation"
    );
}

#[test]
fn test_koruma_attr_parse_full_and_unwrapped_field_targets() {
    let attr: DataFieldKorumaAttr = syn::parse_quote!(
        required = full(general::RequiredValidation::<_>),
        len = unwrapped(string::LenValidation::<_>::min(1))
    );

    let DataFieldKorumaItem::FieldValidation(full_spec) = &attr.items()[0] else {
        panic!("expected full field validator");
    };
    assert!(matches!(
        full_spec.validator().target(),
        ValidatorTargetSelector::Full { .. }
    ));
    assert_eq!(
        full_spec.validator().label().map(ToString::to_string),
        Some("required".to_string())
    );

    let DataFieldKorumaItem::FieldValidation(unwrapped_spec) = &attr.items()[1] else {
        panic!("expected unwrapped field validator");
    };
    assert!(matches!(
        unwrapped_spec.validator().target(),
        ValidatorTargetSelector::Unwrapped { .. }
    ));
}

#[test]
fn test_koruma_attr_parse_labeled_element_validator() {
    let attr: DataFieldKorumaAttr = syn::parse_quote!(each(
        tag_prefix = string::PrefixValidation::<_>::prefix("tag:")
    ));

    let DataFieldKorumaItem::ElementValidation(spec) = &attr.items()[0] else {
        panic!("expected element validator");
    };
    assert_eq!(
        spec.validators()[0].label().map(ToString::to_string),
        Some("tag_prefix".to_string())
    );
    assert_eq!(
        spec.validators()[0].validator().name().to_string(),
        "PrefixValidation"
    );
}

#[test]
fn test_koruma_attr_parse_labeled_full_element_validator() {
    let attr: DataFieldKorumaAttr =
        syn::parse_quote!(each(item_required = full(general::RequiredValidation::<_>)));

    let DataFieldKorumaItem::ElementValidation(spec) = &attr.items()[0] else {
        panic!("expected element validator");
    };
    assert!(matches!(
        spec.validators()[0].target(),
        ValidatorTargetSelector::Full { .. }
    ));
    assert_eq!(
        spec.validators()[0].label().map(ToString::to_string),
        Some("item_required".to_string())
    );
}

#[test]
fn test_koruma_attr_parse_multiple_validators() {
    let attr: DataFieldKorumaAttr =
        syn::parse_quote!(ValidatorA::x(1), ValidatorB, ValidatorC::<_>::y(2));
    assert!(!attr.is_skip());
    assert_eq!(attr.field_validator_count(), 3);
    assert!(!attr.has_element_validators());
    let field_validators: Vec<_> = attr.field_validators().collect();
    assert!(!field_validators[0].uses_type_inference());
    assert!(!field_validators[1].uses_type_inference());
    assert!(field_validators[2].uses_type_inference());
}

#[test]
fn test_koruma_attr_parse_combined_field_and_each() {
    // Combined: field validator + each(element validators) with inferred generics
    let attr: DataFieldKorumaAttr = syn::parse_quote!(
        LenValidator::min(1).max(10),
        each(RangeValidation::<_>::min(0).max(100))
    );
    assert!(!attr.is_skip());
    assert_eq!(attr.field_validator_count(), 1);
    let field_validators: Vec<_> = attr.field_validators().collect();
    assert_eq!(field_validators[0].name().to_string(), "LenValidator");
    assert_eq!(attr.element_validator_count(), 1);
    let element_validators: Vec<_> = attr.element_validators().collect();
    assert_eq!(element_validators[0].name().to_string(), "RangeValidation");
    assert!(element_validators[0].uses_type_inference());
}

#[test]
fn test_koruma_attr_parse_each_then_field() {
    // each() can come before field validators too
    let attr: DataFieldKorumaAttr =
        syn::parse_quote!(each(RangeValidation::min(0).max(100)), LenValidator::min(1));
    assert!(!attr.is_skip());
    assert_eq!(attr.field_validator_count(), 1);
    assert_eq!(attr.element_validator_count(), 1);
}

#[test]
fn test_validator_attr_parse_nested_generic() {
    let attr: ValidatorAttr = syn::parse_quote!(RequiredValidation::<Option<_>>);
    assert_eq!(attr.name().to_string(), "RequiredValidation");
    assert!(!attr.uses_type_inference());
    assert!(attr.explicit_type().is_some());

    let explicit_ty = attr.explicit_type().unwrap();
    let ty_str = quote::quote!(#explicit_ty).to_string();
    assert!(
        ty_str.contains("Option"),
        "expected Option<_>, got: {}",
        ty_str
    );
}

#[test]
fn test_validator_attr_parse_nested_generic_concrete() {
    let attr: ValidatorAttr = syn::parse_quote!(SomeValidator::<Vec<String>>);
    assert_eq!(attr.name().to_string(), "SomeValidator");
    assert!(!attr.uses_type_inference());
    assert!(attr.explicit_type().is_some());

    let explicit_ty = attr.explicit_type().unwrap();
    let ty_str = quote::quote!(#explicit_ty).to_string();
    assert!(
        ty_str.contains("Vec") && ty_str.contains("String"),
        "expected Vec<String>, got: {}",
        ty_str
    );
}

#[test]
fn test_validator_attr_parse_deeply_nested_generic() {
    let attr: ValidatorAttr = syn::parse_quote!(DeepValidator::<Option<Vec<_>>>);
    assert_eq!(attr.name().to_string(), "DeepValidator");
    assert!(!attr.uses_type_inference());
    assert!(attr.explicit_type().is_some());
}

#[test]
fn test_validator_attr_codegen_names_preserve_path_segments() {
    let attr: ValidatorAttr = syn::parse_quote!(foo_bar::RangeValidation::<_>);
    assert_eq!(attr.name().to_string(), "RangeValidation");
    assert_eq!(attr.path_name(), "foo_bar::RangeValidation");
    assert_eq!(attr.codegen_snake_name(), "foo_bar_range_validation");
    assert_eq!(attr.codegen_upper_camel_name(), "FooBarRangeValidation");
}

#[test]
fn test_validator_attr_parse_option_infer_type() {
    // The parser preserves explicit `Option<_>` type arguments; derive lowering
    // decides whether that type argument is valid for a field.
    let attr: ValidatorAttr = syn::parse_quote!(RequiredValidation::<Option<_>>);
    assert_eq!(attr.name().to_string(), "RequiredValidation");
    assert!(!attr.uses_type_inference());
    assert!(attr.explicit_type().is_some());
    let explicit_ty = attr.explicit_type().unwrap();
    let ty_str = quote::quote!(#explicit_ty).to_string();
    assert!(
        ty_str.contains("Option"),
        "expected Option<_>, got: {}",
        ty_str
    );
}

#[test]
fn test_validator_attr_parse_invalid_call_error() {
    let result: Result<ValidatorAttr, _> = syn::parse_str("Validator<_>");
    let err = result.expect_err("expected invalid validator call syntax to be rejected");
    assert!(
        err.to_string()
            .contains("requires a direct validator chain")
    );
}

#[test]
fn test_struct_options_parse_try_new() {
    let opts: StructOptions = syn::parse_quote!(try_new);
    let StructMode::Regular = opts.mode() else {
        panic!("expected regular mode");
    };
    assert!(opts.constructors().try_new());
    assert!(!opts.constructors().try_from());
}

#[test]
fn test_struct_options_parse_unknown_error() {
    let result: Result<StructOptions, _> = syn::parse_str("unknown_option");
    assert!(result.is_err(), "expected error for unknown option");
    let err = result.err().unwrap().to_string();
    assert!(
        err.contains("unknown struct-level koruma option"),
        "expected helpful error message, got: {}",
        err
    );
}
