use std::{
    collections::{BTreeMap, HashMap, HashSet},
    env, fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, anyhow, bail};
use fluent_syntax::{ast, parser, serializer};
use heck::ToSnakeCase as _;
use syn::{
    Attribute, Block, Expr, ExprLit, File, ImplItem, Item, ItemImpl, Lit, Macro, Member, Meta,
    Stmt, Type, parse::Parser as _, punctuated::Punctuated,
};

#[derive(Clone, Debug)]
struct SyncOptions {
    check: bool,
    allow_new_variables: bool,
    verbose: bool,
}

#[derive(Clone, Debug)]
struct ValidatorInfo {
    name: String,
    namespace: String,
    message_id: String,
    source: PathBuf,
}

#[derive(Clone, Debug)]
struct DisplayInfo {
    template: Vec<TemplatePart>,
    source: PathBuf,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum TemplatePart {
    Text(String),
    Placeholder(String),
}

#[derive(Debug)]
struct FtlResource {
    path: PathBuf,
    resource: ast::Resource<String>,
    changed: usize,
}

#[derive(Debug, Default)]
struct ExistingPatternInfo {
    variables: HashSet<String>,
    placeable_for_var: HashMap<String, ast::PatternElement<String>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum FormatChunk {
    Text(String),
    Slot(String),
}

fn main() -> Result<()> {
    let mut args: Vec<String> = env::args().skip(1).collect();

    if args
        .first()
        .is_some_and(|arg| arg == "-h" || arg == "--help")
    {
        print_help();
        return Ok(());
    }

    let command = if args.first().is_some_and(|arg| !arg.starts_with('-')) {
        args.remove(0)
    } else {
        "sync-display-ftl".to_string()
    };

    match command.as_str() {
        "sync-display-ftl" => {
            let options = parse_sync_options(&args)?;
            run_sync_display_ftl(options)
        },
        other => {
            bail!("Unknown command '{other}'. Expected 'sync-display-ftl'. Use --help for usage.")
        },
    }
}

fn print_help() {
    println!("Usage:");
    println!(
        "  cargo run -p xtask -- sync-display-ftl [--check] [--allow-new-variables] [--verbose]"
    );
    println!();
    println!("Flags:");
    println!("  --check                Exit with non-zero status if files would change.");
    println!(
        "  --allow-new-variables  Allow placeholders not already present in the EN FTL message."
    );
    println!("  --verbose              Print each updated message id.");
}

fn parse_sync_options(args: &[String]) -> Result<SyncOptions> {
    let mut options = SyncOptions {
        check: false,
        allow_new_variables: false,
        verbose: false,
    };

    for arg in args {
        match arg.as_str() {
            "--check" => options.check = true,
            "--allow-new-variables" => options.allow_new_variables = true,
            "--verbose" => options.verbose = true,
            unknown => bail!("Unknown flag '{unknown}'. Use --help for usage."),
        }
    }

    Ok(options)
}

fn run_sync_display_ftl(options: SyncOptions) -> Result<()> {
    let workspace_root = workspace_root();
    let validators_root = workspace_root.join("crates/koruma-collection/src/validators");
    let ftl_root = workspace_root.join("crates/koruma-collection/i18n/en/koruma-collection");

    let mut validator_files = Vec::new();
    collect_rs_files(&validators_root, &mut validator_files)
        .with_context(|| format!("Failed to scan {}", validators_root.display()))?;
    validator_files.sort();

    let mut validators = BTreeMap::<String, ValidatorInfo>::new();
    let mut displays = BTreeMap::<String, DisplayInfo>::new();

    for file in &validator_files {
        let source = fs::read_to_string(file)
            .with_context(|| format!("Failed to read validator file {}", file.display()))?;
        let parsed: File = syn::parse_file(&source)
            .with_context(|| format!("Failed to parse Rust AST for {}", file.display()))?;

        collect_validator_info(file, &parsed, &mut validators);
        collect_display_info(file, &parsed, &mut displays)?;
    }

    let mut resources = HashMap::<String, FtlResource>::new();
    let mut missing_display = Vec::<String>::new();
    let mut missing_message = Vec::<String>::new();
    let mut updated = 0usize;

    for validator in validators.values() {
        let Some(display) = displays.get(&validator.name) else {
            missing_display.push(format!(
                "{} ({})",
                validator.name,
                validator.source.display()
            ));
            continue;
        };

        if !resources.contains_key(&validator.namespace) {
            let path = ftl_root.join(format!("{}.ftl", validator.namespace));
            let source = fs::read_to_string(&path)
                .with_context(|| format!("Failed to read {}", path.display()))?;
            let parsed = parser::parse(source.clone()).map_err(|(_, errors)| {
                anyhow!(
                    "Failed to parse FTL AST for {} ({} parser errors)",
                    path.display(),
                    errors.len()
                )
            })?;

            resources.insert(
                validator.namespace.clone(),
                FtlResource {
                    path,
                    resource: parsed,
                    changed: 0,
                },
            );
        }

        let resource = resources
            .get_mut(&validator.namespace)
            .expect("resource inserted above");

        let Some(message) = find_message_mut(&mut resource.resource, &validator.message_id) else {
            missing_message.push(format!(
                "{} -> {} ({})",
                validator.name,
                validator.message_id,
                resource.path.display()
            ));
            continue;
        };

        let Some(current_pattern) = message.value.as_ref() else {
            continue;
        };

        let existing = analyze_pattern(current_pattern);
        let rendered = render_pattern(&display.template, &existing, options.allow_new_variables);

        if message.value.as_ref() != Some(&rendered) {
            message.value = Some(rendered);
            resource.changed += 1;
            updated += 1;

            if options.verbose {
                println!(
                    "updated {}:{} (from {})",
                    resource.path.display(),
                    validator.message_id,
                    display.source.display()
                );
            }
        }
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

    if options.check {
        if updated == 0 {
            println!("sync-display-ftl: no changes needed.");
            return Ok(());
        }

        bail!("sync-display-ftl: {updated} message(s) would be updated.");
    }

    let mut files_written = 0usize;
    for resource in resources.values() {
        if resource.changed == 0 {
            continue;
        }

        let serialized = serializer::serialize(&resource.resource);
        fs::write(&resource.path, serialized)
            .with_context(|| format!("Failed to write {}", resource.path.display()))?;
        files_written += 1;
    }

    println!("sync-display-ftl: updated {updated} message(s) across {files_written} file(s).");
    Ok(())
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")))
}

fn collect_rs_files(dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;

        if file_type.is_dir() {
            collect_rs_files(&path, out)?;
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            out.push(path);
        }
    }

    Ok(())
}

fn collect_validator_info(
    file_path: &Path,
    parsed: &File,
    validators: &mut BTreeMap<String, ValidatorInfo>,
) {
    for item in &parsed.items {
        let Item::Struct(item_struct) = item else {
            continue;
        };

        let name = item_struct.ident.to_string();
        if !name.ends_with("Validation") {
            continue;
        }

        let Some(namespace) = extract_namespace(&item_struct.attrs) else {
            continue;
        };

        let message_id = name.to_snake_case();
        validators.insert(
            name.clone(),
            ValidatorInfo {
                name,
                namespace,
                message_id,
                source: file_path.to_path_buf(),
            },
        );
    }
}

fn collect_display_info(
    file_path: &Path,
    parsed: &File,
    displays: &mut BTreeMap<String, DisplayInfo>,
) -> Result<()> {
    for item in &parsed.items {
        let Item::Impl(item_impl) = item else {
            continue;
        };

        let Some((type_name, template)) = parse_display_impl(item_impl)? else {
            continue;
        };

        displays.insert(
            type_name.clone(),
            DisplayInfo {
                template,
                source: file_path.to_path_buf(),
            },
        );
    }

    Ok(())
}

fn extract_namespace(attrs: &[Attribute]) -> Option<String> {
    for attr in attrs {
        if attr.path().is_ident("fluent")
            && let Meta::List(list) = &attr.meta
            && let Some(namespace) = extract_namespace_from_fluent_meta(list)
        {
            return Some(namespace);
        }

        if !attr.path().is_ident("cfg_attr") {
            continue;
        }

        let metas = attr
            .parse_args_with(Punctuated::<Meta, syn::Token![,]>::parse_terminated)
            .ok()?;

        for meta in metas {
            let Meta::List(list) = meta else {
                continue;
            };

            if !list.path.is_ident("fluent") {
                continue;
            }

            if let Some(namespace) = extract_namespace_from_fluent_meta(&list) {
                return Some(namespace);
            }
        }
    }

    None
}

fn extract_namespace_from_fluent_meta(list: &syn::MetaList) -> Option<String> {
    let metas = list
        .parse_args_with(Punctuated::<Meta, syn::Token![,]>::parse_terminated)
        .ok()?;

    for meta in metas {
        let Meta::NameValue(named) = meta else {
            continue;
        };

        if !named.path.is_ident("namespace") {
            continue;
        }

        let Expr::Lit(ExprLit {
            lit: Lit::Str(value),
            ..
        }) = named.value
        else {
            continue;
        };

        return Some(value.value());
    }

    None
}

fn parse_display_impl(item_impl: &ItemImpl) -> Result<Option<(String, Vec<TemplatePart>)>> {
    let Some((_, trait_path, _)) = &item_impl.trait_ else {
        return Ok(None);
    };

    if trait_path
        .segments
        .last()
        .is_none_or(|segment| segment.ident != "Display")
    {
        return Ok(None);
    }

    let Some(type_name) = extract_type_ident(&item_impl.self_ty) else {
        return Ok(None);
    };

    let Some(fmt_fn) = item_impl
        .items
        .iter()
        .find_map(|impl_item| match impl_item {
            ImplItem::Fn(method) if method.sig.ident == "fmt" => Some(method),
            _ => None,
        })
    else {
        return Ok(None);
    };

    let Some(write_macro) = find_write_macro_in_block(&fmt_fn.block) else {
        return Ok(None);
    };

    let args = Punctuated::<Expr, syn::Token![,]>::parse_terminated
        .parse2(write_macro.tokens.clone())
        .with_context(|| format!("Failed to parse write! arguments for {type_name}"))?;

    if args.len() < 2 {
        return Ok(None);
    }

    let Some(format_expr) = args.iter().nth(1) else {
        return Ok(None);
    };

    let Expr::Lit(ExprLit {
        lit: Lit::Str(format_lit),
        ..
    }) = format_expr
    else {
        return Ok(None);
    };

    let value_args: Vec<Expr> = args.iter().skip(2).cloned().collect();
    let template = format_to_template(&format_lit.value(), &value_args)?;

    Ok(Some((type_name, template)))
}

fn extract_type_ident(ty: &Type) -> Option<String> {
    match ty {
        Type::Path(type_path) => type_path
            .path
            .segments
            .last()
            .map(|segment| segment.ident.to_string()),
        Type::Reference(reference) => extract_type_ident(&reference.elem),
        Type::Paren(paren) => extract_type_ident(&paren.elem),
        _ => None,
    }
}

fn find_write_macro_in_block(block: &Block) -> Option<&Macro> {
    for stmt in &block.stmts {
        if let Some(mac) = find_write_macro_in_stmt(stmt) {
            return Some(mac);
        }
    }

    None
}

fn find_write_macro_in_stmt(stmt: &Stmt) -> Option<&Macro> {
    match stmt {
        Stmt::Local(local) => local
            .init
            .as_ref()
            .and_then(|init| find_write_macro_in_expr(&init.expr)),
        Stmt::Item(_) => None,
        Stmt::Expr(expr, _) => find_write_macro_in_expr(expr),
        Stmt::Macro(stmt_macro) => {
            if is_write_macro(&stmt_macro.mac) {
                Some(&stmt_macro.mac)
            } else {
                None
            }
        },
    }
}

fn find_write_macro_in_expr(expr: &Expr) -> Option<&Macro> {
    match expr {
        Expr::Macro(expr_macro) => {
            if is_write_macro(&expr_macro.mac) {
                Some(&expr_macro.mac)
            } else {
                None
            }
        },
        Expr::Block(expr_block) => find_write_macro_in_block(&expr_block.block),
        Expr::Group(expr_group) => find_write_macro_in_expr(&expr_group.expr),
        Expr::Paren(expr_paren) => find_write_macro_in_expr(&expr_paren.expr),
        Expr::Return(expr_return) => expr_return
            .expr
            .as_ref()
            .and_then(|expr| find_write_macro_in_expr(expr)),
        Expr::Try(expr_try) => find_write_macro_in_expr(&expr_try.expr),
        _ => None,
    }
}

fn is_write_macro(mac: &Macro) -> bool {
    mac.path
        .segments
        .last()
        .is_some_and(|segment| segment.ident == "write")
}

fn format_to_template(format_str: &str, args: &[Expr]) -> Result<Vec<TemplatePart>> {
    let chunks = parse_format_chunks(format_str)?;

    let mut template = Vec::new();
    let mut next_arg = 0usize;

    for chunk in chunks {
        match chunk {
            FormatChunk::Text(value) => template.push(TemplatePart::Text(value)),
            FormatChunk::Slot(spec) => {
                let (arg_index, is_explicit) = parse_slot_index(&spec, next_arg)?;

                let expr = args.get(arg_index).ok_or_else(|| {
                    anyhow!(
                        "format slot references argument #{arg_index}, but only {} argument(s) found",
                        args.len()
                    )
                })?;

                let inferred =
                    infer_variable_name(expr).unwrap_or_else(|| format!("arg{}", arg_index + 1));
                template.push(TemplatePart::Placeholder(inferred));

                if !is_explicit {
                    next_arg += 1;
                }
            },
        }
    }

    Ok(template)
}

fn parse_slot_index(spec: &str, next_arg: usize) -> Result<(usize, bool)> {
    let trimmed = spec.trim();

    if trimmed.is_empty() || trimmed.starts_with(':') {
        return Ok((next_arg, false));
    }

    let index_text = trimmed.split(':').next().unwrap_or(trimmed).trim();
    if index_text.chars().all(|c| c.is_ascii_digit()) {
        let index = index_text
            .parse::<usize>()
            .with_context(|| format!("invalid explicit slot index '{index_text}'"))?;
        return Ok((index, true));
    }

    bail!("unsupported format slot '{{{spec}}}'")
}

fn parse_format_chunks(format_str: &str) -> Result<Vec<FormatChunk>> {
    let mut chars = format_str.chars().peekable();
    let mut text = String::new();
    let mut chunks = Vec::new();

    while let Some(ch) = chars.next() {
        match ch {
            '{' => {
                if chars.peek() == Some(&'{') {
                    chars.next();
                    text.push('{');
                    continue;
                }

                if !text.is_empty() {
                    chunks.push(FormatChunk::Text(std::mem::take(&mut text)));
                }

                let mut slot = String::new();
                let mut found_end = false;
                for next in chars.by_ref() {
                    if next == '}' {
                        found_end = true;
                        break;
                    }
                    slot.push(next);
                }

                if !found_end {
                    bail!("Unclosed '{{' in format string: {format_str}");
                }

                chunks.push(FormatChunk::Slot(slot));
            },
            '}' => {
                if chars.peek() == Some(&'}') {
                    chars.next();
                    text.push('}');
                } else {
                    bail!("Unmatched '}}' in format string: {format_str}");
                }
            },
            other => text.push(other),
        }
    }

    if !text.is_empty() {
        chunks.push(FormatChunk::Text(text));
    }

    Ok(chunks)
}

fn infer_variable_name(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Field(field) => {
            let member = member_name(&field.member)?;
            if is_self_expr(&field.base) {
                Some(member)
            } else {
                let base = infer_variable_name(&field.base)?;
                Some(format!("{base}_{member}"))
            }
        },
        Expr::MethodCall(call) => {
            let base = infer_variable_name(&call.receiver)?;
            Some(format!("{}_{}", base, call.method))
        },
        Expr::Paren(paren) => infer_variable_name(&paren.expr),
        Expr::Reference(reference) => infer_variable_name(&reference.expr),
        Expr::Unary(unary) => infer_variable_name(&unary.expr),
        _ => None,
    }
}

fn member_name(member: &Member) -> Option<String> {
    match member {
        Member::Named(ident) => Some(ident.to_string()),
        Member::Unnamed(index) => Some(index.index.to_string()),
    }
}

fn is_self_expr(expr: &Expr) -> bool {
    matches!(expr, Expr::Path(path) if path.path.is_ident("self"))
}

fn find_message_mut<'a>(
    resource: &'a mut ast::Resource<String>,
    message_id: &str,
) -> Option<&'a mut ast::Message<String>> {
    for entry in &mut resource.body {
        let ast::Entry::Message(message) = entry else {
            continue;
        };

        if message.id.name == message_id {
            return Some(message);
        }
    }

    None
}

