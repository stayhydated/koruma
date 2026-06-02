//! Snapshot tests for parse_field functionality.
//!
//! Tests parsing of #[koruma(...)] attributes both directly and via #[cfg_attr(...)].

use crate::{
    CapturePolicy, FieldInfo, ParsedFieldSpec, ParsedValidatorUse, SetterDefault, ValidatorAttr,
    ValidatorFieldRole, ValidatorLabel, parse_field, parse_struct_options, parse_validator_struct,
};
use insta::assert_debug_snapshot;
use quote::ToTokens;

#[allow(dead_code)]
#[derive(Debug)]
struct SnapshotValidator {
    label: Option<String>,
    name: String,
    infer_type: bool,
    explicit_type: Option<String>,
    builder_methods: Vec<(String, Vec<String>)>,
}

#[allow(dead_code)]
#[derive(Debug)]
struct SnapshotValidationInfo {
    shape: String,
    field_validators: Vec<SnapshotValidator>,
    element_validators: Vec<SnapshotValidator>,
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
enum SnapshotParsedField {
    Valid(Box<SnapshotFieldInfo>),
    Skip,
    Error(String),
}

fn normalize_tokens<T: ToTokens>(value: &T) -> String {
    value.to_token_stream().to_string()
}

fn snapshot_validator(
    label: Option<&ValidatorLabel>,
    validator: &ValidatorAttr,
) -> SnapshotValidator {
    SnapshotValidator {
        label: label.map(ToString::to_string),
        name: validator.name().to_string(),
        infer_type: validator.uses_type_inference(),
        explicit_type: validator.explicit_type().map(normalize_tokens),
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

fn snapshot_validator_use(validator_use: &ParsedValidatorUse) -> SnapshotValidator {
    snapshot_validator(validator_use.label.as_ref(), &validator_use.validator)
}

fn snapshot_validation(validation: &ParsedFieldSpec) -> SnapshotValidationInfo {
    match validation {
        ParsedFieldSpec::Regular {
            field_validators,
            element_validators,
        } => SnapshotValidationInfo {
            shape: "regular".to_owned(),
            field_validators: field_validators
                .iter()
                .map(snapshot_validator_use)
                .collect(),
            element_validators: element_validators
                .iter()
                .map(snapshot_validator_use)
                .collect(),
        },
        ParsedFieldSpec::Nested { .. } => SnapshotValidationInfo {
            shape: "nested".to_owned(),
            field_validators: Vec::new(),
            element_validators: Vec::new(),
        },
        ParsedFieldSpec::Newtype {
            field_validators, ..
        } => SnapshotValidationInfo {
            shape: "newtype".to_owned(),
            field_validators: field_validators
                .iter()
                .map(snapshot_validator_use)
                .collect(),
            element_validators: Vec::new(),
        },
        ParsedFieldSpec::Skipped => SnapshotValidationInfo {
            shape: "skipped".to_owned(),
            field_validators: Vec::new(),
            element_validators: Vec::new(),
        },
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

fn parse_field_result(field: &syn::Field) -> SnapshotParsedField {
    match parse_field(field, 0) {
        Ok(Some(info)) => SnapshotParsedField::Valid(Box::new(snapshot_field_info(info))),
        Ok(None) => SnapshotParsedField::Skip,
        Err(err) => SnapshotParsedField::Error(err.to_string()),
    }
}

fn parse_struct_options_result(item: &syn::ItemStruct) -> Result<(bool, bool), String> {
    match parse_struct_options(&item.attrs) {
        Ok(options) => Ok((options.try_new(), options.is_newtype())),
        Err(err) => Err(err.to_string()),
    }
}

fn find_value_field_name(input: &syn::ItemStruct) -> Option<String> {
    parse_validator_struct(input)
        .ok()
        .map(|spec| spec.value_field().name.to_string())
}

fn find_value_field_name_strict(input: &syn::ItemStruct) -> Result<String, String> {
    parse_validator_struct(input)
        .map(|spec| spec.value_field().name.to_string())
        .map_err(|err| err.to_string())
}

fn find_value_field_capture_strict(input: &syn::ItemStruct) -> Result<CapturePolicy, String> {
    parse_validator_struct(input)
        .map(|spec| spec.value_spec().capture)
        .map_err(|err| err.to_string())
}

fn parse_field_snapshot(field: &syn::Field) -> Option<SnapshotFieldInfo> {
    match parse_field(field, 0) {
        Ok(Some(info)) => Some(snapshot_field_info(info)),
        _ => None,
    }
}

// =============================================================================
// Direct #[koruma(...)] attribute tests
// =============================================================================

#[test]
fn test_parse_field_direct_single_validator() {
    let field: syn::Field = syn::parse_quote! {
        #[koruma(RangeValidation::min(0).max(100))]
        pub age: i32
    };

    assert_debug_snapshot!(parse_field_snapshot(&field));
}

#[test]
fn test_parse_field_direct_multiple_validators() {
    let field: syn::Field = syn::parse_quote! {
        #[koruma(RangeValidation::min(0).max(100), EvenValidation)]
        pub value: i32
    };

    assert_debug_snapshot!(parse_field_snapshot(&field));
}

#[test]
fn test_parse_field_direct_generic_validator() {
    let field: syn::Field = syn::parse_quote! {
        #[koruma(GenericRange::<_>::min(0.0).max(1.0))]
        pub score: f64
    };

    assert_debug_snapshot!(parse_field_snapshot(&field));
}

#[test]
fn test_parse_field_direct_each() {
    let field: syn::Field = syn::parse_quote! {
        #[koruma(each(RangeValidation::min(0).max(100)))]
        pub scores: Vec<i32>
    };

    assert_debug_snapshot!(parse_field_snapshot(&field));
}

#[test]
fn test_parse_field_direct_labeled_validators() {
    let field: syn::Field = syn::parse_quote! {
        #[koruma(
            lower_bound = RangeValidation::min(0),
            even_check = EvenValidation,
        )]
        pub value: i32
    };

    assert_debug_snapshot!(parse_field_snapshot(&field));
}

#[test]
fn test_parse_field_rejects_non_lower_snake_validator_label() {
    let field: syn::Field = syn::parse_quote! {
        #[koruma(BadLabel = RangeValidation::min(0))]
        pub value: i32
    };

    let err = parse_field(&field, 0).expect_err("invalid labels should fail during parsing");
    assert!(
        err.to_string().contains("lower-snake"),
        "expected lower-snake label error, got: {err}"
    );
}

#[test]
fn test_parse_field_rejects_invalid_element_validator_label() {
    let field: syn::Field = syn::parse_quote! {
        #[koruma(each(bad__label = RangeValidation::min(0)))]
        pub values: Vec<i32>
    };

    let err = parse_field(&field, 0).expect_err("invalid each labels should fail during parsing");
    assert!(
        err.to_string().contains("lower-snake"),
        "expected lower-snake label error, got: {err}"
    );
}

#[test]
fn test_parse_field_direct_labeled_each() {
    let field: syn::Field = syn::parse_quote! {
        #[koruma(each(
            lower_bound = RangeValidation::min(0),
            upper_bound = RangeValidation::max(100),
        ))]
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
fn test_parse_field_direct_newtype_with_validator() {
    let field: syn::Field = syn::parse_quote! {
        #[koruma(newtype, RequiredValidation::<_>)]
        pub index: Option<CommonVariableIndex>
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
        #[cfg_attr(feature = "validation", koruma(RangeValidation::min(0).max(100)))]
        pub age: i32
    };

    assert_debug_snapshot!(parse_field_snapshot(&field));
}

#[test]
fn test_parse_field_cfg_attr_multiple_validators() {
    let field: syn::Field = syn::parse_quote! {
        #[cfg_attr(feature = "validation", koruma(RangeValidation::min(0).max(100), EvenValidation))]
        pub value: i32
    };

    assert_debug_snapshot!(parse_field_snapshot(&field));
}

#[test]
fn test_parse_field_cfg_attr_generic_validator() {
    let field: syn::Field = syn::parse_quote! {
        #[cfg_attr(feature = "validation", koruma(GenericRange::<_>::min(0.0).max(1.0)))]
        pub score: f64
    };

    assert_debug_snapshot!(parse_field_snapshot(&field));
}

#[test]
fn test_parse_field_cfg_attr_each() {
    let field: syn::Field = syn::parse_quote! {
        #[cfg_attr(feature = "validation", koruma(each(RangeValidation::min(0).max(100))))]
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
        #[cfg_attr(feature = "validation", koruma(RangeValidation::min(0).max(100)), some_other_attr)]
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
        #[cfg_attr(any(feature = "validation", feature = "full"), koruma(RangeValidation::min(0).max(100)))]
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

#[test]
fn test_parse_struct_options_rejects_data_field_marker() {
    let input: syn::ItemStruct = syn::parse_quote! {
        #[koruma(nested)]
        pub struct Person {
            pub age: i32,
        }
    };

    let err = parse_struct_options(&input.attrs).unwrap_err();
    let message = err.to_string();
    assert!(message.contains("derive struct"));
    assert!(message.contains("expected `try_new`, `newtype`, or `newtype(try_from)`"));
}

#[test]
fn test_parse_field_rejects_validator_field_marker_value() {
    let field: syn::Field = syn::parse_quote! {
        #[koruma(value)]
        pub name: String
    };

    let err = parse_field(&field, 0).expect_err("expected data-field context error");
    let message = err.to_string();
    assert!(message.contains("derive data field"));
    assert!(message.contains("expected `skip`, `nested`, `newtype`, validators, or `each(...)`"));
}

#[test]
fn test_parse_field_rejects_struct_option_try_new() {
    let field: syn::Field = syn::parse_quote! {
        #[koruma(try_new)]
        pub name: String
    };

    let err = parse_field(&field, 0).expect_err("expected data-field context error");
    let message = err.to_string();
    assert!(message.contains("`try_new` is not valid"));
    assert!(message.contains("derive data field"));
}

#[test]
fn test_parse_field_rejects_struct_newtype_options() {
    let field: syn::Field = syn::parse_quote! {
        #[koruma(newtype(try_from))]
        pub name: String
    };

    let err = parse_field(&field, 0).expect_err("expected data-field context error");
    assert!(err.to_string().contains("`newtype(...)` is not valid"));
}

#[test]
fn test_parse_field_rejects_bare_each_marker() {
    let field: syn::Field = syn::parse_quote! {
        #[koruma(each)]
        pub names: Vec<String>
    };

    let err = parse_field(&field, 0).expect_err("expected each syntax error");
    assert!(
        err.to_string()
            .contains("`each` is only valid as `each(...)`")
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
fn test_parse_validator_struct_rejects_duplicate_markers_on_same_field() {
    let input: syn::ItemStruct = syn::parse_quote! {
        pub struct Validator {
            min: i32,
            #[koruma(value)]
            #[doc = "other attrs stay untouched"]
            #[koruma(value)]
            actual: Option<i32>,
        }
    };

    let err = parse_validator_struct(&input).unwrap_err();
    assert!(
        err.to_string()
            .contains("field `actual` has multiple `#[koruma(value)]` markers")
    );
}

#[test]
fn test_parse_validator_struct_rejects_multiple_value_fields() {
    let input: syn::ItemStruct = syn::parse_quote! {
        pub struct Validator {
            #[koruma(value)]
            actual: Option<i32>,
            #[koruma(value)]
            expected: Option<i32>,
        }
    };

    let err = parse_validator_struct(&input).unwrap_err();
    assert!(
        err.to_string()
            .contains("requires exactly one `#[koruma(value)]` field")
    );
}

#[test]
fn test_parse_validator_struct_rejects_unknown_marker() {
    let input: syn::ItemStruct = syn::parse_quote! {
        pub struct Validator {
            #[koruma(other)]
            actual: Option<i32>,
        }
    };

    let err = parse_validator_struct(&input).unwrap_err();
    let message = err.to_string();
    assert!(message.contains("validator field"));
    assert!(message.contains("expected `value`, `value(capture = skip)`, or `setter(...)`"));
}

#[test]
fn test_parse_validator_struct_returns_value_name() {
    let input: syn::ItemStruct = syn::parse_quote! {
        pub struct Validator {
            min: i32,
            #[koruma(value)]
            actual: Option<i32>,
        }
    };

    assert_eq!(
        find_value_field_name_strict(&input).unwrap(),
        "actual".to_string()
    );
}

#[test]
fn test_parse_validator_struct_preserves_setter_metadata() {
    let input: syn::ItemStruct = syn::parse_quote! {
        pub struct Validator {
            #[koruma(setter(into, name = lower_bound))]
            min: i32,
            #[koruma(value)]
            actual: Option<i32>,
        }
    };

    assert_eq!(
        find_value_field_name_strict(&input).unwrap(),
        "actual".to_string()
    );
}

#[test]
fn test_parse_validator_struct_supports_capture_skip_policy() {
    let input: syn::ItemStruct = syn::parse_quote! {
        pub struct Validator {
            min: i32,
            #[koruma(value(capture = skip))]
            actual: Option<i32>,
        }
    };

    assert_eq!(
        find_value_field_capture_strict(&input).unwrap(),
        CapturePolicy::Skip
    );
}

#[test]
fn test_parse_validator_struct_rejects_unknown_validator_field_marker() {
    let input: syn::ItemStruct = syn::parse_quote! {
        pub struct Validator {
            #[koruma(capture)]
            #[koruma(value)]
            actual: Option<i32>,
        }
    };

    let err = parse_validator_struct(&input).unwrap_err();
    assert!(
        err.to_string()
            .contains("`capture` is not valid in a validator field")
    );
}

#[test]
fn test_parse_validator_struct_parses_typed_setter_metadata() {
    let input: syn::ItemStruct = syn::parse_quote! {
        pub struct Validator {
            #[koruma(setter(into, name = label))]
            title: String,
            #[koruma(setter(required))]
            limit: Option<usize>,
            #[koruma(setter(default = 10))]
            fallback: usize,
            #[koruma(value(capture = skip))]
            actual: Option<String>,
        }
    };

    let spec = parse_validator_struct(&input).unwrap();
    assert_eq!(spec.value_field().name.to_string(), "actual");
    assert!(matches!(spec.value_spec().capture, CapturePolicy::Skip));

    let ValidatorFieldRole::Setter(title) = &spec.fields[0].role else {
        panic!("expected setter field");
    };
    assert_eq!(title.method.to_string(), "label");
    assert!(title.into);
    assert!(!title.required);
    assert!(matches!(title.default, SetterDefault::None));

    let ValidatorFieldRole::Setter(limit) = &spec.fields[1].role else {
        panic!("expected setter field");
    };
    assert_eq!(limit.method.to_string(), "limit");
    assert!(limit.required);

    let ValidatorFieldRole::Setter(fallback) = &spec.fields[2].role else {
        panic!("expected setter field");
    };
    let SetterDefault::Expr(expr) = &fallback.default else {
        panic!("expected expression default");
    };
    assert_eq!(expr.to_token_stream().to_string(), "10");
}

#[test]
fn test_parse_validator_struct_rejects_value_with_setter() {
    let input: syn::ItemStruct = syn::parse_quote! {
        pub struct Validator {
            #[koruma(value, setter(required))]
            actual: Option<String>,
        }
    };

    let err = parse_validator_struct(&input).unwrap_err();
    assert!(
        err.to_string()
            .contains("fields cannot also use `#[koruma(setter(...))]`")
    );
}

#[test]
fn test_parse_validator_struct_rejects_required_default_setter() {
    let input: syn::ItemStruct = syn::parse_quote! {
        pub struct Validator {
            #[koruma(setter(required, default))]
            limit: usize,
            #[koruma(value)]
            actual: Option<String>,
        }
    };

    let err = parse_validator_struct(&input).unwrap_err();
    assert!(
        err.to_string()
            .contains("`required` and `default` cannot be combined")
    );
}

#[test]
fn test_parse_validator_struct_rejects_duplicate_value_markers_with_capture() {
    let input: syn::ItemStruct = syn::parse_quote! {
        pub struct Validator {
            #[koruma(value(capture = skip))]
            #[koruma(value)]
            actual: Option<i32>,
        }
    };

    let err = parse_validator_struct(&input).unwrap_err();
    assert!(
        err.to_string()
            .contains("field `actual` has multiple `#[koruma(value)]` markers")
    );
}

#[test]
fn test_parse_validator_struct_rejects_unknown_value_option() {
    let input: syn::ItemStruct = syn::parse_quote! {
        pub struct Validator {
            #[koruma(value(mode = skip))]
            actual: Option<i32>,
        }
    };

    let err = parse_validator_struct(&input).unwrap_err();
    assert!(
        err.to_string()
            .contains("unsupported `value(...)` option; supported option is `capture = skip`")
    );
}

#[test]
fn test_parse_validator_struct_rejects_unknown_capture_policy() {
    let input: syn::ItemStruct = syn::parse_quote! {
        pub struct Validator {
            #[koruma(value(capture = clone))]
            actual: Option<i32>,
        }
    };

    let err = parse_validator_struct(&input).unwrap_err();
    assert!(
        err.to_string()
            .contains("unsupported capture policy; supported policy is `skip`")
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
        #[cfg_attr(feature = "validation", koruma(RangeValidation::min(0).max(100)))]
        #[doc = "Some documentation"]
        pub value: i32
    };

    assert_debug_snapshot!(parse_field_snapshot(&field));
}

#[test]
fn test_parse_field_unnamed() {
    let field: syn::Field = syn::parse_quote! {
        #[koruma(NonEmptyStringValidation)]
        String
    };

    // Unnamed fields in tuple structs don't have an ident, so we rely on the index passed to parse_field
    assert_debug_snapshot!(parse_field_snapshot(&field));
}
