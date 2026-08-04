# xtask

This crate is the repository's internal maintenance CLI. Run commands from the workspace root and
use `cargo xtask --help` for the complete argument reference.

| Command | Purpose |
| --- | --- |
| `cargo xtask sync-display-ftl` | Synchronize English Fluent messages with validator `Display` implementations |
| `cargo xtask sync-display-ftl --check` | Check message synchronization without writing files |
| `cargo xtask build book` | Build the mdBook into `web/public/book` |
| `cargo xtask build llms-txt` | Generate the public LLM documentation files from the book |
| `cargo xtask build web` | Build the release site into `web/dist` |
| `cargo xtask release plan` | Print the crates.io publication order |
| `cargo xtask release publish` | Print publication commands in dependency order |

Pass `--execute` to `release publish` to upload crates. Publishing requires a clean tracked
worktree unless xtask's `--allow-dirty` flag is supplied, and uses `cargo-hack` unless
`--include-dev-deps` is selected.
