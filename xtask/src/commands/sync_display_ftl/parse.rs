use std::collections::HashMap;

use anyhow::{Context as _, Result, anyhow, bail};
use quote::ToTokens as _;
use syn::{
    Block, Expr, ExprLit, ImplItem, ItemImpl, Lit, Macro, Member, Stmt, Type, parse::Parser as _,
    punctuated::Punctuated, spanned::Spanned as _,
};

use super::types::{FormatChunk, ParsedDisplay};

pub fn parse_display_impl(item_impl: &ItemImpl) -> Result<Option<ParsedDisplay>> {
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

    let format_expr = args
        .iter()
        .nth(1)
        .expect("write! argument count checked above");

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

pub fn display_impl_type_name(item_impl: &ItemImpl) -> Option<String> {
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

pub fn extract_type_ident(ty: &Type) -> Option<String> {
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

pub fn find_fmt_fn(item_impl: &ItemImpl) -> Option<&syn::ImplItemFn> {
    item_impl
        .items
        .iter()
        .find_map(|impl_item| match impl_item {
            ImplItem::Fn(method) if method.sig.ident == "fmt" => Some(method),
            _ => None,
        })
}

pub fn find_write_macro_in_block(block: &Block) -> Option<&Macro> {
    for stmt in &block.stmts {
        if let Some(mac) = find_write_macro_in_stmt(stmt) {
            return Some(mac);
        }
    }

    None
}

pub fn find_write_macro_in_stmt(stmt: &Stmt) -> Option<&Macro> {
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

pub fn find_write_macro_in_expr(expr: &Expr) -> Option<&Macro> {
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

pub fn is_write_macro(mac: &Macro) -> bool {
    mac.path
        .segments
        .last()
        .is_some_and(|segment| segment.ident == "write")
}

pub fn format_to_expr_map(format_str: &str, args: &[Expr]) -> Result<HashMap<String, String>> {
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

pub fn parse_slot_index(spec: &str, next_arg: usize) -> Result<(usize, bool)> {
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

    bail!("unrecognized format slot '{{{spec}}}'")
}

pub fn parse_format_chunks(format_str: &str) -> Result<Vec<FormatChunk>> {
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

pub fn infer_variable_name(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Path(path) => path.path.get_ident().map(|ident| ident.to_string()),
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

pub fn member_name(member: &Member) -> Option<String> {
    match member {
        Member::Named(ident) => Some(ident.to_string()),
        Member::Unnamed(index) => Some(index.index.to_string()),
    }
}

pub fn is_self_expr(expr: &Expr) -> bool {
    matches!(expr, Expr::Path(path) if path.path.is_ident("self"))
}
