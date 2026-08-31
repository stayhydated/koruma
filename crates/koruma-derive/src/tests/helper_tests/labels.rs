use super::support::*;

#[test]
fn test_unlabeled_validator_name_collisions_require_labels() {
    let input: syn::DeriveInput = syn::parse_quote! {
        struct Demo {
            #[koruma(foo::Baz, bar::Baz)]
            value: String,
        }
    };

    let err = ValidationPlan::build(&input, "Koruma")
        .expect_err("expected duplicate generated validator names to fail");
    assert!(err.to_string().contains("add explicit validator labels"));
}

#[test]
fn test_labeled_validator_names_drive_getters_and_variants() {
    let input: syn::DeriveInput = syn::parse_quote! {
        struct Demo {
            #[koruma(
                first_baz = foo::Baz,
                second_baz = bar::Baz,
            )]
            value: String,
            other: String,
        }
    };

    let plan = ValidationPlan::build(&input, "Koruma").expect("labels should disambiguate names");
    let validators = plan.fields[0].field_validators();
    assert_eq!(validators[0].field_ident.to_string(), "first_baz");
    assert_eq!(validators[0].variant_ident.to_string(), "FirstBaz");
    assert_eq!(validators[1].field_ident.to_string(), "second_baz");
    assert_eq!(validators[1].variant_ident.to_string(), "SecondBaz");
}

#[test]
fn test_same_validator_type_can_be_used_twice_with_labels() {
    let input: syn::DeriveInput = syn::parse_quote! {
        struct Demo {
            #[koruma(
                min_len = LengthValidation::<_>.min(3),
                max_len = LengthValidation::<_>.max(30),
            )]
            value: String,
        }
    };

    let plan = ValidationPlan::build(&input, "Koruma")
        .expect("labeled repeated validators should be accepted");
    let validators = plan.fields[0].field_validators();
    assert_eq!(validators.len(), 2);
    assert_eq!(validators[0].field_ident.to_string(), "min_len");
    assert_eq!(validators[1].field_ident.to_string(), "max_len");
}

#[test]
fn test_labeled_element_validator_names_drive_getters_and_variants() {
    let input: syn::DeriveInput = syn::parse_quote! {
        struct Demo {
            #[koruma(each(
                tag_prefix = string::PrefixValidation::<_>.prefix("tag:"),
            ))]
            tags: Vec<String>,
        }
    };

    let plan = ValidationPlan::build(&input, "Koruma")
        .expect("element validator labels should plan successfully");
    let validators = plan.fields[0].element_validators();
    assert_eq!(validators[0].field_ident.to_string(), "tag_prefix");
    assert_eq!(validators[0].variant_ident.to_string(), "TagPrefix");
}

#[test]
fn test_reserved_validator_label_errors() {
    let input: syn::DeriveInput = syn::parse_quote! {
        struct Demo {
            #[koruma(all = RequiredValidation)]
            value: String,
        }
    };

    let err =
        ValidationPlan::build(&input, "Koruma").expect_err("reserved validator labels should fail");
    assert!(err.to_string().contains("reserved"));
}

#[test]
fn test_validator_label_must_be_lower_snake() {
    let input: syn::DeriveInput = syn::parse_quote! {
        struct Demo {
            #[koruma(BadLabel = RequiredValidation)]
            value: String,
        }
    };

    let err = ValidationPlan::build(&input, "Koruma")
        .expect_err("non-lower-snake validator labels should fail");
    assert!(err.to_string().contains("lower-snake"));
}

#[test]
fn test_duplicate_validator_labels_error() {
    let input: syn::DeriveInput = syn::parse_quote! {
        struct Demo {
            #[koruma(
                length_check = MinLengthValidation,
                length_check = MaxLengthValidation,
            )]
            value: String,
        }
    };

    let err = ValidationPlan::build(&input, "Koruma")
        .expect_err("duplicate validator labels should fail");
    assert!(err.to_string().contains("collides"));
}

#[test]
fn test_validator_labels_cannot_match_field_names() {
    let input: syn::DeriveInput = syn::parse_quote! {
        struct Demo {
            #[koruma(other = RequiredValidation)]
            value: String,
            other: String,
        }
    };

    let err = ValidationPlan::build(&input, "Koruma")
        .expect_err("validator labels matching field names should fail");
    assert!(err.to_string().contains("generated field name"));
}
