# AGENTS.md

This is the working guide for contributors and coding agents in the `koruma`
workspace.

Use it to decide:

1. where documentation belongs,
2. whether a crate or surface is user-facing, public integration, or internal,
3. which related docs, examples, and skills must change together,
4. which validation command should run before handoff.

For most application code, start with `crates/koruma`.

Reach for `crates/koruma-collection` when you want built-in validators instead
of defining your own.

## Project Summary

`koruma` is a Rust validation ecosystem centered on per-field validation.

Its priorities are:

1. **Type safety**: keep validators, derived error types, and validation flows strongly typed.
2. **Ergonomics**: make validator definitions and field annotations concise.
3. **Developer experience**: support optional constructors, nested and newtype validation, built-in validators, and i18n.

## Quick Decision Flow

Before editing, classify the change:

1. **Find the surface in the workspace map.** Use its audience label to decide
   how much public explanation the change needs.
2. **Place documentation by content, not by crate audience.** README files, the
   book, and the public site are always user-facing. Internal design belongs in
   the matching `docs/ARCHITECTURE.md`.
3. **Sync public workflow changes.** If behavior, validator inventory, feature
   flags, message shape, generated output, or recommended usage changes, update
   the relevant example, README, book page, and public `skills/*` guidance in
   the same change when applicable.
4. **Validate narrowly.** Run the smallest command that proves the edited
   behavior or documentation surface is still sound.

## Audience Labels

These labels describe the crate or surface itself, not the documentation file
being edited:

- **User-facing**: normal entry points for application developers.
- **Public integration**: public crates meant for extensions, tooling, or
  deeper customization. These are usually not the default starting point.
- **Internal**: workspace plumbing, implementation details, demos, and maintenance tooling.

## Documentation Placement

### User-Facing Documentation

Treat these surfaces as user-facing:

- every `README.md` in the workspace,
- the mdBook under `book/`,
- the public site under `web/`.

Even README files for public-integration or internal crates should explain:

- who the crate is for,
- what it does,
- what most users should use instead.

Keep user-facing documentation example-first. Prefer Rust snippets over
prose-only explanations when showing behavior changes.

### Internal Documentation

Use the relevant `docs/ARCHITECTURE.md` file for internal documentation, such
as the crate-level paths listed in the workspace map.

Keep these topics in architecture documents, not in READMEs or the book:

- implementation details,
- macro expansion and parsing details,
- subsystem boundaries,
- data flow,
- design rationale,
- internal relationships.

### Skill Guidance

`skills/use-koruma` is the public reusable skill for application developers. It
must not include internal wording, maintainer-only language, repo-private
assumptions, or implementation details.

Do not assume root-level `skills/*` entries are auto-loaded as repo-local Codex
skills. Treat `skills/*` as public distribution sources.

Update relevant in-repository skill guidance when a code change alters
user-facing workflows, validator behavior, feature flags, generated output, i18n
integration patterns, or recommended usage.

## Synchronization Rules

When a substantive change modifies a public workflow, public feature, feature
flag story, validator inventory, validator message shape, or user-visible API
shape:

1. Update the executable example in `examples/readme` when relevant.
2. Update the affected user-facing `README.md` files.
3. Update the matching `book/src/*.md` pages.
4. Update relevant public `skills/*` guidance.
5. Keep these surfaces aligned in the same change unless there is a documented reason not to.

`examples/readme` is the canonical source of truth for usage examples.

Keep `crates/koruma-collection/README.md` and
`book/src/koruma_collection.md` synchronized when validator inventory, feature
flags, or usage guidance changes.

## Workspace Map

### Main User-Facing Entry Points

- `crates/koruma`
  Audience: **User-facing**
  Docs: [Architecture](crates/koruma/docs/ARCHITECTURE.md)
  Role: workspace facade, default entry point, and home of the public feature gates. Re-exports core traits, derive macros, and the `bon` builder API.

- `crates/koruma-collection`
  Audience: **User-facing**
  Docs: [Architecture](crates/koruma-collection/docs/ARCHITECTURE.md)
  Role: curated validator library organized by domain (`string`, `format`, `numeric`, `collection`, `general`) with optional Fluent-based i18n.

### Public Integration Crates

