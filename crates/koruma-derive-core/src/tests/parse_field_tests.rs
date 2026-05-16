//! Snapshot tests for parse_field functionality.
//!
//! Tests parsing of #[koruma(...)] attributes both directly and via #[cfg_attr(...)].

use crate::{
    FieldInfo, ParseFieldResult, ValidationInfo, ValidatorAttr, ValueFieldCapture,
    find_value_field, find_value_field_info_strict, find_value_field_strict, parse_field,
    parse_struct_options,
};
use insta::assert_debug_snapshot;
use quote::ToTokens;

#[allow(dead_code)]
#[derive(Debug)]
struct SnapshotValidator {
    name: String,
    infer_type: bool,
    explicit_type: Option<String>,
    builder_methods: Vec<(String, Vec<String>)>,
}

#[allow(dead_code)]
#[derive(Debug)]
struct SnapshotValidationInfo {
    field_validators: Vec<SnapshotValidator>,
    element_validators: Vec<SnapshotValidator>,
    is_nested: bool,
    is_newtype: bool,
}

#[allow(dead_code)]
#[derive(Debug)]
struct SnapshotFieldInfo {
    name: String,
    member: String,
    ty: String,
    validation: SnapshotValidationInfo,
}

#[allow(dead_code)]
#[derive(Debug)]
enum SnapshotParseFieldResult {
    Valid(Box<SnapshotFieldInfo>),
    Skip,
    Error(String),
}

fn normalize_tokens<T: ToTokens>(value: &T) -> String {
    value.to_token_stream().to_string()
}

fn snapshot_validator(validator: &ValidatorAttr) -> SnapshotValidator {
    SnapshotValidator {
        name: validator.name().to_string(),
        infer_type: validator.infer_type,
        explicit_type: validator.explicit_type.as_ref().map(normalize_tokens),
        builder_methods: validator
            .builder_methods
            .iter()
            .map(|method| {
                (
                    method.method.to_string(),
                    method.args.iter().map(normalize_tokens).collect(),
                )
            })
            .collect(),
    }
}

fn snapshot_validation(validation: &ValidationInfo) -> SnapshotValidationInfo {
    SnapshotValidationInfo {
        field_validators: validation
            .field_validators
            .iter()
            .map(snapshot_validator)
            .collect(),
        element_validators: validation
            .element_validators
            .iter()
            .map(snapshot_validator)
            .collect(),
        is_nested: validation.is_nested,
        is_newtype: validation.is_newtype,
    }
}

fn snapshot_field_info(info: FieldInfo) -> SnapshotFieldInfo {
    SnapshotFieldInfo {
        name: info.name.to_string(),
        member: normalize_tokens(&info.member),
        ty: normalize_tokens(&info.ty),
        validation: snapshot_validation(&info.validation),
    }
}

fn parse_field_result(field: &syn::Field) -> SnapshotParseFieldResult {
    match parse_field(field, 0) {
        ParseFieldResult::Valid(info) => {
            SnapshotParseFieldResult::Valid(Box::new(snapshot_field_info(*info)))
        },
        ParseFieldResult::Skip => SnapshotParseFieldResult::Skip,
        ParseFieldResult::Error(err) => SnapshotParseFieldResult::Error(err.to_string()),
    }
}

fn parse_struct_options_result(item: &syn::ItemStruct) -> Result<(bool, bool), String> {
    match parse_struct_options(&item.attrs) {
        Ok(options) => Ok((options.try_new, options.newtype)),
        Err(err) => Err(err.to_string()),
    }
}

fn find_value_field_name(input: &syn::ItemStruct) -> Option<String> {
    find_value_field(input).map(|(name, _)| name.to_string())
}

fn find_value_field_name_strict(input: &syn::ItemStruct) -> Result<Option<String>, String> {
    find_value_field_strict(input)
        .map(|value| value.map(|(name, _)| name.to_string()))
        .map_err(|err| err.to_string())
}

fn find_value_field_capture_strict(
    input: &syn::ItemStruct,
) -> Result<Option<ValueFieldCapture>, String> {
    find_value_field_info_strict(input)
        .map(|value| value.map(|info| info.capture))
        .map_err(|err| err.to_string())
}

