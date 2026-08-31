use super::support::*;

#[test]
fn test_validator_builder_plan_covers_setter_signatures_and_capture_policy() {
    let input: ItemStruct = syn::parse_quote! {
        pub struct PlannedBuilderValidation {
            #[koruma(skip_capture)]
            actual: Option<String>,
            #[koruma(setter(into, name = label))]
            title: String,
            #[koruma(setter(required))]
            required_limit: Option<usize>,
            optional_limit: Option<usize>,
            #[koruma(setter(default = false))]
            enabled: bool,
            #[koruma(setter(default = Some(3)))]
            defaulted_optional: Option<usize>,
        }
    };

    let plan = ValidatorBuilderPlan::test_build(&input).expect("expected builder plan");
    assert_eq!(plan.capture_policy(), CapturePolicy::Skip);
    let summaries = plan.slot_summaries();
    let compact: Vec<_> = summaries
        .iter()
        .map(|summary| {
            (
                summary.name.as_str(),
                summary.kind,
                summary.required,
                summary.state_ident.as_deref(),
                summary.method.as_deref(),
                summary.signature.as_deref(),
                summary.maybe_method.as_deref(),
            )
        })
        .collect();

    assert_eq!(
        compact,
        vec![
            ("actual", "skipped_value", false, None, None, None, None),
            (
                "title",
                "required_setter",
                true,
                Some("__KorumaTitleState"),
                Some("label"),
                Some("into(String)"),
                None
            ),
            (
                "required_limit",
                "required_setter",
                true,
                Some("__KorumaRequiredLimitState"),
                Some("required_limit"),
                Some("optional_exact(Option < usize >)"),
                None
            ),
            (
                "optional_limit",
                "optional_setter",
                false,
                None,
                Some("optional_limit"),
                Some("optional_inner(usize, into=false)"),
                Some("maybe_optional_limit")
            ),
            (
                "enabled",
                "defaulted_setter",
                false,
                None,
                Some("enabled"),
                Some("exact(bool)"),
                None
            ),
            (
                "defaulted_optional",
                "defaulted_setter",
                false,
                None,
                Some("defaulted_optional"),
                Some("exact(Option < usize >)"),
                None
            ),
        ]
    );
    assert_eq!(
        plan.direct_method_summaries(),
        vec![
            ("label".to_owned(), None),
            ("required_limit".to_owned(), None),
            (
                "optional_limit".to_owned(),
                Some("maybe_optional_limit".to_owned())
            ),
            ("enabled".to_owned(), None),
            ("defaulted_optional".to_owned(), None),
        ]
    );
}

