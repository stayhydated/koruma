//! Unit tests for helper functions in the expand module.

use crate::expand::{
    codegen::{
        EachIterationKind, FieldCardinality, ValidationSite, classify_each_collection,
        helper_generics_for_usages,
    },
    effective_validation_type,
    plan::{
        ErrorStorage, PlannedErrorDefault, PlannedErrorGetter, PlannedErrorIsEmpty, PlannedField,
        PlannedFieldErrorKind, PlannedMainErrorStorage, PlannedRegularAll,
        PlannedRegularFieldErrorDoc, PlannedRegularFieldErrorIsEmpty,
        PlannedRegularFieldErrorStorage, PlannedValidationOperation, PlannedValidatorTypeArg,
        StructPlan, TargetAccess, TargetCardinality, TargetPolicy, TargetScope, ValidationPlan,
    },
};
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
fn test_validator_wants_full_type_only_for_full_wrapper() {
    let attr: ValidatorAttr = syn::parse_quote!(RequiredValidation::<Option<String>>);
    assert!(!attr.wants_full_target());

    let qualified_attr: ValidatorAttr =
        syn::parse_quote!(RequiredValidation::<core::option::Option<String>>);
    assert!(!qualified_attr.wants_full_target());

    let non_option_attr: ValidatorAttr = syn::parse_quote!(RequiredValidation::<String>);
    assert!(!non_option_attr.wants_full_target());

    let full_attr: ValidatorAttr = syn::parse_quote!(full(RequiredValidation::<_>));
    assert!(full_attr.wants_full_target());
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
    assert_eq!(array_collection.iteration, EachIterationKind::Array);
    assert_eq!(
        array_collection.outer_cardinality,
        FieldCardinality::Required
    );
    let array_element_ty = array_collection.element_ty;
    assert_eq!(quote!(#array_element_ty).to_string(), "i32");

    let paren_ty: syn::Type = syn::parse_quote!((Vec<i32>));
    let paren_collection =
        classify_each_collection(&paren_ty).expect("parenthesized Vec should support each(...)");
    assert_eq!(paren_collection.iteration, EachIterationKind::VecLike);
    let paren_element_ty = paren_collection.element_ty;
    assert_eq!(quote!(#paren_element_ty).to_string(), "i32");

    let group_ty = syn::Type::Group(syn::TypeGroup {
        group_token: Default::default(),
        elem: Box::new(syn::parse_quote!(Vec<i32>)),
    });
    let group_collection =
        classify_each_collection(&group_ty).expect("grouped Vec should support each(...)");
    assert_eq!(group_collection.iteration, EachIterationKind::VecLike);
    let group_collection_ty = group_collection.collection_ty;
    let group_element_ty = group_collection.element_ty;
    assert_eq!(quote!(#group_collection_ty).to_string(), "Vec < i32 >");
    assert_eq!(quote!(#group_element_ty).to_string(), "i32");
}

#[test]
fn test_each_collection_classifier_covers_supported_collection_shapes() {
    let optional_std_vec: syn::Type = syn::parse_quote!(Option<std::vec::Vec<Option<i32>>>);
    let collection =
        classify_each_collection(&optional_std_vec).expect("std::vec::Vec should classify");
    assert_eq!(collection.iteration, EachIterationKind::VecLike);
    assert_eq!(collection.outer_cardinality, FieldCardinality::Optional);
    assert_eq!(collection.element_cardinality, FieldCardinality::Optional);
    let collection_ty = collection.collection_ty;
    let element_ty = collection.element_ty;
    assert_eq!(
        quote!(#collection_ty).to_string(),
        "std :: vec :: Vec < Option < i32 > >"
    );
    assert_eq!(quote!(#element_ty).to_string(), "Option < i32 >");

    let alloc_vec: syn::Type = syn::parse_quote!(alloc::vec::Vec<String>);
    let collection = classify_each_collection(&alloc_vec).expect("alloc::vec::Vec should classify");
    assert_eq!(collection.iteration, EachIterationKind::VecLike);
    let element_ty = collection.element_ty;
    assert_eq!(quote!(#element_ty).to_string(), "String");

    let slice: syn::Type = syn::parse_quote!(&[u8]);
    let collection = classify_each_collection(&slice).expect("borrowed slice should classify");
    assert_eq!(collection.iteration, EachIterationKind::Slice);
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
            actual: Option<i32>,
        }
    };

    let result = find_value_field_strict(&input).expect("expected value lookup");
    assert!(result.is_some());
    let (name, _ty) = result.unwrap();
    assert_eq!(name.to_string(), "actual");
}

#[test]
fn test_find_value_field_returns_none_when_missing() {
    let input: ItemStruct = syn::parse_quote! {
        pub struct Test {
            min: i32,
            max: i32,
            actual: Option<i32>,
        }
    };

    assert!(
        find_value_field_strict(&input)
            .expect("expected value lookup")
            .is_none()
    );
}

#[test]
fn test_parse_field_with_single_validator() {
    let field: syn::Field = syn::parse_quote! {
        #[koruma(RangeValidation::min(0).max(100))]
        pub age: i32
    };

    let info = parse_field(&field, 0)
        .expect("expected field parse")
        .expect("expected validated field");
    assert_eq!(info.name.to_string(), "age");
    assert_eq!(info.field_validators().len(), 1);
    assert_eq!(
        info.field_validators()[0].validator.name().to_string(),
        "RangeValidation"
    );
    assert!(!info.field_validators()[0].validator.uses_type_inference());
    assert_eq!(
        info.field_validators()[0].validator.builder_methods.len(),
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

    let info = parse_field(&field, 0)
        .expect("expected field parse")
        .expect("expected validated field");
    assert!(info.field_validators()[0].validator.uses_type_inference());
}

#[test]
fn test_parse_field_with_each() {
    let field: syn::Field = syn::parse_quote! {
        #[koruma(each(RangeValidation::min(0).max(100)))]
        pub scores: Vec<i32>
    };

    let info = parse_field(&field, 0)
        .expect("expected field parse")
        .expect("expected validated field");
    assert!(info.field_validators().is_empty());
    assert_eq!(info.element_validators().len(), 1);
}

#[test]
fn test_parse_field_with_skip_returns_skip() {
    let field: syn::Field = syn::parse_quote! {
        #[koruma(skip)]
        pub internal: u64
    };

    assert!(
        parse_field(&field, 0)
            .expect("expected field parse")
            .is_none()
    );
}

#[test]
fn test_parse_field_without_koruma_returns_skip() {
    let field: syn::Field = syn::parse_quote! {
        pub normal_field: String
    };

    assert!(
        parse_field(&field, 0)
            .expect("expected field parse")
            .is_none()
    );
}

#[test]
fn test_validation_plan_resolves_targets_names_and_type_args() {
    let input: syn::DeriveInput = syn::parse_quote! {
        struct Planned {
            #[koruma(full(RequiredValidation::<_>), LengthValidation::<_>::min(1))]
            name: Option<String>,
            #[koruma(each(full(ItemRequired::<_>), ItemLength::<_>::min(1)))]
            tags: Vec<Option<String>>,
        }
    };

    let plan = ValidationPlan::build(&input, "Koruma").expect("expected plan");
    assert_eq!(plan.fields.len(), 2);
    assert!(matches!(plan.struct_plan, StructPlan::Record));
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
    assert_eq!(full_field_target.scope, TargetScope::Field);
    assert_eq!(full_field_target.policy, TargetPolicy::Full);
    assert_eq!(full_field_target.cardinality, TargetCardinality::Optional);
    assert_eq!(full_field_target.access, TargetAccess::BorrowField);
    let full_field_raw_type = &full_field_target.raw_type;
    let full_field_validate_type = &full_field_target.validate_type;
    assert_eq!(
        quote!(#full_field_raw_type).to_string(),
        "Option < String >"
    );
    assert_eq!(
        quote!(#full_field_validate_type).to_string(),
        "Option < String >"
    );

    let unwrapped_field_target = &plan.fields[0].field_validators()[1].target;
    assert_eq!(unwrapped_field_target.scope, TargetScope::Field);
    assert_eq!(unwrapped_field_target.policy, TargetPolicy::UnwrapOption);
    assert_eq!(
        unwrapped_field_target.access,
        TargetAccess::AlreadyBorrowedLocal
    );
    let unwrapped_field_raw_type = &unwrapped_field_target.raw_type;
    let unwrapped_field_validate_type = &unwrapped_field_target.validate_type;
    assert_eq!(
        quote!(#unwrapped_field_raw_type).to_string(),
        "Option < String >"
    );
    assert_eq!(quote!(#unwrapped_field_validate_type).to_string(), "String");

    let full_element_target = &plan.fields[1].element_validators()[0].target;
    assert_eq!(full_element_target.scope, TargetScope::Element);
    assert_eq!(full_element_target.policy, TargetPolicy::Full);
    assert_eq!(full_element_target.cardinality, TargetCardinality::Optional);
    assert_eq!(
        full_element_target.access,
        TargetAccess::AlreadyBorrowedLocal
    );
    let full_element_raw_type = &full_element_target.raw_type;
    let full_element_validate_type = &full_element_target.validate_type;
    assert_eq!(
        quote!(#full_element_raw_type).to_string(),
        "Option < String >"
    );
    assert_eq!(
        quote!(#full_element_validate_type).to_string(),
        "Option < String >"
    );

    let unwrapped_element_target = &plan.fields[1].element_validators()[1].target;
    assert_eq!(unwrapped_element_target.scope, TargetScope::Element);
    assert_eq!(unwrapped_element_target.policy, TargetPolicy::UnwrapOption);
    assert_eq!(
        unwrapped_element_target.access,
        TargetAccess::AlreadyBorrowedLocal
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
    let PlannedField::Regular(tags_plan) = &plan.fields[1].shape else {
        panic!("expected regular tags field");
    };
    let tags_collection_type = tags_plan
        .collection_type
        .as_ref()
        .expect("expected classified collection type");
    assert_eq!(
        quote!(#tags_collection_type).to_string(),
        "Vec < Option < String > >"
    );
    assert_eq!(tags_plan.each_iteration, Some(EachIterationKind::VecLike));
    assert_eq!(plan.fields[0].full_field_validators().count(), 1);
    assert_eq!(plan.fields[0].unwrapped_field_validators().count(), 1);
    assert_eq!(plan.fields[1].full_element_validators().count(), 1);
    assert_eq!(plan.fields[1].unwrapped_element_validators().count(), 1);
    assert!(matches!(
        plan.fields[0].error_storage,
        ErrorStorage::RegularFieldValidators
    ));
    assert!(matches!(
        plan.fields[1].error_storage,
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
    assert_eq!(quote!(#resolved_ty).to_string(), "String");
}

#[test]
fn test_validation_plan_exposes_render_ready_validation_operations() {
    let input: syn::DeriveInput = syn::parse_quote! {
        struct Planned {
            #[koruma(full(RequiredValidation::<_>), LengthValidation::<_>::min(1))]
            name: Option<String>,
            #[koruma(each(full(ItemRequired::<_>), ItemLength::<_>::min(1)))]
            tags: Vec<Option<String>>,
        }
    };

    let plan = ValidationPlan::build(&input, "Koruma").expect("expected plan");
    let operations = plan.validation_operations();
    assert_eq!(operations.len(), 2);

    let PlannedValidationOperation::Regular(name_operation) = &operations[0] else {
        panic!("expected regular validation operation");
    };
    assert_eq!(name_operation.field.name.to_string(), "name");
    assert!(name_operation.field_validators.field_optional);
    assert_eq!(
        name_operation.field_validators.full_type_validators.len(),
        1
    );
    assert_eq!(
        name_operation.field_validators.unwrapped_validators.len(),
        1
    );
    assert!(name_operation.element_validators.is_none());

    let PlannedValidationOperation::Regular(tags_operation) = &operations[1] else {
        panic!("expected regular validation operation");
    };
    assert!(!tags_operation.field_validators.has_any());
    let element = tags_operation
        .element_validators
        .as_ref()
        .expect("expected element operation");
    assert!(!element.field_optional);
    assert!(element.element_optional);
    assert_eq!(element.full_type_validators.len(), 1);
    assert_eq!(element.unwrapped_validators.len(), 1);
}

#[test]
fn test_validation_plan_exposes_main_error_layout() {
    let input: syn::DeriveInput = syn::parse_quote! {
        #[koruma(newtype)]
        struct Planned(#[koruma(nested)] Child);
    };

    let plan = ValidationPlan::build(&input, "Koruma").expect("expected plan");
    let layout = plan.main_error_layout();
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
    let layout = plan.main_error_layout();
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
fn test_validation_plan_exposes_field_error_layout() {
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
    let layout = plan.field_error_layout();
    assert_eq!(layout.fields.len(), 6);

    assert_eq!(layout.fields[0].field.name.to_string(), "raw");
    assert_eq!(
        layout.fields[0].kind,
        PlannedFieldErrorKind::NewtypeInner {
            inner_optional: false,
            deref: true,
        }
    );

    assert_eq!(layout.fields[1].field.name.to_string(), "wrapper");
    assert_eq!(
        layout.fields[1].kind,
        PlannedFieldErrorKind::NewtypeInner {
            inner_optional: true,
            deref: false,
        }
    );

    assert_eq!(layout.fields[2].field.name.to_string(), "checked");
    assert_eq!(
        layout.fields[2].kind,
        PlannedFieldErrorKind::NewtypeWithValidators {
            inner_optional: false,
        }
    );
    assert_eq!(layout.fields[2].field_validators.len(), 1);

    assert_eq!(layout.fields[3].field.name.to_string(), "name");
    assert_eq!(
        layout.fields[3].kind,
        PlannedFieldErrorKind::Regular {
            storage: PlannedRegularFieldErrorStorage::FieldValidators,
            doc: PlannedRegularFieldErrorDoc::FieldValidators,
            all: PlannedRegularAll::FieldValidators,
            is_empty: PlannedRegularFieldErrorIsEmpty::FieldValidators,
            has_element_error: false,
        }
    );

    assert_eq!(layout.fields[4].field.name.to_string(), "tags");
    assert_eq!(
        layout.fields[4].kind,
        PlannedFieldErrorKind::Regular {
            storage: PlannedRegularFieldErrorStorage::ElementErrors,
            doc: PlannedRegularFieldErrorDoc::ElementValidators,
            all: PlannedRegularAll::None,
            is_empty: PlannedRegularFieldErrorIsEmpty::ElementErrors,
            has_element_error: true,
        }
    );
    assert_eq!(layout.fields[4].element_validators.len(), 1);

    assert_eq!(layout.fields[5].field.name.to_string(), "values");
    assert_eq!(
        layout.fields[5].kind,
        PlannedFieldErrorKind::Regular {
            storage: PlannedRegularFieldErrorStorage::FieldAndElementErrors,
            doc: PlannedRegularFieldErrorDoc::FieldAndElementValidators,
            all: PlannedRegularAll::FieldValidators,
            is_empty: PlannedRegularFieldErrorIsEmpty::FieldAndElementValidators,
            has_element_error: true,
        }
    );
    assert_eq!(layout.fields[5].field_validators.len(), 1);
    assert_eq!(layout.fields[5].element_validators.len(), 1);
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
    assert_eq!(nested.cardinality, FieldCardinality::Required);
    let nested_inner_type = &nested.inner_type;
    assert_eq!(quote!(#nested_inner_type).to_string(), "Child");
    assert!(plan.fields[0].field_validators().is_empty());
    assert!(plan.fields[0].element_validators().is_empty());
    assert!(matches!(
        plan.fields[0].error_storage,
        ErrorStorage::Nested {
            cardinality: FieldCardinality::Required
        }
    ));

    let PlannedField::Newtype(newtype) = &plan.fields[1].shape else {
        panic!("expected newtype planned field");
    };
    assert_eq!(newtype.cardinality, FieldCardinality::Optional);
    let newtype_inner_type = &newtype.inner_type;
    assert_eq!(quote!(#newtype_inner_type).to_string(), "Wrapped");
    assert_eq!(newtype.field_validators.len(), 1);
    assert!(plan.fields[1].element_validators().is_empty());
    assert!(matches!(
        plan.fields[1].error_storage,
        ErrorStorage::NewtypeWithValidators {
            cardinality: FieldCardinality::Optional
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