- `crates/koruma-core`
  Audience: **Public integration**
  Docs: [Architecture](crates/koruma-core/docs/ARCHITECTURE.md)
  Role: foundational validation traits, validation error interfaces, nested and newtype support, and optional showcase registry types. Most application users should start with `koruma` instead.

- `crates/koruma-derive`
  Audience: **Public integration**
  Docs: [Architecture](crates/koruma-derive/docs/ARCHITECTURE.md)
  Role: proc-macro crate for `#[derive(Koruma)]`, `KorumaAllDisplay`, `KorumaAllFluent`, and `#[koruma::validator]`. Most users should depend on `koruma` instead of this crate directly.

- `crates/koruma-derive-core`
  Audience: **Public integration**
  Docs: [Architecture](crates/koruma-derive-core/docs/ARCHITECTURE.md)
  Role: parsing layer for `#[koruma(...)]` metadata shared by derive macros and tooling. Most application users should not depend on it directly.

### Internal Crates and Tooling

- `xtask`
  Audience: **Internal**
  Docs: [Architecture](xtask/docs/ARCHITECTURE.md)
  Role: workspace maintenance tooling.

  Key commands:
  - `sync-display-ftl`: syncs English FTL message templates with `Display` implementations in `koruma-collection` validators.
  - `build-book`: builds the mdBook into `web/public/book`.
  - `build-llms-txt`: concatenates mdBook sources into `web/public/llms.txt`.

### Examples and Web Surfaces

- `examples/readme`
  Canonical executable documentation examples. Keep this aligned with the root `README.md` and the book.

- `examples/shared-lib`
  Shared example library used by the documentation example and showcase demos.

- `examples/i18n`
  Shared Fluent translation assets used by the examples.

- `web`
  Audience: **User-facing**
  Role: Dioxus-based GitHub Pages site hosting demos and the mdBook.

- `book`
  Audience: **User-facing**
  Role: mdBook for public workflows, validator usage, nested and newtype patterns, i18n integration, and `koruma-collection`.

## Validation and Editing Rules

### Validation After Changes

- Validation is the default after code or workflow changes.
- Run the narrowest command that proves the edited behavior works for the
  affected crate, docs, example, or web surface.
- Prefer targeted crate, example, docs, or web checks before full-workspace validation.
- Use `just check`, `just test`, or a more specific `justfile` recipe when the change spans multiple surfaces.
- If validation cannot be run, state why and what remains unvalidated.
- Do not claim a change works unless it was validated, generated from a source of truth, or the remaining risk is explicitly documented.

### When Editing Docs

- Keep READMEs, the book, and the public site user-facing.
- Move parsing internals, macro expansion details, and subsystem design into `docs/ARCHITECTURE.md`.
- Prefer examples over prose-only explanations.
- Sync `examples/readme`, relevant READMEs, book pages, and public `skills/*` guidance in the same change.
- For validator catalog or feature-flag changes, update both `crates/koruma-collection/README.md` and `book/src/koruma_collection.md`.

### When Editing Rust Crates

- Use `cargo` for build, test, and run tasks.
- Keep dependency versions in the workspace root `Cargo.toml`.
- Use `workspace = true` in member crates.
- Let each crate choose its own dependency features in its own `Cargo.toml`.
- Use `path` dependencies only in the root `Cargo.toml` and in examples.
- Non-example crates should reference workspace crates with `workspace = true`, not explicit paths.

### When Editing Validators or Validator Messages

- Add validators under `crates/koruma-collection/src/validators/` and re-export them from the appropriate module.
- Add or update localized messages under `crates/koruma-collection/i18n/` when Fluent support is in use.
- Keep English FTL message templates and `Display` implementations aligned.
- Keep showcase metadata and showcase demos aligned when `internal-showcase` behavior changes.

### When Writing Tests

- Prefer [insta](https://insta.rs/) for snapshot tests when it fits better than assertion-heavy unit tests.
- Prefer raw multiline strings, or `quote! { ... }` in macro contexts, over escaped single-line literals for embedded Rust code.

### When Editing JavaScript/Typescript

- Use [bun](https://bun.com/) for dependency management.
- Use [turborepo](https://turborepo.org/) as the build system.
