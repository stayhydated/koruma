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
mod tests;