#[test]
fn test_validation_plan_resolves_targets_names_and_type_args() {
    let input: syn::DeriveInput = syn::parse_quote! {
        struct Planned {
            #[koruma(full(RequiredValidation::<_>), LengthValidation::<_>.min(1))]
            name: Option<String>,
            #[koruma(each(full(RequiredValidation::<_>), ItemLength::<_>.min(1)))]
            tags: Vec<Option<String>>,
        }
    };

    let plan = ValidationPlan::build(&input, "Koruma").expect("expected plan");
    assert_eq!(plan.fields.len(), 2);
    assert!(matches!(plan.struct_plan, StructPlan::Record));
    assert_eq!(
        plan.main_error_struct.to_string(),
        "PlannedKorumaValidationError"
    );
    assert_eq!(
        plan.fields[0]
            .generated_names
            .field_error_struct
            .to_string(),
        "PlannedNameKorumaValidationError"
    );
    assert_eq!(
        plan.fields[1]
            .generated_names
            .element_validator_ref_enum
            .to_string(),
        "PlannedTagsElementKorumaValidatorRef"
    );
    assert!(matches!(plan.fields[0].shape, PlannedField::Regular(_)));
    assert!(matches!(plan.fields[1].shape, PlannedField::Regular(_)));

    let full_field_target = &plan.fields[0].field_validators()[0].target;
    let ValidationTarget::FieldFull(full_field_target) = full_field_target else {
        panic!("expected full field target");
    };
    assert_eq!(full_field_target.cardinality, Cardinality::Optional);
    assert_eq!(full_field_target.borrow, TargetBorrow::Reference);
    let full_field_raw_type = &full_field_target.ty;
    let full_field_validate_type = &full_field_target.ty;
    assert_eq!(
        quote!(#full_field_raw_type).to_string(),
        "Option < String >"
    );
    assert_eq!(
        quote!(#full_field_validate_type).to_string(),
        "Option < String >"
    );

    let unwrapped_field_target = &plan.fields[0].field_validators()[1].target;
    let ValidationTarget::FieldUnwrapped(unwrapped_field_target) = unwrapped_field_target else {
        panic!("expected unwrapped field target");
    };
    assert_eq!(unwrapped_field_target.borrow, TargetBorrow::AlreadyBorrowed);
    let unwrapped_field_raw_type = &unwrapped_field_target.raw_type;
    let unwrapped_field_validate_type = &unwrapped_field_target.validate_type;
    assert_eq!(
        quote!(#unwrapped_field_raw_type).to_string(),
        "Option < String >"
    );
    assert_eq!(quote!(#unwrapped_field_validate_type).to_string(), "String");

    let full_element_target = &plan.fields[1].element_validators()[0].target;
    let ValidationTarget::ElementFull(full_element_target) = full_element_target else {
        panic!("expected full element target");
    };
    assert_eq!(full_element_target.cardinality, Cardinality::Optional);
    assert_eq!(full_element_target.borrow, TargetBorrow::AlreadyBorrowed);
    let full_element_raw_type = &full_element_target.ty;
    let full_element_validate_type = &full_element_target.ty;
    assert_eq!(
        quote!(#full_element_raw_type).to_string(),
        "Option < String >"
    );
    assert_eq!(
        quote!(#full_element_validate_type).to_string(),
        "Option < String >"
    );

    let unwrapped_element_target = &plan.fields[1].element_validators()[1].target;
    let ValidationTarget::ElementUnwrapped(unwrapped_element_target) = unwrapped_element_target
    else {
        panic!("expected unwrapped element target");
    };
    assert_eq!(
        unwrapped_element_target.borrow,
        TargetBorrow::AlreadyBorrowed
    );
    let unwrapped_element_raw_type = &unwrapped_element_target.raw_type;
    let unwrapped_element_validate_type = &unwrapped_element_target.validate_type;
    assert_eq!(
        quote!(#unwrapped_element_raw_type).to_string(),
        "Option < String >"
    );
    assert_eq!(
        quote!(#unwrapped_element_validate_type).to_string(),
        "String"
    );
    assert!(plan.fields[0].field_optional());
    assert!(plan.fields[1].element_optional());
    let name_inner_type = &plan.fields[0].inner_type();
    assert_eq!(quote!(#name_inner_type).to_string(), "String");
    let tags_element_type = plan.fields[1]
        .element_type()
        .expect("expected planned element type");
    assert_eq!(quote!(#tags_element_type).to_string(), "Option < String >");
    assert_eq!(plan.fields[0].full_field_validators().count(), 1);
    assert_eq!(plan.fields[0].unwrapped_field_validators().count(), 1);
    assert_eq!(plan.fields[1].full_element_validators().count(), 1);
    assert_eq!(plan.fields[1].unwrapped_element_validators().count(), 1);
    assert!(matches!(
        plan.fields[0].error_storage(),
        ErrorStorage::RegularFieldValidators
    ));
    assert!(matches!(
        plan.fields[1].error_storage(),
        ErrorStorage::RegularElementValidators
    ));
    let required_builder = &plan.fields[0].field_validators()[0].builder_type;
    assert_eq!(
        quote!(#required_builder).to_string(),
        "RequiredValidation < Option < String > >"
    );
    assert!(plan.fields[0].field_validators()[0].setter_calls.is_empty());
    assert_eq!(
        plan.fields[0].field_validators()[1].setter_calls[0]
            .method
            .to_string(),
        "min"
    );

    let PlannedValidatorTypeArg::Resolved(resolved_ty) =
        &plan.fields[1].element_validators()[1].resolved_type_arg
    else {
        panic!("expected inferred element validator type");
    };
    let resolved_ty = resolved_ty.as_ref();
    assert_eq!(quote!(#resolved_ty).to_string(), "String");
}

#[test]
fn test_required_validation_name_does_not_select_full_target() {
    let input: syn::DeriveInput = syn::parse_quote! {
        struct Planned {
            #[koruma(RequiredValidation::<_>)]
            name: Option<String>,
        }
    };

    let plan = ValidationPlan::build(&input, "Koruma").expect("expected plan");
    let target = &plan.fields[0].field_validators()[0].target;
    let ValidationTarget::FieldUnwrapped(target) = target else {
        panic!("RequiredValidation should use the default unwrapped target without full(...)");
    };
    let validate_type = &target.validate_type;
    assert_eq!(quote!(#validate_type).to_string(), "String");
}

#[test]
fn test_validation_plan_infers_full_targets_from_explicit_option_types() {
    let input: syn::DeriveInput = syn::parse_quote! {
        struct Planned {
            #[koruma(GenericPresence::<Option<_>>)]
            value: Option<i32>,
            #[koruma(each(GenericElementPresence::<Option<_>>))]
            values: Vec<Option<String>>,
        }
    };

    let plan = ValidationPlan::build(&input, "Koruma").expect("expected plan");

    let field_validator = &plan.fields[0].field_validators()[0];
    let ValidationTarget::FieldFull(full_field_target) = &field_validator.target else {
        panic!("expected explicit Option field validator to infer full field target");
    };
    let field_ty = &full_field_target.ty;
    assert_eq!(quote!(#field_ty).to_string(), "Option < i32 >");
    let PlannedValidatorTypeArg::Resolved(resolved_field_ty) = &field_validator.resolved_type_arg
    else {
        panic!("expected explicit Option field type to resolve");
    };
    let resolved_field_ty = resolved_field_ty.as_ref();
    assert_eq!(quote!(#resolved_field_ty).to_string(), "Option < i32 >");

    let element_validator = &plan.fields[1].element_validators()[0];
    let ValidationTarget::ElementFull(full_element_target) = &element_validator.target else {
        panic!("expected explicit Option element validator to infer full element target");
    };
    let element_ty = &full_element_target.ty;
    assert_eq!(quote!(#element_ty).to_string(), "Option < String >");
    let PlannedValidatorTypeArg::Resolved(resolved_element_ty) =
        &element_validator.resolved_type_arg
    else {
        panic!("expected explicit Option element type to resolve");
    };
    let resolved_element_ty = resolved_element_ty.as_ref();
    assert_eq!(
        quote!(#resolved_element_ty).to_string(),
        "Option < String >"
    );
}