fn analyze_pattern(pattern: &ast::Pattern<String>) -> ExistingPatternInfo {
    let mut info = ExistingPatternInfo::default();
    collect_pattern_variables(pattern, &mut info.variables);

    for element in &pattern.elements {
        let ast::PatternElement::Placeable { expression } = element else {
            continue;
        };

        let Some(variable) = root_variable_for_expression(expression) else {
            continue;
        };

        info.placeable_for_var
            .entry(variable)
            .or_insert_with(|| element.clone());
    }

    info
}

fn collect_pattern_variables(pattern: &ast::Pattern<String>, vars: &mut HashSet<String>) {
    for element in &pattern.elements {
        match element {
            ast::PatternElement::TextElement { .. } => {},
            ast::PatternElement::Placeable { expression } => {
                collect_expression_variables(expression, vars)
            },
        }
    }
}

fn collect_expression_variables(expression: &ast::Expression<String>, vars: &mut HashSet<String>) {
    match expression {
        ast::Expression::Inline(inline) => collect_inline_variables(inline, vars),
        ast::Expression::Select { selector, variants } => {
            collect_inline_variables(selector, vars);
            for variant in variants {
                collect_pattern_variables(&variant.value, vars);
            }
        },
    }
}

fn collect_inline_variables(inline: &ast::InlineExpression<String>, vars: &mut HashSet<String>) {
    match inline {
        ast::InlineExpression::VariableReference { id } => {
            vars.insert(id.name.clone());
        },
        ast::InlineExpression::FunctionReference { arguments, .. } => {
            collect_call_argument_variables(arguments, vars);
        },
        ast::InlineExpression::TermReference { arguments, .. } => {
            if let Some(arguments) = arguments {
                collect_call_argument_variables(arguments, vars);
            }
        },
        ast::InlineExpression::Placeable { expression } => {
            collect_expression_variables(expression, vars);
        },
        ast::InlineExpression::StringLiteral { .. }
        | ast::InlineExpression::NumberLiteral { .. }
        | ast::InlineExpression::MessageReference { .. } => {},
    }
}

