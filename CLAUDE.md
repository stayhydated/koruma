# Project Overview

`koruma` is a per-field validation framework written in Rust, focused on:

1. Type Safety: Strongly typed validation error structs generated at compile time.
1. Ergonomics: Derive macros and validator attributes that minimize boilerplate.
1. Developer Experience: Optional constructors, nested/newtype validation, and fluent/i18n hooks.

## Architecture Documentation Index

| Crate | Link to Architecture Doc | Purpose |
| ------------------------ | -------------------------------------------------------------- | ------------------------------------------------------------------------------------- |
| **Core** | | |
| `koruma` | [Architecture](crates/koruma/docs/ARCHITECTURE.md) | Public facade crate, re-exports core traits and derive macros, defines feature gates. |
| `koruma-core` | [Architecture](crates/koruma-core/docs/ARCHITECTURE.md) | Core traits and optional showcase registry types. |
| **Derive & Parsing** | | |
| `koruma-derive-core` | [Architecture](crates/koruma-derive-core/docs/ARCHITECTURE.md) | Attribute parsing and utilities shared by derive macros. |
| `koruma-derive` | [Architecture](crates/koruma-derive/docs/ARCHITECTURE.md) | Proc-macros for validators, error structs, and helper derives. |
| **Validator Collection** | | |
| `koruma-collection` | [Architecture](crates/koruma-collection/docs/ARCHITECTURE.md) | Built-in validators with optional fluent/i18n resources. |
| **Examples** | | |
| `examples/collection` | | Interactive TUI showcasing validators via `showcase`. |
| `examples/user-defined` | | Minimal example with custom validators and derives. |
| `examples/shared-lib` | | Workspace example sharing validators across crates. |
| `examples/i18n` | | Shared Fluent translation assets for examples. |

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

- **`examples/collection`**: Interactive TUI listing showcase-registered validators.
- **`examples/user-defined`**: Custom validator and derive usage.
- **`examples/shared-lib`**: Shared library example for cross-crate validation types.
- **`examples/i18n`**: Fluent locale files used by examples.

## Development

- **Rust**: Use `cargo` for building, testing, and running Rust code. In this workspace, prefer `workspace = true` inheritance syntax for dependencies in member crates' `Cargo.toml` files, with actual versions and features declared in the workspace root `Cargo.toml`. This pattern applies to both external crates and local workspace members reserve `path` syntax exclusively for the root `Cargo.toml` when defining workspace members; member crates should reference other workspace crates using `workspace = true` rather than explicit paths.
- For [fluent](https://projectfluent.org/) resources, install the [es-fluent](https://crates.io/crates/es-fluent-cli) cli to generate the resources.
- **Testing**: Use [insta](https://insta.rs/) for snapshot tests where appropriate, rather than complex assertion-based unit tests.
- **Test snippets**: Prefer raw multiline strings (or `quote! { ... }` in macro contexts) over escaped single-line literals when embedding Rust code in tests.
