use std::{collections::HashMap, fs, path::Path};

use anyhow::{Context as _, Result, anyhow};
use fluent_syntax::{ast, parser};

use super::types::TemplatePart;

pub(super) fn namespace_from_ftl_path(path: &Path) -> Result<Option<String>> {
    if path.extension().is_none_or(|ext| ext != "ftl") {
        return Ok(None);
    }

    let stem = path
        .file_stem()
        .with_context(|| format!("Missing file stem for {}", path.display()))?;
    let namespace = stem
        .to_str()
        .with_context(|| format!("FTL namespace stem is not valid UTF-8: {}", path.display()))?;

    Ok(Some(namespace.to_owned()))
}

pub fn collect_ftl_templates(
    ftl_root: &Path,
) -> Result<HashMap<(String, String), Vec<TemplatePart>>> {
    let mut templates = HashMap::new();

    for entry in fs::read_dir(ftl_root)? {
        let entry = entry?;
        let path = entry.path();
        let Some(namespace) = namespace_from_ftl_path(&path)? else {
            continue;
        };

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
                    "Unrecognized message pattern for '{}' in {}",
                    message.id.name,
                    path.display()
                )
            })?;

            templates.insert((namespace.clone(), message.id.name), template);
        }
    }

    Ok(templates)
}

pub fn template_from_pattern(pattern: &ast::Pattern<String>) -> Result<Vec<TemplatePart>> {
    let mut template = Vec::new();

    for element in &pattern.elements {
        match element {
            ast::PatternElement::TextElement { value } => {
                push_text_part(&mut template, value.clone())
            },
            ast::PatternElement::Placeable { expression } => {
                let Some(name) = root_variable_for_expression(expression) else {
                    anyhow::bail!("Only variable/select placeables are supported");
                };
                template.push(TemplatePart::Placeholder(name));
            },
        }
    }

    Ok(template)
}

pub fn root_variable_for_expression(expression: &ast::Expression<String>) -> Option<String> {
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

pub fn push_text_part(template: &mut Vec<TemplatePart>, text: String) {
    if text.is_empty() {
        return;
    }

    if let Some(TemplatePart::Text(existing)) = template.last_mut() {
        existing.push_str(&text);
    } else {
        template.push(TemplatePart::Text(text));
    }
}
