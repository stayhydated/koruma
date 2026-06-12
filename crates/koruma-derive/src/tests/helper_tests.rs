//! Unit tests for helper functions in the expand module.

use crate::expand::{
    codegen::{Cardinality, ValidationSite, classify_each_collection, helper_generics_for_usages},
    effective_validation_type,
    plan::{
        ErrorStorage, FieldErrorShape, PlannedElementValidation, PlannedErrorDefault,
        PlannedErrorGetter, PlannedErrorIsEmpty, PlannedField, PlannedMainErrorStorage,
        PlannedValidationOperation, PlannedValidatorTypeArg, StructPlan, TargetBorrow,
        ValidationPlan, ValidationTarget,
    },
    validator::ValidatorBuilderPlan,
};
use koruma_derive_core::parse_validator_struct;
use koruma_derive_core::*;

use quote::{format_ident, quote};
use syn::ItemStruct;

#[test]
fn test_option_inner_type_extracts_inner() {
    let ty: syn::Type = syn::parse_quote!(Option<i32>);
    let inner = option_inner_type(&ty);
    assert!(inner.is_some());
    let inner_str = quote!(#inner).to_string();
    assert!(
        inner_str.contains("i32"),
        "Expected i32, got: {}",
        inner_str
    );
}

#[test]
fn test_option_inner_type_nested() {
    let ty: syn::Type = syn::parse_quote!(Option<Vec<String>>);
    let inner = option_inner_type(&ty);
    assert!(inner.is_some());
    let inner_str = quote!(#inner).to_string();
    assert!(
        inner_str.contains("Vec"),
        "Expected Vec<String>, got: {}",
        inner_str
    );
}

#[test]
fn test_option_inner_type_supports_qualified_paths() {
    let std_ty: syn::Type = syn::parse_quote!(std::option::Option<i32>);
    assert_eq!(
        option_inner_type(&std_ty).map(|inner| quote!(#inner).to_string()),
        Some("i32".to_string())
    );

    let core_ty: syn::Type = syn::parse_quote!(core::option::Option<String>);
    assert_eq!(
        option_inner_type(&core_ty).map(|inner| quote!(#inner).to_string()),
        Some("String".to_string())
    );
}

#[test]
fn test_option_inner_type_returns_none_for_non_option() {
    let ty: syn::Type = syn::parse_quote!(i32);
    assert!(option_inner_type(&ty).is_none());

    let ty: syn::Type = syn::parse_quote!(Vec<i32>);
    assert!(option_inner_type(&ty).is_none());

    let ty: syn::Type = syn::parse_quote!(String);
    assert!(option_inner_type(&ty).is_none());
}

#[test]
fn test_vec_inner_type_extracts_inner() {
    let ty: syn::Type = syn::parse_quote!(Vec<f64>);
    let inner = vec_inner_type(&ty);
    assert!(inner.is_some());
    let inner_str = quote!(#inner).to_string();
    assert!(
        inner_str.contains("f64"),
        "Expected f64, got: {}",
        inner_str
    );
}

#[test]
fn test_vec_inner_type_complex() {
    let ty: syn::Type = syn::parse_quote!(Vec<Option<String>>);
    let inner = vec_inner_type(&ty);
    assert!(inner.is_some());
    let inner_str = quote!(#inner).to_string();
    assert!(
        inner_str.contains("Option"),
        "Expected Option<String>, got: {}",
        inner_str
    );
}

#[test]
fn test_vec_inner_type_supports_qualified_paths() {
    let std_ty: syn::Type = syn::parse_quote!(std::vec::Vec<f64>);
    assert_eq!(
        vec_inner_type(&std_ty).map(|inner| quote!(#inner).to_string()),
        Some("f64".to_string())
    );

    let alloc_ty: syn::Type = syn::parse_quote!(alloc::vec::Vec<String>);
    assert_eq!(
        vec_inner_type(&alloc_ty).map(|inner| quote!(#inner).to_string()),
        Some("String".to_string())
    );
}

#[test]
fn test_vec_inner_type_returns_none_for_non_vec() {
    let ty: syn::Type = syn::parse_quote!(i32);
    assert!(vec_inner_type(&ty).is_none());

    let ty: syn::Type = syn::parse_quote!(Option<i32>);
    assert!(vec_inner_type(&ty).is_none());

    let ty: syn::Type = syn::parse_quote!(HashMap<String, i32>);
    assert!(vec_inner_type(&ty).is_none());
}

#[test]
fn test_effective_validation_type_for_each_on_optional_vec_uses_element_type() {
    let ty: syn::Type = syn::parse_quote!(Option<Vec<i32>>);
    let effective = effective_validation_type(&ty, ValidationSite::Element);
    assert_eq!(quote!(#effective).to_string(), "i32");
}

#[test]
fn test_effective_validation_type_for_each_on_qualified_option_vec_uses_element_type() {
    let ty: syn::Type =
        syn::parse_quote!(core::option::Option<std::vec::Vec<core::option::Option<String>>>);
    let effective = effective_validation_type(&ty, ValidationSite::Element);
    assert_eq!(quote!(#effective).to_string(), "String");
}

#[test]
fn test_effective_validation_type_for_each_on_vec_option_unwraps_inner_option() {
    let ty: syn::Type = syn::parse_quote!(Vec<Option<String>>);
    let effective = effective_validation_type(&ty, ValidationSite::Element);
    assert_eq!(quote!(#effective).to_string(), "String");
}

#[test]
fn test_effective_validation_type_for_each_on_slice_uses_element_type() {
    let ty: syn::Type = syn::parse_quote!(&[i32]);
    let effective = effective_validation_type(&ty, ValidationSite::Element);
    assert_eq!(quote!(#effective).to_string(), "i32");
}

#[test]
fn test_effective_validation_type_for_each_on_optional_slice_option_unwraps_inner_option() {
    let ty: syn::Type = syn::parse_quote!(Option<&[Option<String>]>);
    let effective = effective_validation_type(&ty, ValidationSite::Element);
    assert_eq!(quote!(#effective).to_string(), "String");
}

#[test]
fn test_helper_generics_tracks_lifetimes_consts_and_where_dependencies() {
    let item: ItemStruct = syn::parse_quote! {
        struct Demo<'a, 'b, T, U, const N: usize>
        where
            T: Into<U>,
            U: Clone,
            [u8; N]: Default,
            &'a T: Default,
            &'b str: Default,
        {
            value: &'a T,
        }
    };

    let usages: Vec<syn::Type> = vec![syn::parse_quote! { (&'a T, [u8; N], &'z str) }];
    let helper = helper_generics_for_usages(&item.generics, &usages);
    let definition_generics = &helper.definition;
    let definition = quote!(#definition_generics).to_string();
    assert!(definition.contains("'a"));
    assert!(!definition.contains("'b"));
    assert!(definition.contains("T"));
    assert!(definition.contains("U"));
    assert!(definition.contains("N"));

    let helper_ident = format_ident!("Helper");
    assert_eq!(
        helper.type_path(&helper_ident).to_string(),
        "Helper < 'a , T , U , N >"
    );
    assert!(helper.where_clause.to_string().contains("T : Into < U >"));
}

#[test]
fn test_helper_generics_ignores_non_generic_path_segments() {
    let item: ItemStruct = syn::parse_quote! {
        struct Demo<T, U, Result, const N: usize>
        where
            U: Iterator<Item = T>,
            Result: Default,
        {
            value: U,
        }
    };

    let usages: Vec<syn::Type> =
        vec![syn::parse_quote! { (::std::result::Result<U, ()>, [u8; N]) }];
    let helper = helper_generics_for_usages(&item.generics, &usages);
    let definition_generics = &helper.definition;
    let definition = quote!(#definition_generics).to_string();
    assert!(definition.contains("T"));
    assert!(definition.contains("U"));
    assert!(definition.contains("N"));
    assert!(!definition.contains("Result : Default"));

    let helper_ident = format_ident!("Helper");
    assert_eq!(
        helper.type_path(&helper_ident).to_string(),
        "Helper < T , U , N >"
    );
}

#[test]
fn test_each_collection_accepts_arrays_groups_and_parentheses() {
    let array_ty: syn::Type = syn::parse_quote!([i32; 3]);
    let array_collection =
        classify_each_collection(&array_ty).expect("arrays should support each(...)");
    assert_eq!(array_collection.outer_cardinality, Cardinality::Required);
    let array_element_ty = array_collection.element_ty;
    assert_eq!(quote!(#array_element_ty).to_string(), "i32");

    let paren_ty: syn::Type = syn::parse_quote!((Vec<i32>));
    let paren_collection =
        classify_each_collection(&paren_ty).expect("parenthesized Vec should support each(...)");
    let paren_element_ty = paren_collection.element_ty;
    assert_eq!(quote!(#paren_element_ty).to_string(), "i32");

    let group_ty = syn::Type::Group(syn::TypeGroup {
        group_token: Default::default(),
        elem: Box::new(syn::parse_quote!(Vec<i32>)),
    });
    let group_collection =
        classify_each_collection(&group_ty).expect("grouped Vec should support each(...)");
    let group_element_ty = group_collection.element_ty;
    assert_eq!(quote!(#group_element_ty).to_string(), "i32");
}

#[test]
fn test_each_collection_classifier_covers_supported_collection_shapes() {
    let optional_std_vec: syn::Type = syn::parse_quote!(Option<std::vec::Vec<Option<i32>>>);
    let collection =
        classify_each_collection(&optional_std_vec).expect("std::vec::Vec should classify");
    assert_eq!(collection.outer_cardinality, Cardinality::Optional);
    assert_eq!(collection.element_cardinality, Cardinality::Optional);
    let element_ty = collection.element_ty;
    assert_eq!(quote!(#element_ty).to_string(), "Option < i32 >");

    let alloc_vec: syn::Type = syn::parse_quote!(alloc::vec::Vec<String>);
    let collection = classify_each_collection(&alloc_vec).expect("alloc::vec::Vec should classify");
    let element_ty = collection.element_ty;
    assert_eq!(quote!(#element_ty).to_string(), "String");

    let slice: syn::Type = syn::parse_quote!(&[u8]);
    let collection = classify_each_collection(&slice).expect("borrowed slice should classify");
    let element_ty = collection.element_ty;
    assert_eq!(quote!(#element_ty).to_string(), "u8");

    let unsupported: syn::Type = syn::parse_quote!(std::collections::HashMap<String, String>);
    assert!(classify_each_collection(&unsupported).is_err());
}

#[test]
fn test_resolve_explicit_infer_type_reports_unmatched_shapes() {
    let input: syn::DeriveInput = syn::parse_quote! {
        struct BadInfer {
            #[koruma(GenericValidation::<std::collections::HashMap<_, _>>)]
            value: Option<String>,
        }
    };

    let err = ValidationPlan::build(&input, "Koruma")
        .expect_err("expected unmatched explicit infer shape to fail");
    assert!(err.to_string().contains("cannot infer `_`"));
}

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
                min_len = LengthValidation::<_>::min(3),
                max_len = LengthValidation::<_>::max(30),
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
                tag_prefix = string::PrefixValidation::<_>::prefix("tag:"),
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

#[test]
fn test_find_value_field_finds_marked_field() {
    let input: ItemStruct = syn::parse_quote! {
        pub struct Test {
            min: i32,
            max: i32,
            #[koruma(value)]
            checked: Option<i32>,
        }
    };

    let result = parse_validator_struct(&input).expect("expected validator struct parse");
    assert_eq!(result.value_field().name().to_string(), "checked");
}

#[test]
fn test_parse_validator_struct_errors_when_missing_value() {
    let input: ItemStruct = syn::parse_quote! {
        pub struct Test {
            #[koruma(setter)]
            min: i32,
            #[koruma(setter)]
            max: i32,
            #[koruma(setter)]
            checked: Option<i32>,
        }
    };

    assert!(
        parse_validator_struct(&input)
            .expect_err("expected missing value field")
            .to_string()
            .contains("requires a value field")
    );
}

#[test]
fn test_parse_field_with_single_validator() {
    let field: syn::Field = syn::parse_quote! {
        #[koruma(RangeValidation::min(0).max(100))]
        pub age: i32
    };

    let ParsedDataField::Participating(info) =
        parse_field(&field, 0).expect("expected field parse")
    else {
        panic!("expected validated field")
    };
    assert_eq!(info.name().to_string(), "age");
    assert_eq!(info.field_validators().len(), 1);
    assert_eq!(
        info.field_validators()[0].validator().name().to_string(),
        "RangeValidation"
    );
    assert!(!info.field_validators()[0].validator().uses_type_inference());
    assert_eq!(
        info.field_validators()[0].validator().setter_calls().len(),
        2
    );
    assert!(info.element_validators().is_empty());
}

#[test]
fn test_parse_field_with_generic_validator() {
    let field: syn::Field = syn::parse_quote! {
        #[koruma(GenericRange::<_>::min(0.0).max(1.0))]
        pub score: f64
    };

    let ParsedDataField::Participating(info) =
        parse_field(&field, 0).expect("expected field parse")
    else {
        panic!("expected validated field")
    };
    assert!(info.field_validators()[0].validator().uses_type_inference());
}

#[test]
fn test_parse_field_with_each() {
    let field: syn::Field = syn::parse_quote! {
        #[koruma(each(RangeValidation::min(0).max(100)))]
        pub scores: Vec<i32>
    };

    let ParsedDataField::Participating(info) =
        parse_field(&field, 0).expect("expected field parse")
    else {
        panic!("expected validated field")
    };
    assert!(info.field_validators().is_empty());
    assert_eq!(info.element_validators().len(), 1);
}

#[test]
fn test_parse_field_with_skip_returns_skip() {
    let field: syn::Field = syn::parse_quote! {
        #[koruma(skip)]
        pub internal: u64
    };

    assert!(matches!(
        parse_field(&field, 0).expect("expected field parse"),
        ParsedDataField::Skipped { .. }
    ));
}

#[test]
fn test_parse_field_without_koruma_returns_skip() {
    let field: syn::Field = syn::parse_quote! {
        pub normal_field: String
    };

    assert!(matches!(
        parse_field(&field, 0).expect("expected field parse"),
        ParsedDataField::Unannotated(_)
    ));
}

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
            #[koruma(full(RequiredValidation::<_>), LengthValidation::<_>::min(1))]
            name: Option<String>,
            #[koruma(each(full(RequiredValidation::<_>), ItemLength::<_>::min(1)))]
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

#[test]
fn test_validation_plan_exposes_render_ready_validation_operations() {
    let input: syn::DeriveInput = syn::parse_quote! {
        struct Planned {
            #[koruma(full(RequiredValidation::<_>), LengthValidation::<_>::min(1))]
            name: Option<String>,
            #[koruma(each(full(RequiredValidation::<_>), ItemLength::<_>::min(1)))]
            tags: Vec<Option<String>>,
        }
    };

    let plan = ValidationPlan::build(&input, "Koruma").expect("expected plan");
    let render_plan = plan.validation_render_plan();
    insta::assert_debug_snapshot!(
        validation_render_plan_summary(&render_plan),
        @r###"
    [
        (
            "regular_optional",
            "name",
            1,
            1,
            "none",
        ),
        (
            "regular_required",
            "tags",
            0,
            0,
            "required_collection_optional_element(full=1, unwrapped=1)",
        ),
    ]
    "###
    );
    let operations = &render_plan.operations;
    assert_eq!(operations.len(), 2);

    let PlannedValidationOperation::RegularOptional(name_operation) = &operations[0] else {
        panic!("expected optional regular validation operation");
    };
    assert_eq!(name_operation.field.name.to_string(), "name");
    assert_eq!(
        name_operation.field_validators.full_type_validators.len(),
        1
    );
    assert_eq!(
        name_operation.field_validators.unwrapped_validators.len(),
        1
    );
    assert!(name_operation.element_validators.is_none());

    let PlannedValidationOperation::RegularRequired(tags_operation) = &operations[1] else {
        panic!("expected required regular validation operation");
    };
    assert!(!tags_operation.field_validators.has_any());
    let element = tags_operation
        .element_validators
        .as_ref()
        .expect("expected element operation");
    let PlannedElementValidation::RequiredCollectionOptional(element) = element else {
        panic!("expected optional element validation operation");
    };
    assert_eq!(element.full_type_validators.len(), 1);
    assert_eq!(element.unwrapped_validators.len(), 1);
}

fn validation_render_plan_summary(
    render_plan: &crate::expand::plan::ValidationRenderPlan<'_>,
) -> Vec<(&'static str, String, usize, usize, String)> {
    render_plan
        .operations
        .iter()
        .map(|operation| match operation {
            PlannedValidationOperation::NestedRequired(operation) => (
                "nested_required",
                operation.field.name.to_string(),
                0,
                0,
                "none".to_owned(),
            ),
            PlannedValidationOperation::NestedOptional(operation) => (
                "nested_optional",
                operation.field.name.to_string(),
                0,
                0,
                "none".to_owned(),
            ),
            PlannedValidationOperation::NewtypeRequired(operation) => (
                "newtype_required",
                operation.field.name.to_string(),
                operation.field_validators.full_type_validators.len(),
                operation.field_validators.unwrapped_validators.len(),
                "none".to_owned(),
            ),
            PlannedValidationOperation::NewtypeOptional(operation) => (
                "newtype_optional",
                operation.field.name.to_string(),
                operation.field_validators.full_type_validators.len(),
                operation.field_validators.unwrapped_validators.len(),
                "none".to_owned(),
            ),
            PlannedValidationOperation::RegularRequired(operation) => (
                "regular_required",
                operation.field.name.to_string(),
                operation.field_validators.full_type_validators.len(),
                operation.field_validators.unwrapped_validators.len(),
                element_summary(operation.element_validators.as_ref()),
            ),
            PlannedValidationOperation::RegularOptional(operation) => (
                "regular_optional",
                operation.field.name.to_string(),
                operation.field_validators.full_type_validators.len(),
                operation.field_validators.unwrapped_validators.len(),
                element_summary(operation.element_validators.as_ref()),
            ),
        })
        .collect()
}

fn element_summary(element: Option<&PlannedElementValidation<'_>>) -> String {
    match element {
        None => "none".to_owned(),
        Some(PlannedElementValidation::RequiredCollectionRequired(element)) => format!(
            "required_collection_required_element(full={}, unwrapped={})",
            element.full_type_validators.len(),
            element.unwrapped_validators.len()
        ),
        Some(PlannedElementValidation::RequiredCollectionOptional(element)) => format!(
            "required_collection_optional_element(full={}, unwrapped={})",
            element.full_type_validators.len(),
            element.unwrapped_validators.len()
        ),
        Some(PlannedElementValidation::OptionalCollectionRequired(element)) => format!(
            "optional_collection_required_element(full={}, unwrapped={})",
            element.full_type_validators.len(),
            element.unwrapped_validators.len()
        ),
        Some(PlannedElementValidation::OptionalCollectionOptional(element)) => format!(
            "optional_collection_optional_element(full={}, unwrapped={})",
            element.full_type_validators.len(),
            element.unwrapped_validators.len()
        ),
    }
}

#[test]
fn test_validation_plan_exposes_main_error_render_plan() {
    let input: syn::DeriveInput = syn::parse_quote! {
        #[koruma(newtype)]
        struct Planned(#[koruma(nested)] Child);
    };

    let plan = ValidationPlan::build(&input, "Koruma").expect("expected plan");
    let layout = plan.main_error_render_plan();
    assert_eq!(layout.fields.len(), 1);
    assert_eq!(layout.fields[0].field.name.to_string(), "_0");
    assert_eq!(
        layout.fields[0].storage,
        PlannedMainErrorStorage::NestedDirect
    );
    assert_eq!(layout.fields[0].getter, PlannedErrorGetter::NestedDirect);
    assert_eq!(layout.fields[0].default, PlannedErrorDefault::NestedDirect);
    assert_eq!(layout.fields[0].is_empty, PlannedErrorIsEmpty::NestedDirect);

    let input: syn::DeriveInput = syn::parse_quote! {
        struct Planned {
            #[koruma(nested)]
            child: Option<Child>,
            #[koruma(newtype)]
            raw: Wrapper,
            #[koruma(newtype)]
            wrapper: Option<Wrapper>,
            #[koruma(newtype, RequiredValidation)]
            checked: Wrapper,
            #[koruma(newtype, RequiredValidation)]
            maybe_checked: Option<Wrapper>,
            #[koruma(RangeValidation::min(0), each(ItemValidation))]
            values: Vec<i32>,
        }
    };

    let plan = ValidationPlan::build(&input, "Koruma").expect("expected plan");
    let layout = plan.main_error_render_plan();
    assert_eq!(layout.fields.len(), 6);

    assert_eq!(
        layout.fields[0].storage,
        PlannedMainErrorStorage::NestedOptional
    );
    assert_eq!(layout.fields[0].getter, PlannedErrorGetter::NestedOptional);
    assert_eq!(layout.fields[0].default, PlannedErrorDefault::None);
    assert_eq!(
        layout.fields[0].is_empty,
        PlannedErrorIsEmpty::NestedOptional
    );

    assert_eq!(
        layout.fields[1].storage,
        PlannedMainErrorStorage::FieldError
    );
    assert_eq!(
        layout.fields[1].getter,
        PlannedErrorGetter::NewtypeInnerDirect
    );
    assert_eq!(
        layout.fields[1].default,
        PlannedErrorDefault::FieldErrorDefault
    );
    assert_eq!(layout.fields[1].is_empty, PlannedErrorIsEmpty::FieldError);

    assert_eq!(
        layout.fields[2].storage,
        PlannedMainErrorStorage::FieldError
    );
    assert_eq!(
        layout.fields[2].getter,
        PlannedErrorGetter::NewtypeInnerOptional
    );
    assert_eq!(
        layout.fields[2].default,
        PlannedErrorDefault::FieldErrorDefault
    );
    assert_eq!(layout.fields[2].is_empty, PlannedErrorIsEmpty::FieldError);

    assert_eq!(
        layout.fields[3].storage,
        PlannedMainErrorStorage::FieldError
    );
    assert_eq!(layout.fields[3].getter, PlannedErrorGetter::FieldError);
    assert_eq!(
        layout.fields[3].default,
        PlannedErrorDefault::NewtypeWithValidators {
            inner_optional: false,
        }
    );
    assert_eq!(layout.fields[3].is_empty, PlannedErrorIsEmpty::FieldError);

    assert_eq!(
        layout.fields[4].storage,
        PlannedMainErrorStorage::FieldError
    );
    assert_eq!(layout.fields[4].getter, PlannedErrorGetter::FieldError);
    assert_eq!(
        layout.fields[4].default,
        PlannedErrorDefault::NewtypeWithValidators {
            inner_optional: true,
        }
    );
    assert_eq!(layout.fields[4].is_empty, PlannedErrorIsEmpty::FieldError);

    assert_eq!(
        layout.fields[5].storage,
        PlannedMainErrorStorage::FieldError
    );
    assert_eq!(layout.fields[5].getter, PlannedErrorGetter::FieldError);
    assert_eq!(
        layout.fields[5].default,
        PlannedErrorDefault::Regular {
            has_field_validators: true,
            has_element_validators: true,
        }
    );
    assert_eq!(layout.fields[5].is_empty, PlannedErrorIsEmpty::FieldError);
}

#[test]
fn test_validation_plan_exposes_field_error_render_plan() {
    let input: syn::DeriveInput = syn::parse_quote! {
        struct Planned {
            #[koruma(nested)]
            child: Child,
            #[koruma(newtype)]
            raw: Wrapper,
            #[koruma(newtype)]
            wrapper: Option<Wrapper>,
            #[koruma(newtype, RequiredValidation)]
            checked: Wrapper,
            #[koruma(RequiredValidation)]
            name: String,
            #[koruma(each(ItemValidation))]
            tags: Vec<String>,
            #[koruma(LengthValidation::min(1), each(ItemValidation))]
            values: Vec<String>,
        }
    };

    let plan = ValidationPlan::build(&input, "Koruma").expect("expected plan");
    let layout = plan.field_error_render_plan();
    assert_eq!(layout.fields.len(), 6);

    assert_eq!(layout.fields[0].field().name.to_string(), "raw");
    assert_eq!(
        layout.fields[0].shape(),
        FieldErrorShape::NewtypeInnerRequired
    );

    assert_eq!(layout.fields[1].field().name.to_string(), "wrapper");
    assert_eq!(
        layout.fields[1].shape(),
        FieldErrorShape::NewtypeInnerOptional
    );

    assert_eq!(layout.fields[2].field().name.to_string(), "checked");
    assert_eq!(
        layout.fields[2].shape(),
        FieldErrorShape::NewtypeWithValidatorsRequired
    );
    assert_eq!(layout.fields[2].field_validators().len(), 1);

    assert_eq!(layout.fields[3].field().name.to_string(), "name");
    assert_eq!(layout.fields[3].shape(), FieldErrorShape::RegularFieldOnly);

    assert_eq!(layout.fields[4].field().name.to_string(), "tags");
    assert_eq!(
        layout.fields[4].shape(),
        FieldErrorShape::RegularElementOnly
    );
    assert_eq!(layout.fields[4].element_validators().len(), 1);

    assert_eq!(layout.fields[5].field().name.to_string(), "values");
    assert_eq!(
        layout.fields[5].shape(),
        FieldErrorShape::RegularFieldAndElement
    );
    assert_eq!(layout.fields[5].field_validators().len(), 1);
    assert_eq!(layout.fields[5].element_validators().len(), 1);
}

#[test]
fn test_validation_plan_uses_shape_specific_field_data() {
    let input: syn::DeriveInput = syn::parse_quote! {
        struct PlannedShapes {
            #[koruma(nested)]
            child: Child,
            #[koruma(newtype, RequiredValidation)]
            wrapped: Option<Wrapped>,
        }
    };

    let plan = ValidationPlan::build(&input, "Koruma").expect("expected plan");
    assert_eq!(plan.fields.len(), 2);

    let PlannedField::Nested(nested) = &plan.fields[0].shape else {
        panic!("expected nested planned field");
    };
    assert_eq!(nested.cardinality, Cardinality::Required);
    let nested_inner_type = &nested.inner_type;
    assert_eq!(quote!(#nested_inner_type).to_string(), "Child");
    assert!(plan.fields[0].field_validators().is_empty());
    assert!(plan.fields[0].element_validators().is_empty());
    assert!(matches!(
        plan.fields[0].error_storage(),
        ErrorStorage::Nested {
            cardinality: Cardinality::Required
        }
    ));

    let PlannedField::Newtype(newtype) = &plan.fields[1].shape else {
        panic!("expected newtype planned field");
    };
    assert_eq!(newtype.cardinality, Cardinality::Optional);
    let newtype_inner_type = &newtype.inner_type;
    assert_eq!(quote!(#newtype_inner_type).to_string(), "Wrapped");
    assert_eq!(newtype.field_validators.len(), 1);
    assert!(plan.fields[1].element_validators().is_empty());
    assert!(matches!(
        plan.fields[1].error_storage(),
        ErrorStorage::NewtypeWithValidators {
            cardinality: Cardinality::Optional
        }
    ));
}

#[test]
fn test_validation_plan_encodes_struct_level_newtype_shape() {
    let input: syn::DeriveInput = syn::parse_quote! {
        #[koruma(newtype)]
        struct Email(#[koruma(newtype)] InnerEmail);
    };

    let plan = ValidationPlan::build(&input, "Koruma").expect("expected plan");
    let Some(field_plan) = plan.struct_newtype() else {
        panic!("expected struct-level newtype plan");
    };
    assert_eq!(plan.fields.len(), 1);
    assert!(matches!(
        plan.struct_plan,
        StructPlan::Newtype { field_index: 0 }
    ));
    assert_eq!(field_plan.name.to_string(), "_0");
    assert_eq!(field_plan.source.index, 0);
    assert!(field_plan.is_newtype());
}
