# AGENTS

This repository is a Rust workspace. Keep changes scoped, follow existing code style, and prefer simple, explicit implementations. See `CONTRIBUTING.md` for contribution expectations.

## Quick commands
- `just fmt` (runs `cargo sort-derives`, `cargo fmt`, `taplo fmt`, `mdformat`)
- `just clippy`
- `just check`
- `just test`
- `just test-docs`
- `just test-publish`

## Workspace layout
- `crates/koruma`: public facade crate; re-exports traits and derive macros; feature gates.
- `crates/koruma-core`: core traits and optional showcase registry types.
- `crates/koruma-derive`: proc-macro crate; expansion logic in `src/expand`.
- `crates/koruma-derive-core`: parsing + utilities shared by derive macros.
- `crates/koruma-collection`: built-in validators + optional i18n resources.
- `examples/`: usage demos and integration examples.

## Conventions
- Validators are structs annotated with `#[koruma::validator]` and have a `#[koruma(value)]` field.
- Validator builders are generated via `bon` (re-exported as `koruma::bon`).
- Derive targets use `#[derive(Koruma)]`; error enums can use `KorumaAllDisplay`/`KorumaAllFluent`.
- Prefer snapshot tests (`insta`) for macro parsing/expansion behavior.

## Docs
- `crates/koruma/src/lib.rs` includes `crates/koruma/README.md` via `include_str!`. Keep that README aligned with the root `README.md` when updating docs.
