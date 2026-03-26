use std::{
    collections::{BTreeMap, HashSet},
    fs,
    path::Path,
};

use anyhow::Result;
use heck::ToSnakeCase as _;
use syn::{Attribute, Expr, ExprLit, File, Item, Lit, Meta, punctuated::Punctuated};

use super::parse::parse_display_impl;
use super::types::{DisplayInfo, ValidatorInfo};

pub fn collect_rs_files(dir: &Path, out: &mut Vec<std::path::PathBuf>) -> Result<()> {
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

pub fn collect_validator_info(
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

pub fn collect_display_info(
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

pub fn extract_namespace(attrs: &[Attribute]) -> Option<String> {
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

pub fn extract_namespace_from_fluent_meta(list: &syn::MetaList) -> Option<String> {
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
