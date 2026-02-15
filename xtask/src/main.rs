use std::{
    collections::{BTreeMap, HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, anyhow, bail};
use clap::{Args, Parser, Subcommand};
use fluent_syntax::{ast, parser};
use heck::ToSnakeCase as _;
use proc_macro2::{LineColumn, Span};
use quote::ToTokens as _;
use syn::{
    Attribute, Block, Expr, ExprLit, File, ImplItem, Item, ItemImpl, Lit, Macro, Member, Meta,
    Stmt, Type, parse::Parser as _, punctuated::Punctuated, spanned::Spanned as _,
};

#[derive(Clone, Debug)]
struct SyncOptions {
    check: bool,
    verbose: bool,
}

#[derive(Debug, Parser)]
#[command(
    name = "xtask",
    about = "Workspace maintenance tasks.",
    disable_help_subcommand = true
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,

    #[command(flatten)]
    sync: SyncArgs,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Sync source of truth: EN FTL -> Rust std::fmt::Display messages.
    SyncDisplayFtl(SyncArgs),
}

#[derive(Args, Clone, Debug, Default)]
struct SyncArgs {
    /// Exit with non-zero status if files would change.
    #[arg(long)]
    check: bool,
    /// Print each updated Display impl.
    #[arg(long)]
    verbose: bool,
}

impl From<SyncArgs> for SyncOptions {
    fn from(value: SyncArgs) -> Self {
        Self {
            check: value.check,
            verbose: value.verbose,
        }
    }
}

#[derive(Clone, Debug)]
struct ValidatorInfo {
    name: String,
    namespace: String,
    message_id: String,
    source: PathBuf,
    fields: HashSet<String>,
}

#[derive(Clone, Debug)]
struct DisplayInfo {
    expr_by_placeholder: HashMap<String, String>,
    source: PathBuf,
    write_span: Span,
}

