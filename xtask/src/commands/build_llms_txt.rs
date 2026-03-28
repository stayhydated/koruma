use std::fs;
use std::path::Path;

use anyhow::Context;
use mdbook_driver::MDBook;
use mdbook_driver::book::BookItem;

use crate::util::workspace_root;

pub fn run() -> anyhow::Result<()> {
    run_from_workspace_root(&workspace_root()?)
}

fn run_from_workspace_root(workspace_root: &Path) -> anyhow::Result<()> {
    run_with_paths(
        &workspace_root.join("book"),
        &workspace_root.join("web").join("public").join("llms.txt"),
    )
}

pub fn run_with_paths(book_root: &Path, output_path: &Path) -> anyhow::Result<()> {
    println!("Building llms.txt to {}", output_path.display());

    let mdbook = MDBook::load(book_root)
        .with_context(|| format!("Failed to load book from {}", book_root.display()))?;

    let mut output = String::new();

    for item in mdbook.iter() {
        if let BookItem::Chapter(chapter) = item {
            if chapter.is_draft_chapter() {
                continue;
            }
            output.push_str(&chapter.content);
            output.push_str("\n\n---\n\n");
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

#[cfg(test)]
mod tests {
    use std::{fs, path::Path};

    use super::{run_from_workspace_root, run_with_paths};

    fn write_file(path: &Path, content: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("failed to create parent directory");
        }
        fs::write(path, content).expect("failed to write file");
    }

    fn create_book_toml(book_root: &Path) {
        let toml = r#"[book]
title = "Test Book"
"#;
        write_file(&book_root.join("book.toml"), toml);
    }

    #[test]
    fn run_with_paths_concatenates_files_with_separators() {
        let tmp = tempfile::tempdir().expect("failed to create temp directory");
        let book_root = tmp.path().join("book");
        let book_src = book_root.join("src");
        let output_path = tmp.path().join("web").join("public").join("llms.txt");

        create_book_toml(&book_root);

        let summary = r#"# Summary

- [Intro](intro.md)
- [Guide](guide.md)
"#;

        write_file(&book_src.join("SUMMARY.md"), summary);
        write_file(&book_src.join("intro.md"), "# Introduction\n\nWelcome!");
        write_file(&book_src.join("guide.md"), "# Guide\n\nStep by step.");

        run_with_paths(&book_root, &output_path).expect("run_with_paths should succeed");

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
    fn run_with_paths_skips_draft_chapters() {
        let tmp = tempfile::tempdir().expect("failed to create temp directory");
        let book_root = tmp.path().join("book");
        let book_src = book_root.join("src");
        let output_path = tmp.path().join("output").join("llms.txt");

        create_book_toml(&book_root);

        let summary = r#"# Summary

- [Exists](exists.md)
- [Draft]()
"#;

        write_file(&book_src.join("SUMMARY.md"), summary);
        write_file(&book_src.join("exists.md"), "# Exists\n\nContent here.");

        run_with_paths(&book_root, &output_path).expect("run_with_paths should succeed");

        let result = fs::read_to_string(&output_path).expect("failed to read output");
        assert!(result.contains("# Exists"));
        assert!(!result.contains("Draft"));

        let separator_count = result.matches("---").count();
        assert_eq!(
            separator_count, 1,
            "expected 1 separator for 1 existing file"
        );
    }

    #[test]
    fn run_with_paths_fails_for_missing_book() {
        let tmp = tempfile::tempdir().expect("failed to create temp directory");
        let book_root = tmp.path().join("book");
        let output_path = tmp.path().join("output").join("llms.txt");

        fs::create_dir_all(&book_root).expect("failed to create book directory");

        let result = run_with_paths(&book_root, &output_path);
        assert!(result.is_err(), "should fail when book.toml is missing");
    }

    #[test]
    fn run_with_paths_creates_output_directory() {
        let tmp = tempfile::tempdir().expect("failed to create temp directory");
        let book_root = tmp.path().join("book");
        let book_src = book_root.join("src");
        let output_path = tmp.path().join("nested").join("deep").join("llms.txt");

        create_book_toml(&book_root);

        let summary = "# Summary\n\n- [Test](test.md)\n";
        write_file(&book_src.join("SUMMARY.md"), summary);
        write_file(&book_src.join("test.md"), "# Test\n\nContent.");

        run_with_paths(&book_root, &output_path).expect("run_with_paths should succeed");

        assert!(output_path.exists(), "output file should be created");
    }

    #[test]
    fn run_from_workspace_root_uses_default_workspace_paths() {
        let tmp = tempfile::tempdir().expect("failed to create temp directory");
        let workspace_root = tmp.path().join("workspace");
        let book_root = workspace_root.join("book");
        let book_src = book_root.join("src");
        let output_path = workspace_root.join("web").join("public").join("llms.txt");

        create_book_toml(&book_root);

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
}