fn collect_call_argument_variables(args: &ast::CallArguments<String>, vars: &mut HashSet<String>) {
    for positional in &args.positional {
        collect_inline_variables(positional, vars);
    }

    for named in &args.named {
        collect_inline_variables(&named.value, vars);
    }
}

fn root_variable_for_expression(expression: &ast::Expression<String>) -> Option<String> {
    match expression {
        ast::Expression::Inline(ast::InlineExpression::VariableReference { id }) => {
            Some(id.name.clone())
        },
        ast::Expression::Select {
            selector: ast::InlineExpression::VariableReference { id },
            ..
        } => Some(id.name.clone()),
        _ => None,
    }
}

fn render_pattern(
    template: &[TemplatePart],
    existing: &ExistingPatternInfo,
    allow_new_variables: bool,
) -> ast::Pattern<String> {
    let mut elements = Vec::<ast::PatternElement<String>>::new();

    for part in template {
        match part {
            TemplatePart::Text(value) => push_text_element(&mut elements, value.clone()),
            TemplatePart::Placeholder(variable) => {
                if !allow_new_variables && !existing.variables.contains(variable) {
                    continue;
                }

                if let Some(existing_placeable) = existing.placeable_for_var.get(variable) {
                    elements.push(existing_placeable.clone());
                } else {
                    elements.push(ast::PatternElement::Placeable {
                        expression: ast::Expression::Inline(
                            ast::InlineExpression::VariableReference {
                                id: ast::Identifier {
                                    name: variable.clone(),
                                },
                            },
                        ),
                    });
                }
            },
        }
    }

    normalize_text_elements(&mut elements);

    if elements.is_empty() {
        elements.push(ast::PatternElement::TextElement {
            value: String::new(),
        });
    }

    ast::Pattern { elements }
}