fn parse_field_snapshot(field: &syn::Field) -> Option<SnapshotFieldInfo> {
    match parse_field(field, 0) {
        ParseFieldResult::Valid(info) => Some(snapshot_field_info(*info)),
        _ => None,
    }
}

// =============================================================================
// Direct #[koruma(...)] attribute tests
// =============================================================================

#[test]
fn test_parse_field_direct_single_validator() {
    let field: syn::Field = syn::parse_quote! {
        #[koruma(RangeValidation::builder().min(0).max(100))]
        pub age: i32
    };

    assert_debug_snapshot!(parse_field_snapshot(&field));
}

#[test]
fn test_parse_field_direct_multiple_validators() {
    let field: syn::Field = syn::parse_quote! {
        #[koruma(RangeValidation::builder().min(0).max(100), EvenValidation::builder())]
        pub value: i32
    };

    assert_debug_snapshot!(parse_field_snapshot(&field));
}

#[test]
fn test_parse_field_direct_generic_validator() {
    let field: syn::Field = syn::parse_quote! {
        #[koruma(GenericRange::<_>::builder().min(0.0).max(1.0))]
        pub score: f64
    };

    assert_debug_snapshot!(parse_field_snapshot(&field));
}

#[test]
fn test_parse_field_direct_each() {
    let field: syn::Field = syn::parse_quote! {
        #[koruma(each(RangeValidation::builder().min(0).max(100)))]
        pub scores: Vec<i32>
    };

    assert_debug_snapshot!(parse_field_snapshot(&field));
}

#[test]
fn test_parse_field_direct_nested() {
    let field: syn::Field = syn::parse_quote! {
        #[koruma(nested)]
        pub inner: InnerStruct
    };

    assert_debug_snapshot!(parse_field_snapshot(&field));
}

#[test]
fn test_parse_field_direct_newtype() {
    let field: syn::Field = syn::parse_quote! {
        #[koruma(newtype)]
        pub index: CommonVariableIndex
    };

    assert_debug_snapshot!(parse_field_snapshot(&field));
}

#[test]
fn test_parse_field_direct_skip() {
    let field: syn::Field = syn::parse_quote! {
        #[koruma(skip)]
        pub internal: u64
    };

    assert_debug_snapshot!(parse_field_result(&field));
}

// =============================================================================
// #[cfg_attr(..., koruma(...))] attribute tests
// =============================================================================

#[test]
fn test_parse_field_cfg_attr_single_validator() {
    let field: syn::Field = syn::parse_quote! {
        #[cfg_attr(feature = "validation", koruma(RangeValidation::builder().min(0).max(100)))]
        pub age: i32
    };

    assert_debug_snapshot!(parse_field_snapshot(&field));
}

#[test]
fn test_parse_field_cfg_attr_multiple_validators() {
    let field: syn::Field = syn::parse_quote! {
        #[cfg_attr(feature = "validation", koruma(RangeValidation::builder().min(0).max(100), EvenValidation::builder()))]
        pub value: i32
    };

    assert_debug_snapshot!(parse_field_snapshot(&field));
}

#[test]
fn test_parse_field_cfg_attr_generic_validator() {
    let field: syn::Field = syn::parse_quote! {
        #[cfg_attr(feature = "validation", koruma(GenericRange::<_>::builder().min(0.0).max(1.0)))]
        pub score: f64
    };

    assert_debug_snapshot!(parse_field_snapshot(&field));
}

#[test]
fn test_parse_field_cfg_attr_each() {
    let field: syn::Field = syn::parse_quote! {
        #[cfg_attr(feature = "validation", koruma(each(RangeValidation::builder().min(0).max(100))))]
        pub scores: Vec<i32>
    };

    assert_debug_snapshot!(parse_field_snapshot(&field));
}

#[test]
fn test_parse_field_cfg_attr_nested() {
    let field: syn::Field = syn::parse_quote! {
        #[cfg_attr(feature = "validation", koruma(nested))]
        pub inner: InnerStruct
    };

    assert_debug_snapshot!(parse_field_snapshot(&field));
}

#[test]
fn test_parse_field_cfg_attr_newtype() {
    let field: syn::Field = syn::parse_quote! {
        #[cfg_attr(feature = "validation", koruma(newtype))]
        pub index: CommonVariableIndex
    };

    assert_debug_snapshot!(parse_field_snapshot(&field));
}

