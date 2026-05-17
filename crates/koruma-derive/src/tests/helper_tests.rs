//! Unit tests for helper functions in the expand module.

use crate::expand::{
    codegen::{
        helper_generics_for_usages, resolve_explicit_infer_type, validate_each_collection_type,
        validator_builder_expr, validator_field_ident, validator_variant_ident,
    },
    effective_validation_type, validator_wants_full_type,
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
    let effective = effective_validation_type(&ty, true);
    assert_eq!(quote!(#effective).to_string(), "i32");
}

#[test]
fn test_effective_validation_type_for_each_on_qualified_option_vec_uses_element_type() {
    let ty: syn::Type =
        syn::parse_quote!(core::option::Option<std::vec::Vec<core::option::Option<String>>>);
    let effective = effective_validation_type(&ty, true);
    assert_eq!(quote!(#effective).to_string(), "String");
}

#[test]
fn test_effective_validation_type_for_each_on_vec_option_unwraps_inner_option() {
    let ty: syn::Type = syn::parse_quote!(Vec<Option<String>>);
    let effective = effective_validation_type(&ty, true);
    assert_eq!(quote!(#effective).to_string(), "String");
}

#[test]
fn test_effective_validation_type_for_each_on_slice_uses_element_type() {
    let ty: syn::Type = syn::parse_quote!(&[i32]);
    let effective = effective_validation_type(&ty, true);
    assert_eq!(quote!(#effective).to_string(), "i32");
}

#[test]
fn test_effective_validation_type_for_each_on_optional_slice_option_unwraps_inner_option() {
    let ty: syn::Type = syn::parse_quote!(Option<&[Option<String>]>);
    let effective = effective_validation_type(&ty, true);
    assert_eq!(quote!(#effective).to_string(), "String");
}

#[test]
fn test_validator_wants_full_type_for_explicit_option_type() {
    let attr: ValidatorAttr = syn::parse_quote!(RequiredValidation::<Option<String>>);
    assert!(validator_wants_full_type(&attr));

    let qualified_attr: ValidatorAttr =
        syn::parse_quote!(RequiredValidation::<core::option::Option<String>>);
    assert!(validator_wants_full_type(&qualified_attr));

    let non_option_attr: ValidatorAttr = syn::parse_quote!(RequiredValidation::<String>);
    assert!(!validator_wants_full_type(&non_option_attr));
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

    let helper =
        helper_generics_for_usages(&item.generics, &[quote! { (&'a T, [u8; N], &'z str) }]);
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
fn test_each_collection_accepts_arrays_groups_and_parentheses() {
    let array_ty: syn::Type = syn::parse_quote!([i32; 3]);
    validate_each_collection_type(&array_ty).expect("arrays should support each(...)");

    let paren_ty: syn::Type = syn::parse_quote!((Vec<i32>));
    validate_each_collection_type(&paren_ty).expect("parenthesized Vec should support each(...)");

    let group_ty = syn::Type::Group(syn::TypeGroup {
        group_token: Default::default(),
        elem: Box::new(syn::parse_quote!(Vec<i32>)),
    });
    validate_each_collection_type(&group_ty).expect("grouped Vec should support each(...)");
}

#[test]
fn test_resolve_explicit_infer_type_reports_unmatched_shapes() {
    let attr: ValidatorAttr =
        syn::parse_quote!(GenericValidation::<std::collections::HashMap<_, _>>);
    let field_ty: syn::Type = syn::parse_quote!(Option<String>);

    let err = resolve_explicit_infer_type(&attr, &field_ty, false)
        .expect_err("expected unmatched explicit infer shape to fail");
    assert!(err.to_string().contains("cannot infer `_`"));
}

#[test]
fn test_codegen_names_use_hash_when_flattened_paths_collide() {
    let first: ValidatorAttr = syn::parse_quote!(foo_bar::Baz);
    let second: ValidatorAttr = syn::parse_quote!(foo::bar::Baz);
    let siblings = vec![first.clone(), second.clone()];

    let first_field = validator_field_ident(&first, &siblings).to_string();
    let second_field = validator_field_ident(&second, &siblings).to_string();
    assert!(first_field.starts_with("foo_bar_baz_"));
    assert!(second_field.starts_with("foo_bar_baz_"));
    assert_ne!(first_field, second_field);

    let first_variant = validator_variant_ident(&first, &siblings).to_string();
    let second_variant = validator_variant_ident(&second, &siblings).to_string();
    assert!(first_variant.starts_with("FooBarBazH"));
    assert!(second_variant.starts_with("FooBarBazH"));
    assert_ne!(first_variant, second_variant);
}

#[test]
fn test_validator_builder_expr_without_setters_uses_hidden_builder() {
    let attr: ValidatorAttr = syn::parse_quote!(BareValidation::<_>);
    let field_ty: syn::Type = syn::parse_quote!(Option<String>);
    let expr = validator_builder_expr(&attr, &field_ty, false, &[]);

    assert_eq!(
        quote!(#expr).to_string(),
        "BareValidation :: < String > :: __koruma_builder ()"
    );
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

    let result = find_value_field(&input);
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

    assert!(find_value_field(&input).is_none());
}

#[test]
fn test_parse_field_with_single_validator() {
    let field: syn::Field = syn::parse_quote! {
        #[koruma(RangeValidation::min(0).max(100))]
        pub age: i32
    };

    let result = parse_field(&field, 0);
    let ParseFieldResult::Valid(info) = result else {
        panic!("expected Valid result");
    };
    assert_eq!(info.name.to_string(), "age");
    assert_eq!(info.validation.field_validators.len(), 1);
    assert_eq!(
        info.validation.field_validators[0].name().to_string(),
        "RangeValidation"
    );
    assert!(!info.validation.field_validators[0].infer_type);
    assert_eq!(info.validation.field_validators[0].builder_methods.len(), 2);
    assert!(info.validation.element_validators.is_empty());
}

#[test]
fn test_parse_field_with_generic_validator() {
    let field: syn::Field = syn::parse_quote! {
        #[koruma(GenericRange::<_>::min(0.0).max(1.0))]
        pub score: f64
    };

    let result = parse_field(&field, 0);
    let ParseFieldResult::Valid(info) = result else {
        panic!("expected Valid result");
    };
    assert!(info.validation.field_validators[0].infer_type);
}

#[test]
fn test_parse_field_with_each() {
    let field: syn::Field = syn::parse_quote! {
        #[koruma(each(RangeValidation::min(0).max(100)))]
        pub scores: Vec<i32>
    };

    let result = parse_field(&field, 0);
    let ParseFieldResult::Valid(info) = result else {
        panic!("expected Valid result");
    };
    assert!(info.validation.field_validators.is_empty());
    assert_eq!(info.validation.element_validators.len(), 1);
}

#[test]
fn test_parse_field_with_skip_returns_skip() {
    let field: syn::Field = syn::parse_quote! {
        #[koruma(skip)]
        pub internal: u64
    };

    assert!(matches!(parse_field(&field, 0), ParseFieldResult::Skip));
}

#[test]
fn test_parse_field_without_koruma_returns_skip() {
    let field: syn::Field = syn::parse_quote! {
        pub normal_field: String
    };

    assert!(matches!(parse_field(&field, 0), ParseFieldResult::Skip));
}
