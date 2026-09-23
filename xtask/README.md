# xtask

This crate is the repository's internal maintenance CLI. Run commands from the workspace root and
use `cargo xtask --help` for the complete argument reference.

| Command | Purpose |
| --- | --- |
| `cargo xtask sync-display-ftl` | Update validator `Display` implementations from English Fluent templates |
| `cargo xtask sync-display-ftl --check` | Check message synchronization without writing files |
| `cargo xtask build book` | Build the mdBook into `web/public/book` |
| `cargo xtask build llms-txt` | Generate the public LLM documentation files from the book |
| `cargo xtask build web` | Build the release site into `web/dist` |