#[test]
fn test_parse_field_cfg_attr_skip() {
    let field: syn::Field = syn::parse_quote! {
        #[cfg_attr(feature = "validation", koruma(skip))]
        pub internal: u64
    };

    assert_debug_snapshot!(parse_field_result(&field));
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

    assert_debug_snapshot!(parse_field_snapshot(&field));
}

#[test]
fn test_parse_field_cfg_attr_koruma_first() {
    // koruma before other attrs in same cfg_attr
    let field: syn::Field = syn::parse_quote! {
        #[cfg_attr(feature = "validation", koruma(RangeValidation::builder().min(0).max(100)), some_other_attr)]
        pub age: i32
    };

    assert_debug_snapshot!(parse_field_snapshot(&field));
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

    assert_debug_snapshot!(parse_field_snapshot(&field));
}

#[test]
fn test_parse_field_cfg_attr_any_condition() {
    // any() condition
    let field: syn::Field = syn::parse_quote! {
        #[cfg_attr(any(feature = "validation", feature = "full"), koruma(RangeValidation::builder().min(0).max(100)))]
        pub age: i32
    };

    assert_debug_snapshot!(parse_field_snapshot(&field));
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

    assert_debug_snapshot!(parse_struct_options_result(&input));
}

#[test]
fn test_parse_struct_options_cfg_attr() {
    let input: syn::ItemStruct = syn::parse_quote! {
        #[cfg_attr(feature = "validation", koruma(try_new))]
        pub struct Person {
            pub age: i32,
        }
    };

    assert_debug_snapshot!(parse_struct_options_result(&input));
}

#[test]
fn test_parse_struct_options_cfg_attr_newtype() {
    let input: syn::ItemStruct = syn::parse_quote! {
        #[cfg_attr(feature = "validation", koruma(newtype))]
        pub struct Email(String);
    };

    assert_debug_snapshot!(parse_struct_options_result(&input));
}

#[test]
fn test_parse_struct_options_cfg_attr_both() {
    let input: syn::ItemStruct = syn::parse_quote! {
        #[cfg_attr(feature = "validation", koruma(try_new, newtype))]
        pub struct Email(String);
    };

    assert_debug_snapshot!(parse_struct_options_result(&input));
}

#[test]
fn test_parse_struct_options_multiple_attrs_merge() {
    let input: syn::ItemStruct = syn::parse_quote! {
        #[koruma(try_new)]
        #[koruma(newtype)]
        pub struct Email(String);
    };

    assert_eq!(parse_struct_options_result(&input).unwrap(), (true, true));
}

#[test]
fn test_parse_struct_options_multiple_attrs_duplicate_error() {
    let input: syn::ItemStruct = syn::parse_quote! {
        #[koruma(try_new)]
        #[koruma(try_new)]
        pub struct Email(String);
    };

    let err = parse_struct_options(&input.attrs).unwrap_err();
    assert!(
        err.to_string()
            .contains("duplicate struct-level koruma option `try_new`")
    );
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
            actual: Option<i32>,
        }
    };

    assert_debug_snapshot!(find_value_field_name(&input));
}

#[test]
fn test_find_value_field_cfg_attr() {
    let input: syn::ItemStruct = syn::parse_quote! {
        pub struct Validator {
            min: i32,
            max: i32,
            #[cfg_attr(feature = "validation", koruma(value))]
            actual: Option<i32>,
        }
    };

    assert_debug_snapshot!(find_value_field_name(&input));
}

#[test]
fn test_find_value_field_cfg_attr_complex_condition() {
    let input: syn::ItemStruct = syn::parse_quote! {
        pub struct Validator {
            min: i32,
            #[cfg_attr(all(feature = "validation", not(test)), koruma(value))]
            actual: Option<i32>,
        }
    };

    assert_debug_snapshot!(find_value_field_name(&input));
}

#[test]
fn test_find_value_field_strict_rejects_duplicate_markers_on_same_field() {
    let input: syn::ItemStruct = syn::parse_quote! {
        pub struct Validator {
            min: i32,
            #[koruma(value)]
            #[doc = "other attrs stay untouched"]
            #[koruma(value)]
            actual: Option<i32>,
        }
    };

    let err = find_value_field_strict(&input).unwrap_err();
    assert!(
        err.to_string()
            .contains("field `actual` has multiple `#[koruma(value)]` markers")
    );
}

