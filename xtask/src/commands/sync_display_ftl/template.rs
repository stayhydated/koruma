use anyhow::{Result, anyhow};
use proc_macro2::{LineColumn, Span};
use quote::ToTokens as _;
use syn::Expr;

use super::types::{DisplayInfo, Replacement, TemplatePart, ValidatorInfo};

pub fn template_to_write_parts(
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

pub fn resolve_placeholder_expr(
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

pub fn escape_format_literal(input: &str) -> String {
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

pub fn build_write_call(format_literal: &str, args: &[String]) -> String {
    let mut write_call = format!("write!(f, \"{format_literal}\"");
    for arg in args {
        write_call.push_str(", ");
        write_call.push_str(arg);
    }
    write_call.push(')');
    write_call
}

pub fn line_start_offsets(source: &str) -> Vec<usize> {
    let mut starts = vec![0];
    for (idx, byte) in source.bytes().enumerate() {
        if byte == b'\n' {
            starts.push(idx + 1);
        }
    }
    starts
}

pub fn span_to_byte_range(
    span: Span,
    line_starts: &[usize],
    source: &str,
) -> Result<(usize, usize)> {
    let start = line_col_to_offset(span.start(), line_starts)?;
    let end = line_col_to_offset(span.end(), line_starts)?;

    if end < start || end > source.len() {
        anyhow::bail!("Invalid byte range from span: {start}..{end}");
    }

    Ok((start, end))
}

pub fn line_col_to_offset(pos: LineColumn, line_starts: &[usize]) -> Result<usize> {
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

pub fn apply_replacements(source: &str, replacements: &[Replacement]) -> String {
    let mut rendered = source.to_string();
    let mut ordered = replacements.to_vec();
    ordered.sort_by_key(|replacement| replacement.start);

    for replacement in ordered.into_iter().rev() {
        rendered.replace_range(replacement.start..replacement.end, &replacement.replacement);
    }

    rendered
}

pub fn write_call_changed(existing: &str, next: &str) -> bool {
    let existing_expr = syn::parse_str::<Expr>(existing);
    let next_expr = syn::parse_str::<Expr>(next);

    match (existing_expr, next_expr) {
        (Ok(existing_expr), Ok(next_expr)) => {
            existing_expr.to_token_stream().to_string() != next_expr.to_token_stream().to_string()
        },
        _ => existing != next,
    }
}
