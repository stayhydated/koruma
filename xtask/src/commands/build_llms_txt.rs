use std::fs;
use std::path::Path;

use anyhow::Context;
use mdbook::book::SummaryItem;

use crate::util::workspace_root;

pub fn run() -> anyhow::Result<()> {
    run_from_workspace_root(&workspace_root()?)
}

fn run_from_workspace_root(workspace_root: &Path) -> anyhow::Result<()> {
    run_with_paths(
        &workspace_root.join("book").join("src"),
        &workspace_root.join("web").join("public").join("llms.txt"),
    )
}

pub fn run_with_paths(book_src_dir: &Path, output_path: &Path) -> anyhow::Result<()> {
    println!("Building llms.txt to {}", output_path.display());

    let summary_path = book_src_dir.join("SUMMARY.md");
    let summary_content = fs::read_to_string(&summary_path)
        .with_context(|| format!("Failed to read SUMMARY.md from {}", summary_path.display()))?;
    let markdown_paths = extract_markdown_paths(&summary_content)
        .with_context(|| format!("Failed to parse SUMMARY.md from {}", summary_path.display()))?;

    let mut output = String::new();

    for md_file in markdown_paths {
        let file_path = book_src_dir.join(&md_file);

        if file_path.exists() {
            let content = fs::read_to_string(&file_path)
                .with_context(|| format!("Failed to read {}", file_path.display()))?;

            output.push_str(&content);
            output.push_str("\n\n---\n\n");
        } else {
            eprintln!("Warning: File not found: {}", file_path.display());
        }
    }

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create output directory {}", parent.display()))?;
    }

    fs::write(output_path, output)
        .with_context(|| format!("Failed to write llms.txt to {}", output_path.display()))?;

    println!("llms.txt built successfully");
    Ok(())
}

fn extract_markdown_paths(summary_content: &str) -> anyhow::Result<Vec<String>> {
    let summary = mdbook::book::parse_summary(summary_content)?;
    let mut markdown_paths = Vec::new();

    collect_markdown_paths(&summary.prefix_chapters, &mut markdown_paths);
    collect_markdown_paths(&summary.numbered_chapters, &mut markdown_paths);
    collect_markdown_paths(&summary.suffix_chapters, &mut markdown_paths);

    Ok(markdown_paths)
}