#[test]
fn test_find_value_field_strict_rejects_multiple_fields() {
    let input: syn::ItemStruct = syn::parse_quote! {
        pub struct Validator {
            #[koruma(value)]
            actual: Option<i32>,
            #[koruma(value)]
            expected: Option<i32>,
        }
    };

    let err = find_value_field_strict(&input).unwrap_err();
    assert!(
        err.to_string()
            .contains("requires exactly one `#[koruma(value)]` field")
    );
}

#[test]
fn test_find_value_field_strict_rejects_unknown_marker() {
    let input: syn::ItemStruct = syn::parse_quote! {
        pub struct Validator {
            #[koruma(other)]
            actual: Option<i32>,
        }
    };

    let err = find_value_field_strict(&input).unwrap_err();
    assert!(err.to_string().contains(
        "validator fields only support `#[koruma(value)]` and `#[koruma(skip_capture)]`"
    ));
}

#[test]
fn test_find_value_field_strict_still_returns_name() {
    let input: syn::ItemStruct = syn::parse_quote! {
        pub struct Validator {
            min: i32,
            #[koruma(value)]
            actual: Option<i32>,
        }
    };

    assert_eq!(
        find_value_field_name_strict(&input).unwrap(),
        Some("actual".to_string())
    );
}

#[test]
fn test_find_value_field_strict_supports_skip_capture() {
    let input: syn::ItemStruct = syn::parse_quote! {
        pub struct Validator {
            min: i32,
            #[koruma(value, skip_capture)]
            actual: Option<i32>,
        }
    };

    assert_eq!(
        find_value_field_capture_strict(&input).unwrap(),
        Some(ValueFieldCapture::Skip)
    );
}

#[test]
fn test_find_value_field_strict_supports_split_skip_capture_attrs() {
    let input: syn::ItemStruct = syn::parse_quote! {
        pub struct Validator {
            #[koruma(skip_capture)]
            #[koruma(value)]
            actual: Option<i32>,
        }
    };

    assert_eq!(
        find_value_field_capture_strict(&input).unwrap(),
        Some(ValueFieldCapture::Skip)
    );
}

#[test]
fn test_find_value_field_strict_rejects_duplicate_skip_capture_markers() {
    let input: syn::ItemStruct = syn::parse_quote! {
        pub struct Validator {
            #[koruma(value, skip_capture)]
            #[koruma(skip_capture)]
            actual: Option<i32>,
        }
    };

    let err = find_value_field_info_strict(&input).unwrap_err();
    assert!(
        err.to_string()
            .contains("field `actual` has multiple `#[koruma(skip_capture)]` markers")
    );
}

#[test]
fn test_find_value_field_strict_rejects_skip_capture_without_value() {
    let input: syn::ItemStruct = syn::parse_quote! {
        pub struct Validator {
            #[koruma(skip_capture)]
            actual: Option<i32>,
        }
    };

    let err = find_value_field_info_strict(&input).unwrap_err();
    assert!(
        err.to_string()
            .contains("uses `#[koruma(skip_capture)]` but is missing `#[koruma(value)]`")
    );
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

    assert_debug_snapshot!(parse_field_result(&field));
}

#[test]
fn test_parse_field_mixed_attrs_only_koruma_parsed() {
    let field: syn::Field = syn::parse_quote! {
        #[cfg_attr(feature = "serde", serde(rename = "val"))]
        #[cfg_attr(feature = "validation", koruma(RangeValidation::builder().min(0).max(100)))]
        #[doc = "Some documentation"]
        pub value: i32
    };

    assert_debug_snapshot!(parse_field_snapshot(&field));
}

#[test]
fn test_parse_field_unnamed() {
    let field: syn::Field = syn::parse_quote! {
        #[koruma(NonEmptyStringValidation::builder())]
        String
    };

    // Unnamed fields in tuple structs don't have an ident, so we rely on the index passed to parse_field
    assert_debug_snapshot!(parse_field_snapshot(&field));
}
