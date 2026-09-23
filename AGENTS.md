# AGENTS.md

Start application-facing changes in `crates/koruma` and built-in validation rules
in `crates/koruma-collection`. Use `just --list` for the local command index.

## Change ownership

| Surface | Audience and responsibility |
| --- | --- |
| `crates/koruma` | Application facade, public feature gates, traits and macro re-exports |
| `crates/koruma-collection/src/validators` | Application-facing validators organized by domain |
| `crates/koruma-core` | Public integration traits, error interfaces, nested and newtype contracts |
| `crates/koruma-derive` | Procedural macros, generated APIs, diagnostics and expansion snapshots |
| `crates/koruma-derive-core` | Public parsers and typed models for Koruma attributes |
| `tests/koruma-derive-fixtures` | Compile-pass and compile-fail validation of macro contracts |
| `examples/readme` | Canonical executable usage examples and shared Fluent assets |
| `book/src` | Application-developer guides and validator reference |
| `skills/use-koruma` | Application-developer skill and its feature and validator references |
| `web` | Public Dioxus site and showcase demos |
| `xtask` | Internal generation and maintenance commands; see `xtask/README.md` |

## Keep public guidance aligned

- When a public API, feature, or usage pattern changes, update the relevant
  executable example, root or crate README, book page, public rustdocs, and
  `skills/use-koruma` reference in the same change.
- Crate READMEs are included in rustdocs with `include_str!`. Treat their Rust
  snippets as crate documentation examples.
- Keep application documentation example-first. Keep implementation rationale
  close to source module docs, comments, tests, snapshots, and UI fixtures.
- When validator inventory or feature requirements change, synchronize
  `crates/koruma-collection/README.md`, `book/src/koruma_collection.md`, and
  `skills/use-koruma/references/validator-catalog.md`.
- When attribute syntax, generated accessors, target selection, nested/newtype
  behavior, constructors, or metadata changes, synchronize macro rustdocs,
  `crates/koruma-derive/README.md`, `crates/koruma-derive-core/README.md`, the
  relevant book pages, and `skills/use-koruma/references/koruma-feature-map.md`.
  Update the affected trybuild fixtures and expansion snapshots with the code.

## Validators and localization

- Add built-in validators under `crates/koruma-collection/src/validators/` and
  re-export them from their domain module.
- English Fluent templates in `crates/koruma-collection/i18n/` own the matching
  Rust `Display` text. Run `cargo xtask sync-display-ftl` to update implementations
  from those templates, then `cargo xtask sync-display-ftl --check`.
- Keep localized messages aligned with validator message fields. If the i18n
  path layout changes, update `crowdin.yml`,
  `.github/workflows/crowdin-upload.yml`, and
  `.github/workflows/crowdin-sync.yml`.
- Keep showcase metadata and `web` demos aligned when changing
  `internal-showcase` behavior.

## Generated documentation and site

Change source files and use the owning generator instead of editing outputs:

| Source and generator | Output |
| --- | --- |
| `book/src`, `cargo xtask build book` | `web/public/book/` |
| `book/src`, `cargo xtask build llms-txt` | `web/public/llms.txt`, `web/public/llms-full.txt`, `web/public/llms/` |
| `web`, `cargo xtask build web` | `web/dist/` |

## Validate the changed surface

Use the narrow check for the affected surface; use `just check` and `just test`
when changes span the workspace.

| Change | Check |
| --- | --- |
| Runtime behavior or built-in validators | `cargo test -p koruma --all-features --locked` or `cargo test -p koruma-collection --all-features --locked` |
| Macro expansion or parsing | `cargo test -p koruma-derive -p koruma-derive-core --all-features --locked` |
| Compile-time macro contracts | `cargo test -p koruma-derive-fixtures --all-features --locked` |
| Executable usage examples | `cargo run -p readme --locked` |
| Rustdoc rendering and links | `cargo doc --workspace --all-features --no-deps --locked` |
| Crate README or rustdoc examples | `cargo test -p koruma -p koruma-collection --doc --all-features --locked` |
| Book rendering | `MDBOOK_BUILD__CREATE_MISSING=false mdbook build book` |
| Markdown formatting | `rumdl check .` |
| Display/FTL synchronization | `cargo xtask sync-display-ftl --check` |

Report which checks ran and their results. Distinguish static review, failed
attempts, and successful validation.