fn collect_markdown_paths(items: &[SummaryItem], markdown_paths: &mut Vec<String>) {
    for item in items {
        if let SummaryItem::Link(link) = item {
            if let Some(location) = &link.location {
                markdown_paths.push(location.to_string_lossy().into_owned());
            }
            collect_markdown_paths(&link.nested_items, markdown_paths);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::{Path, PathBuf},
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::{extract_markdown_paths, run_from_workspace_root, run_with_paths};

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

    #[test]
    fn run_with_paths_concatenates_files_with_separators() {
        let tmp = TempDir::new("build_llms_txt");
        let book_src = tmp.path().join("book").join("src");
        let output_path = tmp.path().join("web").join("public").join("llms.txt");

        let summary = r#"# Summary

- [Intro](intro.md)
- [Guide](guide.md)
"#;

        write_file(&book_src.join("SUMMARY.md"), summary);
        write_file(&book_src.join("intro.md"), "# Introduction\n\nWelcome!");
        write_file(&book_src.join("guide.md"), "# Guide\n\nStep by step.");

        run_with_paths(&book_src, &output_path).expect("run_with_paths should succeed");

        let result = fs::read_to_string(&output_path).expect("failed to read output");
        assert!(result.contains("# Introduction"));
        assert!(result.contains("Welcome!"));
        assert!(result.contains("# Guide"));
        assert!(result.contains("Step by step."));
        assert!(result.contains("\n\n---\n\n"));

        let separator_count = result.matches("---").count();
        assert_eq!(separator_count, 2, "expected 2 separators for 2 files");
    }

    #[test]
    fn run_with_paths_skips_missing_files() {
        let tmp = TempDir::new("build_llms_txt_missing");
        let book_src = tmp.path().join("book").join("src");
        let output_path = tmp.path().join("output").join("llms.txt");

        let summary = r#"# Summary

- [Exists](exists.md)
- [Missing](missing.md)
"#;

        write_file(&book_src.join("SUMMARY.md"), summary);
        write_file(&book_src.join("exists.md"), "# Exists\n\nContent here.");

        run_with_paths(&book_src, &output_path).expect("run_with_paths should succeed");

        let result = fs::read_to_string(&output_path).expect("failed to read output");
        assert!(result.contains("# Exists"));
        assert!(!result.contains("Missing"));

        let separator_count = result.matches("---").count();
        assert_eq!(
            separator_count, 1,
            "expected 1 separator for 1 existing file"
        );
    }

    #[test]
    fn run_with_paths_fails_for_missing_summary() {
        let tmp = TempDir::new("build_llms_txt_no_summary");
        let book_src = tmp.path().join("book").join("src");
        let output_path = tmp.path().join("output").join("llms.txt");

        // Don't create SUMMARY.md
        fs::create_dir_all(&book_src).expect("failed to create book src directory");

        let result = run_with_paths(&book_src, &output_path);
        assert!(result.is_err(), "should fail when SUMMARY.md is missing");
        assert!(result.unwrap_err().to_string().contains("SUMMARY.md"));
    }

    #[test]
    fn run_with_paths_creates_output_directory() {
        let tmp = TempDir::new("build_llms_txt_create_dir");
        let book_src = tmp.path().join("book").join("src");
        let output_path = tmp.path().join("nested").join("deep").join("llms.txt");

        let summary = "# Summary\n\n- [Test](test.md)\n";
        write_file(&book_src.join("SUMMARY.md"), summary);
        write_file(&book_src.join("test.md"), "# Test\n\nContent.");

        run_with_paths(&book_src, &output_path).expect("run_with_paths should succeed");

        assert!(output_path.exists(), "output file should be created");
    }

    #[test]
    fn run_from_workspace_root_uses_default_workspace_paths() {
        let tmp = TempDir::new("build_llms_txt_workspace_root");
        let workspace_root = tmp.path().join("workspace");
        let book_src = workspace_root.join("book").join("src");
        let output_path = workspace_root.join("web").join("public").join("llms.txt");

        let summary = "# Summary\n\n- [Test](test.md)\n";
        write_file(&book_src.join("SUMMARY.md"), summary);
        write_file(&book_src.join("test.md"), "# Test\n\nWorkspace root mode.");

        run_from_workspace_root(&workspace_root).expect("run should succeed");

        let result = fs::read_to_string(&output_path).expect("failed to read output");
        assert!(
            result.contains("Workspace root mode."),
            "output should be built under web/public/llms.txt"
        );
    }

    #[test]
    fn run_with_paths_skips_parent_creation_when_output_has_no_parent() {
        let tmp = TempDir::new("build_llms_txt_no_parent");
        let book_src = tmp.path().join("book").join("src");
        let output_path = Path::new("");

        let summary = "# Summary\n\n- [Test](test.md)\n";
        write_file(&book_src.join("SUMMARY.md"), summary);
        write_file(&book_src.join("test.md"), "# Test\n\nContent.");

        let result = run_with_paths(&book_src, output_path);
        assert!(result.is_err(), "empty output path should fail to write");
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Failed to write llms.txt"),
            "error should come from write context"
        );
    }

    #[test]
    fn extract_markdown_paths_parses_simple_link() {
        let summary = "# Summary\n\n- [Introduction](intro.md)\n";
        assert_eq!(
            extract_markdown_paths(summary).expect("summary should parse"),
            vec!["intro.md".to_string()]
        );
    }

    #[test]
    fn extract_markdown_paths_parses_nested_links() {
        let summary = "# Summary\n\n- [Parent](parent.md)\n  - [Child](child.md)\n";
        assert_eq!(
            extract_markdown_paths(summary).expect("summary should parse"),
            vec!["parent.md".to_string(), "child.md".to_string()]
        );
    }

    #[test]
    fn extract_markdown_paths_parses_prefix_numbered_and_suffix_chapters() {
        let summary = "[Intro](intro.md)\n\n- [Chapter](chapter.md)\n\n[Outro](outro.md)\n";
        assert_eq!(
            extract_markdown_paths(summary).expect("summary should parse"),
            vec![
                "intro.md".to_string(),
                "chapter.md".to_string(),
                "outro.md".to_string()
            ]
        );
    }

    #[test]
    fn extract_markdown_paths_returns_empty_when_no_links_exist() {
        let summary = "# Summary\n\n## Part\n";
        assert!(
            extract_markdown_paths(summary)
                .expect("summary should parse")
                .is_empty()
        );
    }

    #[test]
    fn extract_markdown_paths_fails_for_malformed_summary() {
        let summary = "# Summary\n\n- [No link]\n";
        assert!(
            extract_markdown_paths(summary).is_err(),
            "malformed summary should fail to parse"
        );
    }

    #[test]
    fn extract_markdown_paths_skips_draft_chapters() {
        let summary = "# Summary\n\n- [Draft]()\n";
        assert!(
            extract_markdown_paths(summary)
                .expect("summary should parse")
                .is_empty()
        );
    }
}