fn push_text_element(elements: &mut Vec<ast::PatternElement<String>>, text: String) {
    if text.is_empty() {
        return;
    }

    if let Some(ast::PatternElement::TextElement { value }) = elements.last_mut() {
        value.push_str(&text);
        return;
    }

    elements.push(ast::PatternElement::TextElement { value: text });
}

fn normalize_text_elements(elements: &mut Vec<ast::PatternElement<String>>) {
    for element in elements.iter_mut() {
        let ast::PatternElement::TextElement { value } = element else {
            continue;
        };
        *value = collapse_whitespace(value);
    }

    if let Some(ast::PatternElement::TextElement { value }) = elements.first_mut() {
        *value = value.trim_start().to_string();
    }

    if let Some(ast::PatternElement::TextElement { value }) = elements.last_mut() {
        *value = value.trim_end().to_string();
    }

    let mut normalized: Vec<ast::PatternElement<String>> = Vec::with_capacity(elements.len());
    for element in elements.drain(..) {
        match element {
            ast::PatternElement::TextElement { value } if value.is_empty() => {},
            ast::PatternElement::TextElement { value } => {
                if let Some(ast::PatternElement::TextElement { value: previous }) =
                    normalized.last_mut()
                {
                    previous.push_str(&value);
                } else {
                    normalized.push(ast::PatternElement::TextElement { value });
                }
            },
            other => normalized.push(other),
        }
    }

    *elements = normalized;
}

