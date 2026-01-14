//! Snapshot tests for parse_field functionality.
//!
//! Tests parsing of #[koruma(...)] attributes both directly and via #[cfg_attr(...)].

use crate::{FieldInfo, ParseFieldResult, find_value_field, parse_field, parse_struct_options};
use insta::assert_debug_snapshot;

/// Helper to extract FieldInfo from ParseFieldResult for snapshotting.
fn parse_field_info(field: &syn::Field) -> Option<FieldInfo> {
    match parse_field(field, 0) {
        ParseFieldResult::Valid(info) => Some(*info),
        _ => None,
    }
}

// =============================================================================
// Direct #[koruma(...)] attribute tests
// =============================================================================

#[test]
fn test_parse_field_direct_single_validator() {
    let field: syn::Field = syn::parse_quote! {
        #[koruma(RangeValidation(min = 0, max = 100))]
        pub age: i32
    };

    assert_debug_snapshot!(parse_field_info(&field));
}

#[test]
fn test_parse_field_direct_multiple_validators() {
    let field: syn::Field = syn::parse_quote! {
        #[koruma(RangeValidation(min = 0, max = 100), EvenValidation)]
        pub value: i32
    };

    assert_debug_snapshot!(parse_field_info(&field));
}

#[test]
fn test_parse_field_direct_generic_validator() {
    let field: syn::Field = syn::parse_quote! {
        #[koruma(GenericRange::<_>(min = 0.0, max = 1.0))]
        pub score: f64
    };

    assert_debug_snapshot!(parse_field_info(&field));
}

#[test]
fn test_parse_field_direct_each() {
    let field: syn::Field = syn::parse_quote! {
        #[koruma(each(RangeValidation(min = 0, max = 100)))]
        pub scores: Vec<i32>
    };

    assert_debug_snapshot!(parse_field_info(&field));
}

#[test]
fn test_parse_field_direct_nested() {
    let field: syn::Field = syn::parse_quote! {
        #[koruma(nested)]
        pub inner: InnerStruct
    };

    assert_debug_snapshot!(parse_field_info(&field));
}

#[test]
fn test_parse_field_direct_newtype() {
    let field: syn::Field = syn::parse_quote! {
        #[koruma(newtype)]
        pub index: CommonVariableIndex
    };

    assert_debug_snapshot!(parse_field_info(&field));
}

#[test]
fn test_parse_field_direct_skip() {
    let field: syn::Field = syn::parse_quote! {
        #[koruma(skip)]
        pub internal: u64
    };

    assert_debug_snapshot!(parse_field(&field, 0));
}

// =============================================================================
// #[cfg_attr(..., koruma(...))] attribute tests
// =============================================================================

#[test]
fn test_parse_field_cfg_attr_single_validator() {
    let field: syn::Field = syn::parse_quote! {
        #[cfg_attr(feature = "validation", koruma(RangeValidation(min = 0, max = 100)))]
        pub age: i32
    };

    assert_debug_snapshot!(parse_field_info(&field));
}

#[test]
fn test_parse_field_cfg_attr_multiple_validators() {
    let field: syn::Field = syn::parse_quote! {
        #[cfg_attr(feature = "validation", koruma(RangeValidation(min = 0, max = 100), EvenValidation))]
        pub value: i32
    };

    assert_debug_snapshot!(parse_field_info(&field));
}

#[test]
fn test_parse_field_cfg_attr_generic_validator() {
    let field: syn::Field = syn::parse_quote! {
        #[cfg_attr(feature = "validation", koruma(GenericRange::<_>(min = 0.0, max = 1.0)))]
        pub score: f64
    };

    assert_debug_snapshot!(parse_field_info(&field));
}

#[test]
fn test_parse_field_cfg_attr_each() {
    let field: syn::Field = syn::parse_quote! {
        #[cfg_attr(feature = "validation", koruma(each(RangeValidation(min = 0, max = 100))))]
        pub scores: Vec<i32>
    };

    assert_debug_snapshot!(parse_field_info(&field));
}

#[test]
fn test_parse_field_cfg_attr_nested() {
    let field: syn::Field = syn::parse_quote! {
        #[cfg_attr(feature = "validation", koruma(nested))]
        pub inner: InnerStruct
    };

    assert_debug_snapshot!(parse_field_info(&field));
}

#[test]
fn test_parse_field_cfg_attr_newtype() {
    let field: syn::Field = syn::parse_quote! {
        #[cfg_attr(feature = "validation", koruma(newtype))]
        pub index: CommonVariableIndex
    };

    assert_debug_snapshot!(parse_field_info(&field));
}

#[test]
fn test_parse_field_cfg_attr_skip() {
    let field: syn::Field = syn::parse_quote! {
        #[cfg_attr(feature = "validation", koruma(skip))]
        pub internal: u64
    };

    assert_debug_snapshot!(parse_field(&field, 0));
}

// =============================================================================
// Mixed attributes (cfg_attr with other attributes in same cfg_attr)
// =============================================================================

