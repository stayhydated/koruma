# xtask

`xtask` is the repository's internal maintenance CLI for synchronizing validator
messages and generating the public documentation and site.

Requires Rust 1.99 or newer.

## Commands

Run these commands from the workspace root. The synchronization command reads
English templates under `crates/koruma-collection/i18n/`; the documentation
generators read `book/src`, and the site build reads `web`. Use
`cargo xtask --help` for the complete argument reference.

| Command | Purpose |
| --- | --- |
| `cargo xtask sync-display-ftl` | Update validator `Display` implementations from English Fluent templates |
| `cargo xtask sync-display-ftl --check` | Check message synchronization without writing files |
| `cargo xtask build book` | Build the mdBook into `web/public/book` |
| `cargo xtask build llms-txt` | Generate the public LLM documentation files from the book |
| `cargo xtask build web` | Build the release site into `web/dist` |