fn collapse_whitespace(value: &str) -> String {
    let mut result = String::with_capacity(value.len());
    let mut previous_was_ws = false;

    for ch in value.chars() {
        if ch.is_whitespace() {
            if !previous_was_ws {
                result.push(' ');
                previous_was_ws = true;
            }
        } else {
            result.push(ch);
            previous_was_ws = false;
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_str;

    #[test]
    fn infers_template_slots_from_display_format() {
        let args = vec![
            parse_str::<Expr>("self.actual").expect("valid expression"),
            parse_str::<Expr>("self.min").expect("valid expression"),
            parse_str::<Expr>("self.max").expect("valid expression"),
        ];

        let template = format_to_template("value {} in [{}, {}]", &args).expect("parse template");

        assert_eq!(
            template,
            vec![
                TemplatePart::Text("value ".to_string()),
                TemplatePart::Placeholder("actual".to_string()),
                TemplatePart::Text(" in [".to_string()),
                TemplatePart::Placeholder("min".to_string()),
                TemplatePart::Text(", ".to_string()),
                TemplatePart::Placeholder("max".to_string()),
                TemplatePart::Text("]".to_string()),
            ]
        );
    }

    #[test]
    fn filters_out_non_existing_variables_by_default() {
        let template = vec![
            TemplatePart::Text("value ".to_string()),
            TemplatePart::Placeholder("actual".to_string()),
            TemplatePart::Text(" must be between ".to_string()),
            TemplatePart::Placeholder("min".to_string()),
            TemplatePart::Text(" and ".to_string()),
            TemplatePart::Placeholder("max".to_string()),
        ];

        let mut existing = ExistingPatternInfo::default();
        existing.variables.insert("min".to_string());
        existing.variables.insert("max".to_string());

        let rendered = render_pattern(&template, &existing, false);
        let serialized = serializer::serialize(&ast::Resource {
            body: vec![ast::Entry::Message(ast::Message {
                id: ast::Identifier {
                    name: "range_validation".to_string(),
                },
                value: Some(rendered),
                attributes: vec![],
                comment: None,
            })],
        });

        assert_eq!(
            serialized,
            "range_validation = value must be between { $min } and { $max }\n"
        );
    }
}
