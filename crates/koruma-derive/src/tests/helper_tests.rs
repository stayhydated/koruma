//! Unit tests for helper functions in the expand module.

use crate::expand::{parse::*, utils::*};

use quote::quote;
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
fn test_vec_inner_type_returns_none_for_non_vec() {
    let ty: syn::Type = syn::parse_quote!(i32);
    assert!(vec_inner_type(&ty).is_none());

    let ty: syn::Type = syn::parse_quote!(Option<i32>);
    assert!(vec_inner_type(&ty).is_none());

    let ty: syn::Type = syn::parse_quote!(HashMap<String, i32>);
    assert!(vec_inner_type(&ty).is_none());
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
        #[koruma(RangeValidation(min = 0, max = 100))]
        pub age: i32
    };

    let result = parse_field(&field);
    let ParseFieldResult::Valid(info) = result else {
        panic!("expected Valid result");
    };
    assert_eq!(info.name.to_string(), "age");
    assert_eq!(info.field_validators.len(), 1);
    assert_eq!(
        info.field_validators[0].validator.to_string(),
        "RangeValidation"
    );
    assert!(!info.field_validators[0].infer_type);
    assert_eq!(info.field_validators[0].args.len(), 2);
    assert!(info.element_validators.is_empty());
}

#[test]
fn test_parse_field_with_generic_validator() {
    let field: syn::Field = syn::parse_quote! {
        #[koruma(GenericRange::<_>(min = 0.0, max = 1.0))]
        pub score: f64
    };

    let result = parse_field(&field);
    let ParseFieldResult::Valid(info) = result else {
        panic!("expected Valid result");
    };
    assert!(info.field_validators[0].infer_type);
}

#[test]
fn test_parse_field_with_each() {
    let field: syn::Field = syn::parse_quote! {
        #[koruma(each(RangeValidation(min = 0, max = 100)))]
        pub scores: Vec<i32>
    };

    let result = parse_field(&field);
    let ParseFieldResult::Valid(info) = result else {
        panic!("expected Valid result");
    };
    assert!(info.field_validators.is_empty());
    assert_eq!(info.element_validators.len(), 1);
}

#[test]
fn test_parse_field_with_skip_returns_skip() {
    let field: syn::Field = syn::parse_quote! {
        #[koruma(skip)]
        pub internal: u64
    };

    assert!(matches!(parse_field(&field), ParseFieldResult::Skip));
}

#[test]
fn test_parse_field_without_koruma_returns_skip() {
    let field: syn::Field = syn::parse_quote! {
        pub normal_field: String
    };

    assert!(matches!(parse_field(&field), ParseFieldResult::Skip));
}
