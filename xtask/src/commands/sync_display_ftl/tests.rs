use std::{
    collections::{BTreeMap, HashMap},
    fs,
    path::{Path, PathBuf},
};

use fluent_syntax::{ast, parser};
use proc_macro2::{LineColumn, Span};
use syn::{Block, Expr, Item, ItemImpl, Member, Stmt, Type, spanned::Spanned as _};
use tempfile::TempDir;

use crate::cli::SyncArgs;

use super::collect::{collect_display_info, collect_rs_files, collect_validator_info};
use super::ftl::{collect_ftl_templates, namespace_from_ftl_path};
use super::parse::parse_display_impl;
use super::template::{
    apply_replacements, build_write_call, escape_format_literal, line_start_offsets,
    span_to_byte_range, template_to_write_parts, write_call_changed,
};
use super::types::{DisplayInfo, FormatChunk, Replacement, TemplatePart, ValidatorInfo};
use super::{run, run_with_roots};

fn write_file(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("failed to create parent directory");
    }
    fs::write(path, content).expect("failed to write file");
}

fn fixture_validator_source() -> &'static str {
    r#"
#[fluent(namespace = "sample")]
pub struct ExampleValidation {
    pub min: usize,
    #[koruma(value)]
    pub actual: String,
}

impl std::fmt::Display for ExampleValidation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Wrong {}", self.min)
    }
}
"#
}

fn fixture_ftl_source() -> &'static str {
    r#"
example_validation = Value { $min } and { $actual }.
"#
}

fn create_sync_fixture() -> (TempDir, PathBuf, PathBuf, PathBuf) {
    let tmp = tempfile::tempdir().expect("failed to create temp directory");
    let validators_root = tmp.path().join("validators");
    let ftl_root = tmp.path().join("ftl");
    let validator_file = validators_root.join("sample.rs");
    let ftl_file = ftl_root.join("sample.ftl");
    write_file(&validator_file, fixture_validator_source());
    write_file(&ftl_file, fixture_ftl_source());
    (tmp, validators_root, ftl_root, validator_file)
}

fn compact_ws(input: &str) -> String {
    input.chars().filter(|c| !c.is_whitespace()).collect()
}

fn nth_message_pattern(resource: &ast::Resource<String>, index: usize) -> &ast::Pattern<String> {
    resource
        .body
        .iter()
        .filter_map(|entry| match entry {
            ast::Entry::Message(message) => message.value.as_ref(),
            _ => None,
        })
        .nth(index)
        .expect("expected message with value pattern")
}

include!("tests/filesystem_sync.rs");
include!("tests/collect.rs");
include!("tests/parse.rs");
include!("tests/ftl.rs");
include!("tests/template.rs");
