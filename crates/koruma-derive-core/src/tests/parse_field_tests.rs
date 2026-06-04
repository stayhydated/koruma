//! Snapshot tests for parse_field functionality.
//!
//! Tests parsing of direct #[koruma(...)] attributes.

use crate::{
    CapturePolicy, FieldInfo, ParsedDataField, ParsedFieldSpec, ParsedValidatorUse, SetterDefault,
    SetterInputPolicy, SetterPresence, StructMode, ValidatorAttr, ValidatorFieldRole,
    ValidatorLabel, parse_field, parse_struct_options, parse_validator_struct,
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
            .setter_calls()
            .iter()
            .map(|method| {
                (
                    method.method().to_string(),
                    method.args().iter().map(normalize_tokens).collect(),
                )
            })
            .collect(),
    }
}

fn snapshot_validator_use(validator_use: &ParsedValidatorUse) -> SnapshotValidator {
    snapshot_validator(validator_use.label(), validator_use.validator())
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
    }
}

fn snapshot_field_info(info: FieldInfo) -> SnapshotFieldInfo {
    SnapshotFieldInfo {
        name: info.name().to_string(),
        member: normalize_tokens(info.member()),
        ty: normalize_tokens(info.ty()),
        validation: snapshot_validation(info.validation()),
    }
}

fn parse_field_result(field: &syn::Field) -> SnapshotParsedField {
    match parse_field(field, 0) {
        Ok(ParsedDataField::Participating(info)) => {
            SnapshotParsedField::Valid(Box::new(snapshot_field_info(info)))
        },
        Ok(ParsedDataField::Skipped { .. }) => SnapshotParsedField::Skip,
        Ok(ParsedDataField::Unannotated(_)) => SnapshotParsedField::Skip,
        Err(err) => SnapshotParsedField::Error(err.to_string()),
    }
}

fn parse_struct_options_result(item: &syn::ItemStruct) -> Result<String, String> {
    match parse_struct_options(&item.attrs) {
        Ok(options) => {
            let constructor = match (
                options.constructors().try_new(),
                options.constructors().try_from(),
            ) {
                (false, false) => "None",
                (true, false) => "TryNew",
                (false, true) => "TryFrom",
                (true, true) => "TryNewAndTryFrom",
            };
            let summary = match options.mode() {
                StructMode::Regular => format!("regular::{constructor}"),
                StructMode::Newtype { .. } => format!("newtype::{constructor}"),
            };
            Ok(summary)
        },
        Err(err) => Err(err.to_string()),
    }
}

fn find_value_field_name(input: &syn::ItemStruct) -> Option<String> {
    parse_validator_struct(input)
        .ok()
        .map(|spec| spec.value_field().name().to_string())
}

fn find_value_field_name_strict(input: &syn::ItemStruct) -> Result<String, String> {
    parse_validator_struct(input)
        .map(|spec| spec.value_field().name().to_string())
        .map_err(|err| err.to_string())
}

fn find_value_field_capture_strict(input: &syn::ItemStruct) -> Result<CapturePolicy, String> {
    parse_validator_struct(input)
        .map(|spec| spec.value_spec().capture())
        .map_err(|err| err.to_string())
}