#[test]
fn test_parse_field_cfg_attr_with_other_derives() {
    // koruma after other derives in same cfg_attr
    let field: syn::Field = syn::parse_quote! {
        #[cfg_attr(feature = "validation", derive(Clone), koruma(newtype))]
        pub index: CommonVariableIndex
    };

    assert_debug_snapshot!(parse_field_info(&field));
}

#[test]
fn test_parse_field_cfg_attr_koruma_first() {
    // koruma before other attrs in same cfg_attr
    let field: syn::Field = syn::parse_quote! {
        #[cfg_attr(feature = "validation", koruma(RangeValidation(min = 0, max = 100)), some_other_attr)]
        pub age: i32
    };

    assert_debug_snapshot!(parse_field_info(&field));
}

// =============================================================================
// Complex cfg_attr conditions
// =============================================================================

#[test]
fn test_parse_field_cfg_attr_complex_condition() {
    // all() condition
    let field: syn::Field = syn::parse_quote! {
        #[cfg_attr(all(feature = "validation", not(test)), koruma(newtype))]
        pub index: CommonVariableIndex
    };

    assert_debug_snapshot!(parse_field_info(&field));
}

#[test]
fn test_parse_field_cfg_attr_any_condition() {
    // any() condition
    let field: syn::Field = syn::parse_quote! {
        #[cfg_attr(any(feature = "validation", feature = "full"), koruma(RangeValidation(min = 0, max = 100)))]
        pub age: i32
    };

    assert_debug_snapshot!(parse_field_info(&field));
}

// =============================================================================
// parse_struct_options tests
// =============================================================================

#[test]
fn test_parse_struct_options_direct() {
    let input: syn::ItemStruct = syn::parse_quote! {
        #[koruma(try_new)]
        pub struct Person {
            pub age: i32,
        }
    };

    assert_debug_snapshot!(parse_struct_options(&input.attrs));
}

#[test]
fn test_parse_struct_options_cfg_attr() {
    let input: syn::ItemStruct = syn::parse_quote! {
        #[cfg_attr(feature = "validation", koruma(try_new))]
        pub struct Person {
            pub age: i32,
        }
    };

    assert_debug_snapshot!(parse_struct_options(&input.attrs));
}

#[test]
fn test_parse_struct_options_cfg_attr_newtype() {
    let input: syn::ItemStruct = syn::parse_quote! {
        #[cfg_attr(feature = "validation", koruma(newtype))]
        pub struct Email(String);
    };

    assert_debug_snapshot!(parse_struct_options(&input.attrs));
}

#[test]
fn test_parse_struct_options_cfg_attr_both() {
    let input: syn::ItemStruct = syn::parse_quote! {
        #[cfg_attr(feature = "validation", koruma(try_new, newtype))]
        pub struct Email(String);
    };

    assert_debug_snapshot!(parse_struct_options(&input.attrs));
}

// =============================================================================
// find_value_field tests
// =============================================================================

#[test]
fn test_find_value_field_direct() {
    let input: syn::ItemStruct = syn::parse_quote! {
        pub struct Validator {
            min: i32,
            max: i32,
            #[koruma(value)]
            pub actual: Option<i32>,
        }
    };

    let result = find_value_field(&input);
    assert_debug_snapshot!(result.map(|(name, _ty)| name.to_string()));
}

#[test]
fn test_find_value_field_cfg_attr() {
    let input: syn::ItemStruct = syn::parse_quote! {
        pub struct Validator {
            min: i32,
            max: i32,
            #[cfg_attr(feature = "validation", koruma(value))]
            pub actual: Option<i32>,
        }
    };

    let result = find_value_field(&input);
    assert_debug_snapshot!(result.map(|(name, _ty)| name.to_string()));
}

#[test]
fn test_find_value_field_cfg_attr_complex_condition() {
    let input: syn::ItemStruct = syn::parse_quote! {
        pub struct Validator {
            min: i32,
            #[cfg_attr(all(feature = "validation", not(test)), koruma(value))]
            pub actual: Option<i32>,
        }
    };

    let result = find_value_field(&input);
    assert_debug_snapshot!(result.map(|(name, _ty)| name.to_string()));
}

// =============================================================================
// Edge cases: non-koruma attributes should be skipped
// =============================================================================

#[test]
fn test_parse_field_non_koruma_cfg_attr_skipped() {
    let field: syn::Field = syn::parse_quote! {
        #[cfg_attr(feature = "serde", serde(rename = "something"))]
        pub name: String
    };

    assert_debug_snapshot!(parse_field(&field, 0));
}

#[test]
fn test_parse_field_mixed_attrs_only_koruma_parsed() {
    let field: syn::Field = syn::parse_quote! {
        #[cfg_attr(feature = "serde", serde(rename = "val"))]
        #[cfg_attr(feature = "validation", koruma(RangeValidation(min = 0, max = 100)))]
        #[doc = "Some documentation"]
        pub value: i32
    };

    assert_debug_snapshot!(parse_field_info(&field));
}

#[test]
fn test_parse_field_unnamed() {
    let field: syn::Field = syn::parse_quote! {
        #[koruma(NonEmptyStringValidation)]
        String
    };

    // Unnamed fields in tuple structs don't have an ident, so we rely on the index passed to parse_field
    assert_debug_snapshot!(parse_field_info(&field));
}