#[derive(Clone, Debug)]
struct SyncTarget {
    validator: ValidatorInfo,
    display: DisplayInfo,
    template: Vec<TemplatePart>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum TemplatePart {
    Text(String),
    Placeholder(String),
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum FormatChunk {
    Text(String),
    Slot(String),
}

type ParsedDisplay = (String, HashMap<String, String>, Span);

#[derive(Clone, Debug)]
struct Replacement {
    start: usize,
    end: usize,
    replacement: String,
    type_name: String,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let options: SyncOptions = match cli.command {
        Some(Command::SyncDisplayFtl(sync)) => sync.into(),
        None => cli.sync.into(),
    };

    run_sync_display_ftl(options)
}

fn run_sync_display_ftl(options: SyncOptions) -> Result<()> {
    let workspace_root = workspace_root();
    let validators_root = workspace_root.join("crates/koruma-collection/src/validators");
    let ftl_root = workspace_root.join("crates/koruma-collection/i18n/en/koruma-collection");

    run_sync_display_ftl_with_roots(&validators_root, &ftl_root, options)
}

fn run_sync_display_ftl_with_roots(
    validators_root: &Path,
    ftl_root: &Path,
    options: SyncOptions,
) -> Result<()> {
    let mut validator_files = Vec::new();
    collect_rs_files(validators_root, &mut validator_files)
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

    let templates = collect_ftl_templates(&ftl_root)?;

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
            let (start, end) = span_to_byte_range(target.display.write_span, &line_starts, &source)
                .with_context(|| {
                    format!(
                        "Failed to map write! span for {} in {}",
                        type_name,
                        file.display()
                    )
                })?;

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

        let mut fields = HashSet::new();
        if let syn::Fields::Named(named) = &item_struct.fields {
            for field in &named.named {
                if let Some(ident) = &field.ident {
                    fields.insert(ident.to_string());
                }
            }
        }

        let message_id = name.to_snake_case();
        validators.insert(
            name.clone(),
            ValidatorInfo {
                name,
                namespace,
                message_id,
                source: file_path.to_path_buf(),
                fields,
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

        let Some((type_name, expr_by_placeholder, write_span)) = parse_display_impl(item_impl)?
        else {
            continue;
        };

        displays.insert(
            type_name,
            DisplayInfo {
                expr_by_placeholder,
                source: file_path.to_path_buf(),
                write_span,
            },
        );
    }

    Ok(())
}

fn collect_ftl_templates(ftl_root: &Path) -> Result<HashMap<(String, String), Vec<TemplatePart>>> {
    let mut templates = HashMap::new();

    for entry in fs::read_dir(ftl_root)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().is_none_or(|ext| ext != "ftl") {
            continue;
        }

        let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) else {
            continue;
        };
        let namespace = stem.to_string();

        let source = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read {}", path.display()))?;
        let parsed = parser::parse(source).map_err(|(_, errors)| {
            anyhow!(
                "Failed to parse FTL AST for {} ({} parser errors)",
                path.display(),
                errors.len()
            )
        })?;

        for entry in parsed.body {
            let ast::Entry::Message(message) = entry else {
                continue;
            };

            let Some(pattern) = message.value else {
                continue;
            };

            let template = template_from_pattern(&pattern).with_context(|| {
                format!(
                    "Unsupported message pattern for '{}' in {}",
                    message.id.name,
                    path.display()
                )
            })?;

            templates.insert((namespace.clone(), message.id.name), template);
        }
    }

    Ok(templates)
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

fn parse_display_impl(item_impl: &ItemImpl) -> Result<Option<ParsedDisplay>> {
    let Some(type_name) = display_impl_type_name(item_impl) else {
        return Ok(None);
    };

    let Some(fmt_fn) = find_fmt_fn(item_impl) else {
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
    let expr_by_placeholder = format_to_expr_map(&format_lit.value(), &value_args)?;

    Ok(Some((type_name, expr_by_placeholder, write_macro.span())))
}

fn display_impl_type_name(item_impl: &ItemImpl) -> Option<String> {
    let Some((_, trait_path, _)) = &item_impl.trait_ else {
        return None;
    };

    if trait_path
        .segments
        .last()
        .is_none_or(|segment| segment.ident != "Display")
    {
        return None;
    }

    extract_type_ident(&item_impl.self_ty)
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

fn find_fmt_fn(item_impl: &ItemImpl) -> Option<&syn::ImplItemFn> {
    item_impl
        .items
        .iter()
        .find_map(|impl_item| match impl_item {
            ImplItem::Fn(method) if method.sig.ident == "fmt" => Some(method),
            _ => None,
        })
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

fn format_to_expr_map(format_str: &str, args: &[Expr]) -> Result<HashMap<String, String>> {
    let chunks = parse_format_chunks(format_str)?;

    let mut expr_by_placeholder = HashMap::new();
    let mut next_arg = 0usize;

    for chunk in chunks {
        let FormatChunk::Slot(spec) = chunk else {
            continue;
        };

        let (arg_index, is_explicit) = parse_slot_index(&spec, next_arg)?;

        let expr = args.get(arg_index).ok_or_else(|| {
            anyhow!(
                "format slot references argument #{arg_index}, but only {} argument(s) found",
                args.len()
            )
        })?;

        let placeholder =
            infer_variable_name(expr).unwrap_or_else(|| format!("arg{}", arg_index + 1));
        expr_by_placeholder
            .entry(placeholder)
            .or_insert_with(|| expr.to_token_stream().to_string());

        if !is_explicit {
            next_arg += 1;
        }
    }

    Ok(expr_by_placeholder)
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

fn template_from_pattern(pattern: &ast::Pattern<String>) -> Result<Vec<TemplatePart>> {
    let mut template = Vec::new();

    for element in &pattern.elements {
        match element {
            ast::PatternElement::TextElement { value } => {
                push_text_part(&mut template, value.clone())
            },
            ast::PatternElement::Placeable { expression } => {
                let Some(name) = root_variable_for_expression(expression) else {
                    bail!("Only variable/select placeables are supported");
                };
                template.push(TemplatePart::Placeholder(name));
            },
        }
    }

    Ok(template)
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

fn push_text_part(template: &mut Vec<TemplatePart>, text: String) {
    if text.is_empty() {
        return;
    }

    if let Some(TemplatePart::Text(existing)) = template.last_mut() {
        existing.push_str(&text);
    } else {
        template.push(TemplatePart::Text(text));
    }
}

fn template_to_write_parts(
    template: &[TemplatePart],
    validator: &ValidatorInfo,
    display: &DisplayInfo,
) -> Result<(String, Vec<String>)> {
    let mut format_literal = String::new();
    let mut args = Vec::<String>::new();

    for part in template {
        match part {
            TemplatePart::Text(text) => format_literal.push_str(&escape_format_literal(text)),
            TemplatePart::Placeholder(placeholder) => {
                let expr =
                    resolve_placeholder_expr(placeholder, validator, display).ok_or_else(|| {
                        anyhow!(
                            "Cannot resolve placeholder '${}' for {}",
                            placeholder,
                            validator.name
                        )
                    })?;

                format_literal.push_str("{}");
                args.push(expr);
            },
        }
    }

    Ok((format_literal, args))
}

fn resolve_placeholder_expr(
    placeholder: &str,
    validator: &ValidatorInfo,
    display: &DisplayInfo,
) -> Option<String> {
    if let Some(existing) = display.expr_by_placeholder.get(placeholder) {
        return Some(existing.clone());
    }

    if placeholder == "actual" && validator.fields.contains("actual") {
        return Some("self.actual".to_string());
    }

    if validator.fields.contains(placeholder) {
        return Some(format!("self.{placeholder}"));
    }

    None
}

fn escape_format_literal(input: &str) -> String {
    let mut escaped = String::with_capacity(input.len());

    for ch in input.chars() {
        match ch {
            '\\' => escaped.push_str("\\\\"),
            '"' => escaped.push_str("\\\""),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            '{' => escaped.push_str("{{"),
            '}' => escaped.push_str("}}"),
            other => escaped.push(other),
        }
    }

    escaped
}

fn build_write_call(format_literal: &str, args: &[String]) -> String {
    let mut write_call = format!("write!(f, \"{format_literal}\"");
    for arg in args {
        write_call.push_str(", ");
        write_call.push_str(arg);
    }
    write_call.push(')');
    write_call
}

fn line_start_offsets(source: &str) -> Vec<usize> {
    let mut starts = vec![0];
    for (idx, byte) in source.bytes().enumerate() {
        if byte == b'\n' {
            starts.push(idx + 1);
        }
    }
    starts
}

fn span_to_byte_range(span: Span, line_starts: &[usize], source: &str) -> Result<(usize, usize)> {
    let start = line_col_to_offset(span.start(), line_starts)?;
    let end = line_col_to_offset(span.end(), line_starts)?;

    if end < start || end > source.len() {
        bail!("Invalid byte range from span: {start}..{end}");
    }

    Ok((start, end))
}

fn line_col_to_offset(pos: LineColumn, line_starts: &[usize]) -> Result<usize> {
    let line_idx = pos
        .line
        .checked_sub(1)
        .ok_or_else(|| anyhow!("Invalid line number from span: {}", pos.line))?;
    let line_start = *line_starts.get(line_idx).ok_or_else(|| {
        anyhow!(
            "Span line {} is out of bounds (max line {})",
            pos.line,
            line_starts.len()
        )
    })?;

    Ok(line_start + pos.column)
}

fn apply_replacements(source: &str, replacements: &[Replacement]) -> String {
    let mut rendered = source.to_string();
    let mut ordered = replacements.to_vec();
    ordered.sort_by_key(|replacement| replacement.start);

    for replacement in ordered.into_iter().rev() {
        rendered.replace_range(replacement.start..replacement.end, &replacement.replacement);
    }

    rendered
}

fn write_call_changed(existing: &str, next: &str) -> bool {
    let existing_expr = syn::parse_str::<Expr>(existing);
    let next_expr = syn::parse_str::<Expr>(next);

    match (existing_expr, next_expr) {
        (Ok(existing_expr), Ok(next_expr)) => {
            existing_expr.to_token_stream().to_string() != next_expr.to_token_stream().to_string()
        },
        _ => existing != next,
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::{Path, PathBuf},
        time::{SystemTime, UNIX_EPOCH},
    };

    use syn::spanned::Spanned as _;

    use super::*;

    #[derive(Debug)]
    struct TempDir {
        path: PathBuf,
    }

    impl TempDir {
        fn new(prefix: &str) -> Self {
            let nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system time should be after unix epoch")
                .as_nanos();
            let path = std::env::temp_dir().join(format!(
                "koruma_xtask_{prefix}_{}_{}",
                std::process::id(),
                nanos
            ));
            fs::create_dir_all(&path).expect("failed to create temp directory");
            Self { path }
        }

        fn path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

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
        let tmp = TempDir::new("sync_display_ftl");
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

    #[test]
    fn sync_args_into_sync_options() {
        let options: SyncOptions = SyncArgs {
            check: true,
            verbose: true,
        }
        .into();
        assert!(options.check);
        assert!(options.verbose);
    }

    #[test]
    fn sync_display_ftl_check_mode_reports_pending_changes() {
        let (_tmp, validators_root, ftl_root, _validator_file) = create_sync_fixture();

        let err = run_sync_display_ftl_with_roots(
            &validators_root,
            &ftl_root,
            SyncOptions {
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

        run_sync_display_ftl_with_roots(
            &validators_root,
            &ftl_root,
            SyncOptions {
                check: false,
                verbose: true,
            },
        )
        .expect("sync should apply changes");

        let updated = fs::read_to_string(&validator_file).expect("failed to read updated file");
        assert!(compact_ws(&updated).contains("write!(f,\"Value{}and{}.\",self.min,self.actual)"));

        run_sync_display_ftl_with_roots(
            &validators_root,
            &ftl_root,
            SyncOptions {
                check: true,
                verbose: false,
            },
        )
        .expect("check mode should pass after sync");
    }

    #[test]
    fn collect_rs_files_walks_directories() {
        let tmp = TempDir::new("collect_rs_files");
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

        let unsupported_slot_impl: ItemImpl = syn::parse_quote! {
            impl std::fmt::Display for BrokenValidation {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    write!(f, "Value {name}", self.actual)
                }
            }
        };
        let err = parse_display_impl(&unsupported_slot_impl).expect_err("unsupported slot");
        assert!(err.to_string().contains("unsupported format slot"));
    }

    #[test]
    fn find_write_macro_helpers_cover_statement_and_expression_variants() {
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
        assert_eq!(infer_variable_name(&plain_expr), None);

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
            expr_by_placeholder: [("kind".to_string(), "self.kind".to_string())]
                .into_iter()
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
        let ftl = r#"
simple = Value { $actual }.
unsupported = { "literal" }
"#;
        let resource = parser::parse(ftl.to_string()).expect("valid ftl");

        let simple_pattern = match &resource.body[0] {
            ast::Entry::Message(message) => message.value.as_ref().expect("pattern"),
            _ => panic!("expected message"),
        };
        let simple_template = template_from_pattern(simple_pattern).expect("simple template");
        assert_eq!(
            simple_template,
            vec![
                TemplatePart::Text("Value ".to_string()),
                TemplatePart::Placeholder("actual".to_string()),
                TemplatePart::Text(".".to_string()),
            ]
        );

        let unsupported_pattern = match &resource.body[1] {
            ast::Entry::Message(message) => message.value.as_ref().expect("pattern"),
            _ => panic!("expected message"),
        };
        assert!(template_from_pattern(unsupported_pattern).is_err());

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

        let source_with_span = "fn demo() {\n    write!(f, \"message\");\n}\n";
        let parsed_file: File = syn::parse_file(source_with_span).expect("valid rust file");
        let stmt_span = match &parsed_file.items[0] {
            Item::Fn(item_fn) => item_fn.block.stmts[0].span(),
            _ => panic!("expected function item"),
        };
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
        let ftl = r#"
ip_validation =
    Not a valid { $kind ->
        [v4] IPv4
       *[other] IP
    } address.
"#;
        let resource = parser::parse(ftl.to_string()).expect("valid ftl");
        let message = match &resource.body[0] {
            ast::Entry::Message(message) => message,
            _ => panic!("expected message"),
        };
        let pattern = message.value.as_ref().expect("pattern exists");

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
    fn resolves_actual_from_struct_fields() {
        let validator = ValidatorInfo {
            name: "ExampleValidation".to_string(),
            namespace: "example".to_string(),
            message_id: "example_validation".to_string(),
            source: PathBuf::from("example.rs"),
            fields: ["actual".to_string()].into_iter().collect(),
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
