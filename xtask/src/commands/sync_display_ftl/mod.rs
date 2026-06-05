mod collect;
mod ftl;
mod parse;
mod template;
pub mod types;

use std::{
    collections::{BTreeMap, HashMap},
    fs,
    path::Path,
};

use anyhow::{Context as _, Result, bail};

use crate::cli::SyncArgs;
use collect::{collect_display_info, collect_rs_files, collect_validator_info};
use ftl::collect_ftl_templates;
use template::{
    apply_replacements, build_write_call, line_start_offsets, span_to_byte_range,
    template_to_write_parts, write_call_changed,
};
use types::{DisplayInfo, Replacement, SyncTarget, ValidatorInfo};

pub fn run(options: SyncArgs) -> Result<()> {
    let workspace_root = stayhydated_xtask::workspace_root_from_xtask_manifest()?;
    let validators_root = workspace_root.join("crates/koruma-collection/src/validators");
    let ftl_root = workspace_root.join("crates/koruma-collection/i18n/en/koruma-collection");

    run_with_roots(&validators_root, &ftl_root, options)
}

pub fn run_with_roots(validators_root: &Path, ftl_root: &Path, options: SyncArgs) -> Result<()> {
    let mut validator_files = Vec::new();
    collect_rs_files(validators_root, &mut validator_files)
        .with_context(|| format!("Failed to scan {}", validators_root.display()))?;
    validator_files.sort();

    let mut validators = BTreeMap::<String, ValidatorInfo>::new();
    let mut displays = BTreeMap::<String, DisplayInfo>::new();

    for file in &validator_files {
        let source = fs::read_to_string(file)
            .with_context(|| format!("Failed to read validator file {}", file.display()))?;
        let parsed: syn::File = syn::parse_file(&source)
            .with_context(|| format!("Failed to parse Rust AST for {}", file.display()))?;

        collect_validator_info(file, &parsed, &mut validators);
        collect_display_info(file, &parsed, &mut displays)?;
    }

    let templates = collect_ftl_templates(ftl_root)?;

    let mut missing_display = Vec::<String>::new();
    let mut missing_message = Vec::<String>::new();
    let mut targets = HashMap::<String, SyncTarget>::new();

    for validator in validators.values() {
        let Some(display) = displays.get(&validator.name) else {
            missing_display.push(format!(
                "{} ({})",
                validator.name,
                validator.source.display()
            ));
            continue;
        };

        let key = (validator.namespace.clone(), validator.message_id.clone());
        let Some(template) = templates.get(&key) else {
            missing_message.push(format!(
                "{} -> {} ({})",
                validator.name,
                validator.message_id,
                validator.source.display()
            ));
            continue;
        };

        targets.insert(
            validator.name.clone(),
            SyncTarget {
                validator: validator.clone(),
                display: display.clone(),
                template: template.clone(),
            },
        );
    }

    if !missing_display.is_empty() {
        eprintln!(
            "warning: missing std::fmt::Display implementations for {} validator(s):",
            missing_display.len()
        );
        for item in &missing_display {
            eprintln!("  - {item}");
        }
    }

    if !missing_message.is_empty() {
        eprintln!(
            "warning: missing EN FTL messages for {} validator(s):",
            missing_message.len()
        );
        for item in &missing_message {
            eprintln!("  - {item}");
        }
    }

    let mut updated_impls = 0usize;
    let mut changed_files = 0usize;

    for file in &validator_files {
        let source = fs::read_to_string(file)
            .with_context(|| format!("Failed to read validator file {}", file.display()))?;
        let line_starts = line_start_offsets(&source);

        let mut replacements = Vec::<Replacement>::new();

        for (type_name, target) in targets
            .iter()
            .filter(|(_, target)| target.display.source == *file)
        {
            let (format_literal, args) =
                template_to_write_parts(&target.template, &target.validator, &target.display)
                    .with_context(|| {
                        format!(
                            "Failed to convert FTL template for {} from {}",
                            type_name,
                            target.display.source.display()
                        )
                    })?;

            let write_call = build_write_call(&format_literal, &args);
            let span_context = format!(
                "Failed to map write! span for {} in {}",
                type_name,
                file.display()
            );
            let (start, end) = span_to_byte_range(target.display.write_span, &line_starts, &source)
                .context(span_context)?;

            if write_call_changed(&source[start..end], &write_call) {
                replacements.push(Replacement {
                    start,
                    end,
                    replacement: write_call,
                    type_name: type_name.clone(),
                });
            }
        }

        if !replacements.is_empty() {
            updated_impls += replacements.len();
            changed_files += 1;

            if !options.check {
                let rendered = apply_replacements(&source, &replacements);
                fs::write(file, rendered)
                    .with_context(|| format!("Failed to write {}", file.display()))?;
            }

            if options.verbose {
                for replacement in &replacements {
                    println!("updated {} ({})", replacement.type_name, file.display());
                }
            }
        }
    }

    if options.check {
        if updated_impls == 0 {
            println!("sync-display-ftl: no changes needed.");
            return Ok(());
        }

        bail!(
            "sync-display-ftl: {updated_impls} Display impl(s) would be updated across {changed_files} file(s).",
        );
    }

    println!(
        "sync-display-ftl: updated {updated_impls} Display impl(s) across {changed_files} file(s).",
    );
    Ok(())
}

