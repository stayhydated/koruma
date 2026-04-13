# AGENTS.md

This file is the working guide for contributors and coding agents in the `koruma` workspace.

Use it to answer three questions quickly:

1. Where does this documentation belong?
2. Which crates are the default entry points vs integration points vs internals?
3. What other surfaces must be updated in the same change?

## Project summary

`koruma` is a Rust validation ecosystem centered on per-field validation.

Its priorities are:

1. **Type safety**: keep validators, derived error types, and validation flows strongly typed.
2. **Ergonomics**: make validator definitions and field annotations concise.
3. **Developer experience**: support optional constructors, nested/newtype validation, built-in validators, and i18n.

For most application code, start with `crates/koruma`.

Reach for `crates/koruma-collection` when you want built-in validators instead of defining your own.

## Audience labels

These labels describe the crate or surface itself, not the documentation file you are editing:

- **User-facing**: normal entry points for application developers.
- **Public integration**: public crates meant for extensions, tooling, or deeper customization, but not usually the default starting point.
- **Internal**: workspace plumbing, implementation detail, demos, and maintenance tooling.

## Documentation rules

### User-facing documentation

These surfaces are user-facing:

- every `README.md` in the workspace,
- the root `README.md`,
- crate-level `README.md` files,
- the mdBook under `book/`,
- the public site under `web/`.

Even for public-integration or internal crates, a `README.md` should explain:

- who the crate is for,
- what it does,
- what most users should use instead.

### Internal documentation

Only `docs/ARCHITECTURE.md` files are internal documentation.

Use them for:

- macro expansion and parsing details,
- subsystem boundaries,
- data flow,
- design rationale,
- internal relationships.

Do not put implementation detail into READMEs or the book.

## Synchronization rules

When changing a public workflow, feature flag story, validator inventory, validator message shape, or user-visible API shape:

1. Update `examples/readme` when relevant.
2. Update the affected user-facing `README.md` files.
3. Update the matching `book/src/*.md` pages.
4. Keep these surfaces aligned in the same change unless there is a documented reason not to.

Additional rules:

- User-facing documentation should be example-first.
- Prefer a Rust snippet over prose-only explanations when showing behavior changes.
- `examples/readme` is the canonical source of truth for usage examples.
- Keep `crates/koruma-collection/README.md` and `book/src/koruma_collection.md` synchronized when validator inventory, feature flags, or usage guidance changes.

## Workspace map

### Main user-facing entry points

- `crates/koruma`
  Audience: **User-facing**
  Docs: [Architecture](crates/koruma/docs/ARCHITECTURE.md)
  Role: workspace facade, default entry point, and home of the public feature gates. Re-exports core traits, derive macros, and the `bon` builder API.

- `crates/koruma-collection`
  Audience: **User-facing**
  Docs: [Architecture](crates/koruma-collection/docs/ARCHITECTURE.md)
  Role: curated validator library organized by domain (`string`, `format`, `numeric`, `collection`, `general`) with optional Fluent-based i18n.

### Public integration crates

- `crates/koruma-core`
  Audience: **Public integration**
  Docs: [Architecture](crates/koruma-core/docs/ARCHITECTURE.md)
  Role: foundational validation traits, validation error interfaces, nested/newtype support, and optional showcase registry types. Most application users should start with `koruma` instead.

- `crates/koruma-derive`
  Audience: **Public integration**
  Docs: [Architecture](crates/koruma-derive/docs/ARCHITECTURE.md)
  Role: proc-macro crate for `#[derive(Koruma)]`, `KorumaAllDisplay`, `KorumaAllFluent`, and `#[koruma::validator]`. Most users should depend on `koruma` instead of this crate directly.

- `crates/koruma-derive-core`
  Audience: **Public integration**
  Docs: [Architecture](crates/koruma-derive-core/docs/ARCHITECTURE.md)
  Role: parsing layer for `#[koruma(...)]` metadata shared by derive macros and tooling. Most application users should not depend on it directly.

### Internal tooling

- `xtask`
  Audience: **Internal**
  Docs: [Architecture](xtask/docs/ARCHITECTURE.md)
  Role: workspace maintenance tooling.

  Key commands:

  - `sync-display-ftl`: syncs English FTL message templates with `Display` implementations in `koruma-collection` validators.
  - `build-book`: builds the mdBook into `web/public/book`.
  - `build-llms-txt`: concatenates mdBook sources into `web/public/llms.txt`.

### Examples and web surfaces

- `examples/readme`
  Canonical executable documentation examples. Keep this aligned with the root `README.md` and the book.

- `examples/shared-lib`
  Shared example library used by the documentation example and showcase demos.

- `examples/i18n`
  Shared Fluent translation assets used by the examples.

- `examples/collection-ratatui-core`
  Shared ratatui showcase logic used by the native and web demos.

- `examples/collection-ratatui-native`
  Native ratatui showcase app for browsing registered validators.

- `examples/collection-ratatui-web`
  WebAssembly ratatui showcase app.

- `examples/collection-dioxus-web`
  Dioxus-based web showcase app.

- `web`
  Audience: **User-facing**
  Role: Astro-based GitHub Pages site hosting demos and the mdBook.

- `book`
  Audience: **User-facing**
  Role: mdBook for public workflows, validator usage, nested/newtype patterns, i18n integration, and `koruma-collection`.

## Working rules by change type

### When editing docs

- Keep READMEs and the book user-facing.
- Move parsing internals, macro expansion details, and subsystem design into `docs/ARCHITECTURE.md`.
- Prefer examples over prose-only explanations.
- Sync `examples/readme`, relevant READMEs, and book pages in the same change.
- For validator catalog or feature-flag changes, update both `crates/koruma-collection/README.md` and `book/src/koruma_collection.md`.

### When editing Rust crates

- Use `cargo` for build, test, and run tasks.
- Keep dependency versions in the workspace root `Cargo.toml`.
- Use `workspace = true` in member crates.
- Let each crate choose its own dependency features in its own `Cargo.toml`.
- Use `path` dependencies only in the root `Cargo.toml` and in examples.
- Non-example crates should reference workspace crates with `workspace = true`, not explicit paths.

### When editing validators or validator messages

- Add validators under `crates/koruma-collection/src/validators/` and re-export them from the appropriate module.
- Add or update localized messages under `crates/koruma-collection/i18n/` when Fluent support is in use.
- Keep English FTL message templates and `Display` implementations aligned.
- Keep showcase metadata and showcase demos aligned when `internal-showcase` behavior changes.

### When writing tests

- Prefer [insta](https://insta.rs/) for snapshot tests when it fits better than assertion-heavy unit tests.
- Prefer raw multiline strings, or `quote! { ... }` in macro contexts, over escaped single-line literals for embedded Rust code.

### When editing JavaScript or web tooling

- Use [bun](https://bun.com/) for dependency management.
- Use [turborepo](https://turborepo.org/) as the build system.
