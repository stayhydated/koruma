# Project Overview

`koruma` is a per-field validation framework written in Rust, focused on:

1. **Type Safety**: Strongly typed validation error structs generated at compile time.
1. **Ergonomics**: Derive macros and validator attributes that minimize boilerplate.
1. **Developer Experience**: Optional constructors, nested/newtype validation, and fluent/i18n hooks.

## Architecture Documentation Index

| Crate                    | Link to Architecture Doc                                       | Purpose                                                                                      |
| ------------------------ | -------------------------------------------------------------- | -------------------------------------------------------------------------------------------- |
| **Core**                 |                                                                |                                                                                              |
| `koruma`                 | [Architecture](crates/koruma/docs/ARCHITECTURE.md)             | Public facade crate, re-exports core traits and derive macros, defines feature gates.        |
| `koruma-core`            | [Architecture](crates/koruma-core/docs/ARCHITECTURE.md)        | Core traits and optional showcase registry types.                                            |
| **Derive & Parsing**     |                                                                |                                                                                              |
| `koruma-derive-core`     | [Architecture](crates/koruma-derive-core/docs/ARCHITECTURE.md) | Attribute parsing and utilities shared by derive macros.                                     |
| `koruma-derive`          | [Architecture](crates/koruma-derive/docs/ARCHITECTURE.md)      | Proc-macros for validators, error structs, and helper derives.                               |
| **Validator Collection** |                                                                |                                                                                              |
| `koruma-collection`      | [Architecture](crates/koruma-collection/docs/ARCHITECTURE.md)  | Built-in validators with optional fluent/i18n resources.                                     |
| **Automation**           |                                                                |                                                                                              |
| `xtask`                  | [Architecture](xtask/docs/ARCHITECTURE.md)                     | Rust task runner                                                                             |
| **Examples**             |                                                                |                                                                                              |
| `examples/collection-*`  |                                                                | Interactive TUI showcasing validators via the `internal-showcase` feature.                   |
| `examples/shared-lib`    |                                                                | Workspace example sharing validators across crates.                                          |
| `examples/i18n`          |                                                                | Shared Fluent translation assets for examples.                                               |
| `examples/readme`        |                                                                | Canonical executable docs examples. Keep in sync with root `README.md` and `book`            |
| **Web**                  |                                                                |                                                                                              |
| `web`                    |                                                                | Astro-based site for GitHub Pages. Hosts WASM-compiled examples as live demos and the mdBook |
| `book`                   |                                                                | mdBook that shows usage of the user-facing crates                                            |

## Crate Descriptions

### Core Layers

- **`koruma`**: User-facing facade crate. Re-exports core traits, derive macros, and the `bon` builder API.
- **`koruma-core`**: Core validation traits plus optional showcase registry types.

### Derive & Parsing

- **`koruma-derive-core`**: Parses `#[koruma(...)]` attributes into a stable data model for macros and tooling.
- **`koruma-derive`**: Proc-macros for `#[derive(Koruma)]`, `KorumaAllDisplay`, `KorumaAllFluent`, and `#[koruma::validator]`.

### Validator Collection

- **`koruma-collection`**: Built-in validators (string, format, numeric, collection, general) with optional fluent/i18n assets.

### Examples

- **`examples/collection-*`**: Interactive TUI listing showcase-registered validators.
- **`examples/readme`**: Canonical executable source for user-facing examples used by docs.
- **`examples/shared-lib`**: Shared library example for cross-crate validation types.
- **`examples/i18n`**: Fluent locale files used by examples.

### Web

- **`web`**: An Astro-based static site for GitHub Pages. Hosts WASM-compiled examples with live interactive demos. The site is built via the `gh-pages.yml` workflow which compiles Rust examples to WASM and deploys them.

## Development

**Docs**

- User-facing feature documentation must be example-first. Do not add prose-only guidance for behavior changes when a Rust snippet can demonstrate it.
- `examples/readme` is the canonical source of truth for usage examples.
- Keep example behavior and API shape synchronized across `examples/readme` (executable examples), root `README.md` (copied/adapted snippets), and `book/src/*.md` (mdBook narrative + snippets).
- Keep `crates/koruma-collection/README.md` and the `book/src/koruma_collection.md` chapter synchronized when validator inventory, feature flags, or usage guidance changes.
- When updating one of those three surfaces, update the other relevant surfaces in the same change set unless there is a documented reason not to.

**Rust**

- Use `cargo` for building, testing, and running Rust code. In this workspace, keep dependency versions in the workspace root `Cargo.toml` and use `workspace = true` in member crates. Each crate is responsible for selecting the correct dependency `features` in its own `Cargo.toml`.
- Reserve `path` dependencies for the root `Cargo.toml` and for examples (e.g., example-to-example helpers). Non-example crates should reference other workspace crates using `workspace = true` rather than explicit paths.
- Use [insta](https://insta.rs/) for snapshot tests where appropriate, rather than complex assertion-based unit tests.
- Prefer raw multiline strings (or `quote! { ... }` in macro contexts) over escaped single-line literals when embedding Rust code in tests.

**JavaScript**

- Use [bun](https://bun.com/) for dependency management.
- [turborepo](https://turborepo.org/) is used as the build system.