#[cfg(test)]
mod tests {
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

    fn nth_message_pattern<'a>(
        resource: &'a ast::Resource<String>,
        index: usize,
    ) -> &'a ast::Pattern<String> {
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

    #[test]
    fn sync_args_into_sync_options() {
        let options: SyncArgs = SyncArgs {
            check: true,
            verbose: true,
        };
        assert!(options.check);
        assert!(options.verbose);
    }

    #[test]
    fn sync_display_ftl_check_mode_reports_pending_changes() {
        let (_tmp, validators_root, ftl_root, _validator_file) = create_sync_fixture();

        let err = run_with_roots(
            &validators_root,
            &ftl_root,
            SyncArgs {
                check: true,
                verbose: false,
            },
        )
        .expect_err("expected pending changes in check mode");

        assert!(err.to_string().contains("would be updated"));
    }

    #[test]
    fn sync_display_ftl_applies_changes_and_then_reports_clean() {
        let (_tmp, validators_root, ftl_root, validator_file) = create_sync_fixture();

        run_with_roots(
            &validators_root,
            &ftl_root,
            SyncArgs {
                check: false,
                verbose: true,
            },
        )
        .expect("sync should apply changes");

        let updated = fs::read_to_string(&validator_file).expect("failed to read updated file");
        assert!(compact_ws(&updated).contains("write!(f,\"Value{}and{}.\",self.min,self.actual)"));

        run_with_roots(
            &validators_root,
            &ftl_root,
            SyncArgs {
                check: true,
                verbose: false,
            },
        )
        .expect("check mode should pass after sync");
    }

    #[test]
    fn collect_rs_files_walks_directories() {
        let tmp = tempfile::tempdir().expect("failed to create temp directory");
        write_file(&tmp.path().join("root.rs"), "fn root() {}");
        write_file(&tmp.path().join("nested/inner.rs"), "fn inner() {}");
        write_file(&tmp.path().join("nested/ignore.txt"), "ignore");

        let mut files = Vec::new();
        collect_rs_files(tmp.path(), &mut files).expect("collection should succeed");
        files.sort();

        assert_eq!(files.len(), 2);
        assert!(files.iter().any(|path| path.ends_with("root.rs")));
        assert!(files.iter().any(|path| path.ends_with("inner.rs")));
    }

    #[test]
    fn collect_validator_info_filters_and_extracts_namespace() {
        let source = r#"
#[fluent(namespace = "demo")]
pub struct IncludedValidation {
    pub min: usize,
    pub actual: String,
}

pub struct NoNamespaceValidation {
    pub actual: String,
}

#[fluent(namespace = "demo")]
pub struct IgnoredType {
    pub actual: String,
}
"#;

        let parsed = syn::parse_file(source).expect("valid rust");
        let mut validators = BTreeMap::new();
        collect_validator_info(Path::new("demo.rs"), &parsed, &mut validators);

        assert_eq!(validators.len(), 1);
        let info = validators
            .get("IncludedValidation")
            .expect("included validator should exist");
        assert_eq!(info.namespace, "demo");
        assert_eq!(info.message_id, "included_validation");
        assert!(info.fields.contains("min"));
        assert!(info.fields.contains("actual"));
    }

    #[test]
    fn collect_display_info_reads_display_impls() {
        let source = r#"
pub struct IncludedValidation;

impl std::fmt::Display for IncludedValidation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Value {1} and {0}", self.actual, self.min)
    }
}
"#;
        let parsed = syn::parse_file(source).expect("valid rust");
        let mut displays = BTreeMap::new();
        collect_display_info(Path::new("demo.rs"), &parsed, &mut displays).expect("parse display");

        let info = displays
            .get("IncludedValidation")
            .expect("display info should exist");
        assert_eq!(
            info.expr_by_placeholder
                .get("actual")
                .map(|value| compact_ws(value)),
            Some("self.actual".to_string())
        );
        assert_eq!(
            info.expr_by_placeholder
                .get("min")
                .map(|value| compact_ws(value)),
            Some("self.min".to_string())
        );
    }

    #[test]
    fn extract_namespace_supports_fluent_and_cfg_attr() {
        use super::collect::extract_namespace;

        let direct: syn::ItemStruct = syn::parse_quote! {
            #[fluent(namespace = "string")]
            struct Direct;
        };
        assert_eq!(extract_namespace(&direct.attrs), Some("string".to_string()));

        let via_cfg_attr: syn::ItemStruct = syn::parse_quote! {
            #[cfg_attr(feature = "fluent", fluent(namespace = "numeric"))]
            struct CfgAttr;
        };
        assert_eq!(
            extract_namespace(&via_cfg_attr.attrs),
            Some("numeric".to_string())
        );

        let none: syn::ItemStruct = syn::parse_quote! {
            struct Plain;
        };
        assert_eq!(extract_namespace(&none.attrs), None);
    }

    #[test]
    fn parse_display_impl_handles_success_none_and_error_cases() {
        let success_impl: ItemImpl = syn::parse_quote! {
            impl std::fmt::Display for IncludedValidation {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    write!(f, "Value {1} and {0}", self.actual, self.min)
                }
            }
        };
        let (type_name, map, _span) = parse_display_impl(&success_impl)
            .expect("parse should succeed")
            .expect("display impl should be extracted");
        assert_eq!(type_name, "IncludedValidation");
        assert_eq!(
            map.get("actual").map(|value| compact_ws(value)),
            Some("self.actual".to_string())
        );
        assert_eq!(
            map.get("min").map(|value| compact_ws(value)),
            Some("self.min".to_string())
        );

        let local_var_impl: ItemImpl = syn::parse_quote! {
            impl std::fmt::Display for RangeValidation {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    let left_delimiter = if self.exclusive_min { "(" } else { "[" };
                    write!(f, "Value {} {}", left_delimiter, self.min)
                }
            }
        };
        let (_, map, _span) = parse_display_impl(&local_var_impl)
            .expect("parse should succeed")
            .expect("display impl should be extracted");
        assert_eq!(
            map.get("left_delimiter").map(|value| compact_ws(value)),
            Some("left_delimiter".to_string())
        );
        assert_eq!(
            map.get("min").map(|value| compact_ws(value)),
            Some("self.min".to_string())
        );

        let non_display_impl: ItemImpl = syn::parse_quote! {
            impl IncludedValidation {
                fn fmt(&self) {}
            }
        };
        assert!(
            parse_display_impl(&non_display_impl)
                .expect("parse should succeed")
                .is_none()
        );

        let invalid_slot_impl: ItemImpl = syn::parse_quote! {
            impl std::fmt::Display for BrokenValidation {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    write!(f, "Value {name}", self.actual)
                }
            }
        };
        let err = parse_display_impl(&invalid_slot_impl).expect_err("invalid slot");
        assert!(err.to_string().contains("unrecognized format slot"));
    }

    #[test]
    fn find_write_macro_helpers_cover_statement_and_expression_variants() {
        use super::parse::{
            find_write_macro_in_block, find_write_macro_in_expr, find_write_macro_in_stmt,
        };

        let block: Block = syn::parse_quote!({
            let _x = write!(f, "x");
            println!("ignore");
        });
        assert!(find_write_macro_in_block(&block).is_some());

        let item_stmt: Stmt = syn::parse_quote! { fn helper() {} };
        assert!(find_write_macro_in_stmt(&item_stmt).is_none());

        let macro_stmt: Stmt = syn::parse_quote! { println!("x"); };
        assert!(find_write_macro_in_stmt(&macro_stmt).is_none());

        let expr_try: Expr = syn::parse_quote! { (write!(f, "x"))? };
        assert!(find_write_macro_in_expr(&expr_try).is_some());

        let expr_return: Expr = syn::parse_quote! { return write!(f, "x") };
        assert!(find_write_macro_in_expr(&expr_return).is_some());
    }

    #[test]
    fn format_and_slot_parsing_helpers_cover_happy_and_error_paths() {
        use super::parse::{format_to_expr_map, parse_slot_index};

        let args: Vec<Expr> = vec![
            syn::parse_quote!(self.actual),
            syn::parse_quote!(self.min),
            syn::parse_quote!(custom_expr()),
        ];
        let map = format_to_expr_map("{} {1} {}", &args).expect("format map should parse");
        assert_eq!(
            map.get("actual").map(|value| compact_ws(value)),
            Some("self.actual".to_string())
        );
        assert_eq!(
            map.get("min").map(|value| compact_ws(value)),
            Some("self.min".to_string())
        );

        let err = format_to_expr_map("{9}", &args).expect_err("out-of-range slot should fail");
        assert!(err.to_string().contains("references argument #9"));

        assert_eq!(parse_slot_index("", 3).expect("empty spec"), (3, false));
        assert_eq!(
            parse_slot_index(":x", 4).expect("format-only spec"),
            (4, false)
        );
        assert_eq!(parse_slot_index("2", 0).expect("explicit index"), (2, true));
        assert_eq!(
            parse_slot_index("1:?", 0).expect("explicit index with format"),
            (1, true)
        );
        assert!(parse_slot_index("name", 0).is_err());
        assert!(parse_slot_index("999999999999999999999999999999", 0).is_err());
    }

    #[test]
    fn parse_format_chunks_handles_escapes_and_errors() {
        use super::parse::parse_format_chunks;

        let chunks = parse_format_chunks("A {{brace}} and {0}").expect("valid format string");
        assert_eq!(
            chunks,
            vec![
                FormatChunk::Text("A {brace} and ".to_string()),
                FormatChunk::Slot("0".to_string()),
            ]
        );

        let unmatched = parse_format_chunks("bad }").expect_err("unmatched brace should fail");
        assert!(unmatched.to_string().contains("Unmatched"));

        let unclosed = parse_format_chunks("bad {0").expect_err("unclosed brace should fail");
        assert!(unclosed.to_string().contains("Unclosed"));
    }

    #[test]
    fn infer_variable_name_variants() {
        use super::parse::{infer_variable_name, is_self_expr, member_name};

        let field_expr: Expr = syn::parse_quote!(self.actual);
        assert_eq!(infer_variable_name(&field_expr), Some("actual".to_string()));

        let nested_expr: Expr = syn::parse_quote!((self.inner).value);
        assert_eq!(
            infer_variable_name(&nested_expr),
            Some("inner_value".to_string())
        );

        let method_expr: Expr = syn::parse_quote!((&self.actual).len());
        assert_eq!(
            infer_variable_name(&method_expr),
            Some("actual_len".to_string())
        );

        let unary_expr: Expr = syn::parse_quote!(-self.count);
        assert_eq!(infer_variable_name(&unary_expr), Some("count".to_string()));

        let plain_expr: Expr = syn::parse_quote!(some_value);
        assert_eq!(
            infer_variable_name(&plain_expr),
            Some("some_value".to_string())
        );

        let qualified_expr: Expr = syn::parse_quote!(crate::some_value);
        assert_eq!(infer_variable_name(&qualified_expr), None);

        let named_member: Member = syn::parse_quote!(field_name);
        assert_eq!(member_name(&named_member), Some("field_name".to_string()));
        let unnamed_member = Member::Unnamed(syn::Index::from(2));
        assert_eq!(member_name(&unnamed_member), Some("2".to_string()));

        let self_expr: Expr = syn::parse_quote!(self);
        let not_self_expr: Expr = syn::parse_quote!(other);
        assert!(is_self_expr(&self_expr));
        assert!(!is_self_expr(&not_self_expr));
    }

    #[test]
    fn template_conversion_covers_resolver_paths() {
        let validator = ValidatorInfo {
            name: "DemoValidation".to_string(),
            namespace: "demo".to_string(),
            message_id: "demo_validation".to_string(),
            source: PathBuf::from("demo.rs"),
            fields: ["actual", "kind", "other"]
                .into_iter()
                .map(str::to_string)
                .collect(),
        };
        let display = DisplayInfo {
            expr_by_placeholder: std::iter::once(("kind".to_string(), "self.kind".to_string()))
                .collect(),
            source: PathBuf::from("demo.rs"),
            write_span: Span::call_site(),
        };
        let template = vec![
            TemplatePart::Text("A ".to_string()),
            TemplatePart::Placeholder("kind".to_string()),
            TemplatePart::Text(" ".to_string()),
            TemplatePart::Placeholder("other".to_string()),
            TemplatePart::Text(" ".to_string()),
            TemplatePart::Placeholder("actual".to_string()),
        ];

        let (format_literal, args) =
            template_to_write_parts(&template, &validator, &display).expect("template conversion");
        assert_eq!(format_literal, "A {} {} {}");
        assert_eq!(
            args,
            vec![
                "self.kind".to_string(),
                "self.other".to_string(),
                "self.actual".to_string()
            ]
        );

        let unresolved = template_to_write_parts(
            &[TemplatePart::Placeholder("missing".to_string())],
            &validator,
            &display,
        )
        .expect_err("missing placeholder should fail");
        assert!(
            unresolved
                .to_string()
                .contains("Cannot resolve placeholder '$missing'")
        );
    }

    #[test]
    fn template_and_expression_helpers_cover_remaining_branches() {
        use super::ftl::{push_text_part, root_variable_for_expression, template_from_pattern};

        let ftl = r#"
-term = skip
simple = Value { $actual }.
unrecognized = { "literal" }
"#;
        let resource = parser::parse(ftl.to_string()).expect("valid ftl");

        let simple_pattern = nth_message_pattern(&resource, 0);
        let simple_template = template_from_pattern(simple_pattern).expect("simple template");
        assert_eq!(
            simple_template,
            vec![
                TemplatePart::Text("Value ".to_string()),
                TemplatePart::Placeholder("actual".to_string()),
                TemplatePart::Text(".".to_string()),
            ]
        );

        let unrecognized_pattern = nth_message_pattern(&resource, 1);
        assert!(template_from_pattern(unrecognized_pattern).is_err());

        let inline_expr: ast::Expression<String> =
            ast::Expression::Inline(ast::InlineExpression::VariableReference {
                id: ast::Identifier {
                    name: "actual".to_string(),
                },
            });
        assert_eq!(
            root_variable_for_expression(&inline_expr),
            Some("actual".to_string())
        );

        let no_var_expr: ast::Expression<String> =
            ast::Expression::Inline(ast::InlineExpression::StringLiteral {
                value: "x".to_string(),
            });
        assert_eq!(root_variable_for_expression(&no_var_expr), None);

        let mut template = Vec::new();
        push_text_part(&mut template, String::new());
        push_text_part(&mut template, "A".to_string());
        push_text_part(&mut template, "B".to_string());
        assert_eq!(template, vec![TemplatePart::Text("AB".to_string())]);
    }

    #[test]
    fn string_and_range_helpers_cover_error_paths() {
        use super::template::line_col_to_offset;

        assert_eq!(
            escape_format_literal("\\\"\n\r\t{}"),
            "\\\\\\\"\\n\\r\\t{{}}"
        );
        assert_eq!(
            build_write_call("Value {}", &["self.actual".to_string()]),
            "write!(f, \"Value {}\", self.actual)"
        );

        let source = "ab\ncd\nef";
        let starts = line_start_offsets(source);
        assert_eq!(starts, vec![0, 3, 6]);
        assert_eq!(
            line_col_to_offset(LineColumn { line: 2, column: 1 }, &starts).expect("valid line/col"),
            4
        );
        assert!(line_col_to_offset(LineColumn { line: 0, column: 0 }, &starts).is_err());
        assert!(line_col_to_offset(LineColumn { line: 9, column: 0 }, &starts).is_err());

        let source_with_span = "struct Marker;\nfn demo() {\n    write!(f, \"message\");\n}\n";
        let parsed_file: syn::File = syn::parse_file(source_with_span).expect("valid rust file");
        let stmt_span = parsed_file
            .items
            .iter()
            .find_map(|item| match item {
                Item::Fn(item_fn) => Some(item_fn.block.stmts[0].span()),
                _ => None,
            })
            .expect("expected function item");
        let valid_range = span_to_byte_range(
            stmt_span,
            &line_start_offsets(source_with_span),
            source_with_span,
        )
        .expect("valid span");
        assert!(valid_range.1 > valid_range.0);

        let invalid_range =
            span_to_byte_range(stmt_span, &[0], "x").expect_err("invalid byte range");
        let message = invalid_range.to_string();
        assert!(
            message.contains("Invalid byte range") || message.contains("out of bounds"),
            "unexpected span error: {message}"
        );
    }

    #[test]
    fn replacement_and_change_detection_helpers() {
        let source = "abc123xyz";
        let replaced = apply_replacements(
            source,
            &[
                Replacement {
                    start: 3,
                    end: 6,
                    replacement: "###".to_string(),
                    type_name: "middle".to_string(),
                },
                Replacement {
                    start: 0,
                    end: 3,
                    replacement: "ABC".to_string(),
                    type_name: "start".to_string(),
                },
            ],
        );
        assert_eq!(replaced, "ABC###xyz");

        assert!(!write_call_changed(
            "write!(f, \"x\", value)",
            "write!(f,\"x\",value)"
        ));
        assert!(write_call_changed(
            "write!(f, \"x\", value)",
            "write!(f, \"y\", value)"
        ));
        assert!(!write_call_changed("not-an-expr", "not-an-expr"));
        assert!(write_call_changed("not-an-expr", "different"));
    }

    #[test]
    fn extracts_placeholder_from_select_expression() {
        use super::ftl::template_from_pattern;

        let ftl = r#"
ip_validation =
    Not a valid { $kind ->
        [v4] IPv4
       *[other] IP
    } address.
"#;
        let resource = parser::parse(ftl.to_string()).expect("valid ftl");
        let pattern = nth_message_pattern(&resource, 0);

        let template = template_from_pattern(pattern).expect("template conversion works");
        assert_eq!(
            template,
            vec![
                TemplatePart::Text("Not a valid ".to_string()),
                TemplatePart::Placeholder("kind".to_string()),
                TemplatePart::Text(" address.".to_string()),
            ]
        );
    }

    #[test]
    fn workspace_wrapper_and_root_paths_are_reachable() {
        let root = stayhydated_xtask::workspace_root_from_xtask_manifest().unwrap();
        assert!(root.ends_with("koruma"));
        let _ = run(SyncArgs {
            check: true,
            verbose: false,
        });
    }

    #[test]
    fn sync_display_ftl_warns_for_missing_display_and_missing_message() {
        let tmp = tempfile::tempdir().expect("failed to create temp directory");
        let validators_root = tmp.path().join("validators");
        let ftl_root = tmp.path().join("ftl");
        let validator_file = validators_root.join("sample.rs");
        let ftl_file = ftl_root.join("sample.ftl");
        write_file(
            &validator_file,
            r#"
#[fluent(namespace = "sample")]
pub struct MissingDisplayValidation {
    #[koruma(value)]
    pub actual: String,
}

#[fluent(namespace = "sample")]
pub struct MissingMessageValidation {
    #[koruma(value)]
    pub actual: String,
}

impl std::fmt::Display for MissingMessageValidation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Bad {}", self.actual)
    }
}
"#,
        );
        write_file(&ftl_file, "another_validation = Value { $actual }.");

        run_with_roots(
            &validators_root,
            &ftl_root,
            SyncArgs {
                check: false,
                verbose: false,
            },
        )
        .expect("sync should tolerate missing display/message with warnings");
    }

    #[test]
    fn sync_display_ftl_surfaces_template_conversion_context() {
        let tmp = tempfile::tempdir().expect("failed to create temp directory");
        let validators_root = tmp.path().join("validators");
        let ftl_root = tmp.path().join("ftl");
        let validator_file = validators_root.join("sample.rs");
        let ftl_file = ftl_root.join("sample.ftl");
        write_file(&validator_file, fixture_validator_source());
        write_file(&ftl_file, "example_validation = Unknown { $missing }.");

        let err = run_with_roots(
            &validators_root,
            &ftl_root,
            SyncArgs {
                check: false,
                verbose: false,
            },
        )
        .expect_err("placeholder resolution should fail");
        assert!(
            err.to_string()
                .contains("Failed to convert FTL template for ExampleValidation")
        );
    }

    #[test]
    fn collect_validator_and_display_info_cover_additional_paths() {
        let source = r#"
#[fluent(namespace = "demo")]
pub struct TupleValidation(i32);

#[fluent(namespace = "demo")]
pub struct DisplayedValidation {
    pub actual: i32,
}

impl DisplayedValidation {
    fn helper(&self) {}
}

impl std::fmt::Display for DisplayedValidation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.actual)
    }
}
"#;
        let parsed = syn::parse_file(source).expect("valid rust");

        let mut validators = BTreeMap::new();
        collect_validator_info(Path::new("demo.rs"), &parsed, &mut validators);
        let tuple = validators
            .get("TupleValidation")
            .expect("tuple validator should be collected");
        assert!(tuple.fields.is_empty());

        let mut displays = BTreeMap::new();
        collect_display_info(Path::new("demo.rs"), &parsed, &mut displays).expect("display parse");
        assert!(displays.contains_key("DisplayedValidation"));
    }

    #[test]
    fn collect_ftl_templates_covers_additional_branches() {
        let parse_err_tmp = tempfile::tempdir().expect("failed to create temp directory");
        write_file(
            &parse_err_tmp.path().join("broken.ftl"),
            "broken = { $value",
        );
        let parse_err = collect_ftl_templates(parse_err_tmp.path()).expect_err("invalid ftl");
        assert!(parse_err.to_string().contains("Failed to parse FTL AST"));

        let unrecognized_tmp = tempfile::tempdir().expect("failed to create temp directory");
        write_file(
            &unrecognized_tmp.path().join("sample.ftl"),
            r#"
-term = keep
no_value =
    .attr = still ignored
unrecognized = { "literal" }
"#,
        );
        let unrecognized =
            collect_ftl_templates(unrecognized_tmp.path()).expect_err("unrecognized pattern");
        assert!(
            unrecognized
                .to_string()
                .contains("Unrecognized message pattern")
        );

        let skip_tmp = tempfile::tempdir().expect("failed to create temp directory");
        write_file(&skip_tmp.path().join("ignore.txt"), "ignore");
        write_file(
            &skip_tmp.path().join("sample.ftl"),
            "sample_validation = ok",
        );
        let skipped = collect_ftl_templates(skip_tmp.path()).expect("skip paths should succeed");
        assert_eq!(skipped.len(), 1);
    }

    #[test]
    fn namespace_from_ftl_path_filters_non_ftl_paths() {
        assert_eq!(
            namespace_from_ftl_path(Path::new("sample.ftl")).expect("UTF-8 stem should parse"),
            Some("sample".to_string())
        );
        assert_eq!(
            namespace_from_ftl_path(Path::new("ignore.txt")).expect("non-FTL path should skip"),
            None
        );
    }

    #[test]
    fn extract_namespace_helpers_cover_none_paths() {
        use super::collect::{extract_namespace, extract_namespace_from_fluent_meta};

        let from_cfg_attr: syn::ItemStruct = syn::parse_quote! {
            #[doc = "x"]
            #[cfg_attr(feature = "fluent", derive(Clone), fluent(namespace = "cfg-space"))]
            struct CfgNamespace;
        };
        assert_eq!(
            extract_namespace(&from_cfg_attr.attrs),
            Some("cfg-space".to_string())
        );

        let meta_list: syn::MetaList = syn::parse_quote! {
            fluent(flag, module = "x", namespace = 1)
        };
        assert_eq!(extract_namespace_from_fluent_meta(&meta_list), None);

        let simple_cfg_attr: syn::ItemStruct = syn::parse_quote! {
            #[cfg_attr(test, fluent(namespace = "simple-cfg"))]
            struct SimpleCfg;
        };
        assert_eq!(
            extract_namespace(&simple_cfg_attr.attrs),
            Some("simple-cfg".to_string())
        );
    }

    #[test]
    fn parse_display_impl_and_type_helpers_cover_remaining_paths() {
        use super::parse::extract_type_ident;

        let no_fmt: ItemImpl = syn::parse_quote! {
            impl std::fmt::Display for NoFmtValidation {
                fn not_fmt(&self) {}
            }
        };
        assert!(
            parse_display_impl(&no_fmt)
                .expect("parse should succeed")
                .is_none()
        );

        let fmt_without_write: ItemImpl = syn::parse_quote! {
            impl std::fmt::Display for NoWriteValidation {
                fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    Ok(())
                }
            }
        };
        assert!(
            parse_display_impl(&fmt_without_write)
                .expect("parse should succeed")
                .is_none()
        );

        let too_few_args: ItemImpl = syn::parse_quote! {
            impl std::fmt::Display for TooFewArgsValidation {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    write!(f)
                }
            }
        };
        assert!(
            parse_display_impl(&too_few_args)
                .expect("parse should succeed")
                .is_none()
        );

        let non_string_format: ItemImpl = syn::parse_quote! {
            impl std::fmt::Display for NonStringFormatValidation {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    let fmt_lit = "{}";
                    write!(f, fmt_lit, self.value)
                }
            }
        };
        assert!(
            parse_display_impl(&non_string_format)
                .expect("parse should succeed")
                .is_none()
        );

        let ref_ty: Type = syn::parse_quote!(&ExampleValidation);
        assert_eq!(
            extract_type_ident(&ref_ty),
            Some("ExampleValidation".to_string())
        );
        let paren_ty: Type = syn::parse_quote!((ExampleValidation));
        assert_eq!(
            extract_type_ident(&paren_ty),
            Some("ExampleValidation".to_string())
        );
        let tuple_ty: Type = syn::parse_quote!((i32, i32));
        assert_eq!(extract_type_ident(&tuple_ty), None);
    }

    #[test]
    fn write_macro_search_and_change_fallback_cover_remaining_paths() {
        use super::parse::{
            find_write_macro_in_block, find_write_macro_in_expr, find_write_macro_in_stmt,
        };

        let only_write_macro_stmt: Stmt = syn::parse_quote! { write!(f, "x"); };
        assert!(find_write_macro_in_stmt(&only_write_macro_stmt).is_some());

        let no_write_block: Block = syn::parse_quote!({
            println!("not write");
            let _ = 1;
        });
        assert!(find_write_macro_in_block(&no_write_block).is_none());

        let non_write_expr: Expr = syn::parse_quote!(println!("x"));
        assert!(find_write_macro_in_expr(&non_write_expr).is_none());

        let block_expr: Expr = syn::parse_quote!({ write!(f, "x") });
        assert!(find_write_macro_in_expr(&block_expr).is_some());

        let grouped_expr = Expr::Group(syn::ExprGroup {
            attrs: Vec::new(),
            group_token: syn::token::Group::default(),
            expr: Box::new(syn::parse_quote!(write!(f, "x"))),
        });
        assert!(find_write_macro_in_expr(&grouped_expr).is_some());

        assert!(!write_call_changed("{", "{"));
        assert!(write_call_changed("{", "}"));
    }

    #[test]
    fn span_to_byte_range_invalid_range_branch_is_covered() {
        let one_line_source = "struct Marker;fn demo(){write!(f, \"m\");}\n";
        let parsed_file: syn::File = syn::parse_file(one_line_source).expect("valid rust");
        let stmt_span = parsed_file
            .items
            .iter()
            .find_map(|item| match item {
                Item::Fn(item_fn) => Some(item_fn.block.stmts[0].span()),
                _ => None,
            })
            .expect("expected function item");

        let err = span_to_byte_range(stmt_span, &[0], "x").expect_err("invalid range expected");
        assert!(err.to_string().contains("Invalid byte range"));
    }

    #[test]
    fn resolves_actual_from_struct_fields() {
        let validator = ValidatorInfo {
            name: "ExampleValidation".to_string(),
            namespace: "example".to_string(),
            message_id: "example_validation".to_string(),
            source: PathBuf::from("example.rs"),
            fields: std::iter::once("actual".to_string()).collect(),
        };

        let display = DisplayInfo {
            expr_by_placeholder: HashMap::new(),
            source: PathBuf::from("example.rs"),
            write_span: Span::call_site(),
        };

        let template = vec![
            TemplatePart::Text("Value was ".to_string()),
            TemplatePart::Placeholder("actual".to_string()),
            TemplatePart::Text(".".to_string()),
        ];

        let (format_literal, args) =
            template_to_write_parts(&template, &validator, &display).expect("conversion works");

        assert_eq!(format_literal, "Value was {}.");
        assert_eq!(args, vec!["self.actual".to_string()]);
    }

    #[test]
    fn escapes_rust_format_literal_text() {
        assert_eq!(
            escape_format_literal("a { brace } and \"quote\""),
            "a {{ brace }} and \\\"quote\\\""
        );
    }
}