fn parse_field_snapshot(field: &syn::Field) -> Option<SnapshotFieldInfo> {
    match parse_field(field, 0) {
        Ok(ParsedDataField::Participating(info)) => Some(snapshot_field_info(info)),
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
fn test_parse_struct_options_rejects_multiple_attrs() {
    let input: syn::ItemStruct = syn::parse_quote! {
        #[koruma(try_new)]
        #[koruma(newtype)]
        pub struct Email(String);
    };

    let err = parse_struct_options(&input.attrs).unwrap_err();
    assert!(
        err.to_string()
            .contains("only one struct-level `#[koruma(...)]` attribute is allowed")
    );
}

#[test]
fn test_parse_struct_options_rejects_duplicate_option() {
    let input: syn::ItemStruct = syn::parse_quote! {
        #[koruma(try_new, try_new)]
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
    assert!(message.contains("expected `try_new`, `try_from`, or `newtype`"));
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
        #[koruma(newtype())]
        pub name: String
    };

    let err = parse_field(&field, 0).expect_err("expected data-field context error");
    assert!(
        err.to_string()
            .contains("parenthesized `newtype` is not valid")
    );
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
fn test_parse_validator_struct_rejects_multiple_attrs_on_same_field() {
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
            .contains("only one validator-field `#[koruma(...)]` attribute is allowed")
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
    assert!(message.contains("expected `value`, `skip_capture`, or `setter(...)`"));
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
fn test_parse_validator_struct_supports_skip_capture_policy() {
    let input: syn::ItemStruct = syn::parse_quote! {
        pub struct Validator {
            min: i32,
            #[koruma(skip_capture)]
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
            #[koruma(capture, value)]
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
            #[koruma(skip_capture)]
            actual: Option<String>,
        }
    };

    let spec = parse_validator_struct(&input).unwrap();
    assert_eq!(spec.value_field().name().to_string(), "actual");
    assert!(matches!(spec.value_spec().capture(), CapturePolicy::Skip));

    let ValidatorFieldRole::Setter(title) = spec.fields()[0].role() else {
        panic!("expected setter field");
    };
    assert_eq!(title.method().to_string(), "label");
    assert_eq!(title.input(), SetterInputPolicy::Into);
    assert!(matches!(title.presence(), SetterPresence::Optional));

    let ValidatorFieldRole::Setter(limit) = spec.fields()[1].role() else {
        panic!("expected setter field");
    };
    assert_eq!(limit.method().to_string(), "limit");
    assert!(matches!(limit.presence(), SetterPresence::Required));

    let ValidatorFieldRole::Setter(fallback) = spec.fields()[2].role() else {
        panic!("expected setter field");
    };
    let SetterPresence::Defaulted(SetterDefault::Expr(expr)) = fallback.presence() else {
        panic!("expected expression default");
    };
    assert_eq!(expr.as_ref().to_token_stream().to_string(), "10");
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
fn test_parse_validator_struct_rejects_duplicate_value_markers_with_skip_capture() {
    let input: syn::ItemStruct = syn::parse_quote! {
        pub struct Validator {
            #[koruma(skip_capture, value)]
            actual: Option<i32>,
        }
    };

    let err = parse_validator_struct(&input).unwrap_err();
    assert!(
        err.to_string()
            .contains("field `actual` has multiple value markers")
    );
}

#[test]
fn test_parse_validator_struct_rejects_unknown_value_option() {
    let input: syn::ItemStruct = syn::parse_quote! {
        pub struct Validator {
            #[koruma(value())]
            actual: Option<i32>,
        }
    };

    let err = parse_validator_struct(&input).unwrap_err();
    assert!(
        err.to_string()
            .contains("parenthesized `value` markers are unsupported; use `skip_capture`")
    );
}

#[test]
fn test_parse_validator_struct_rejects_parenthesized_skip_capture() {
    let input: syn::ItemStruct = syn::parse_quote! {
        pub struct Validator {
            #[koruma(skip_capture())]
            actual: Option<i32>,
        }
    };

    let err = parse_validator_struct(&input).unwrap_err();
    assert!(
        err.to_string()
            .contains("`skip_capture` is only valid as a bare validator-field")
    );
}

// =============================================================================
// Edge cases: non-koruma attributes should be skipped
// =============================================================================

#[test]
fn test_parse_field_non_koruma_attrs_skipped() {
    let field: syn::Field = syn::parse_quote! {
        #[serde(rename = "something")]
        pub name: String
    };

    assert_debug_snapshot!(parse_field_result(&field));
}

#[test]
fn test_parse_field_mixed_attrs_only_direct_koruma_parsed() {
    let field: syn::Field = syn::parse_quote! {
        #[serde(rename = "val")]
        #[koruma(RangeValidation::min(0).max(100))]
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
