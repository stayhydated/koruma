# Architecture: koruma-derive

## Purpose

`koruma-derive` is the proc-macro crate that generates validation code and helper derives. It owns the macro entry points and delegates attribute parsing to `koruma-derive-core`.

## Entry points

- `crates/koruma-derive/src/lib.rs`:
  - `#[koruma::validator]` attribute macro
  - `#[derive(Koruma)]` derive
  - `#[derive(KorumaAllDisplay)]` derive
  - `#[derive(KorumaAllFluent)]` derive (feature `fluent`)

## Expansion pipeline

1. Parse input with `syn`.
1. Parse `#[koruma(...)]` metadata via `koruma-derive-core`.
1. Build per-field `FieldInfo` plus struct-level `StructOptions`, then normalize them into a `ValidationPlan`.
1. Generate error structs, `all()` enums, `validate()` implementations, optional `try_new` constructors, and optional `TryFrom<Inner>` impls for `newtype(try_from)`.
1. Emit token streams via `expand/*` modules.

## Modules

- `expand/validator.rs`: adds hidden `bon` builder plumbing, direct setter entrypoints,
  `with_value`, hidden `with_value_ref`, and optional showcase registration while preserving the
  validator's original bounds in the generated showcase impl. Direct setter projection is a
  koruma-owned subset of field-level `#[builder(...)]`: `into`, `required`, `name`, and `default`
  are supported; unsupported builder keys are rejected instead of silently mirroring more of `bon`.
- `showcase_modules.rs` (feature `internal-showcase`): generates the linker shim used for
  pulling in all showcase-annotated validators from `koruma-collection` without requiring
  per-module `pub mod` generation in macro output.
- `lib.rs` (feature `internal-showcase`): also exports the `showcase_module_enum!` macro that
  expands `koruma::showcase::ValidatorModule` in a shared crate feature pass.
- `expand/derive.rs`: generates error structs, `validate()`, `try_new`, `TryFrom`, nested/newtype handling, and element validator errors for `each(...)`.
- `expand/plan.rs`: normalizes parsed metadata into struct shape, field shape, field cardinality, typed `each(...)` collection classification, validation site, target policy, and generated-name decisions before rendering.
- `expand/display.rs`: implements `Display` for field and element validator enums.
- `expand/fluent.rs`: implements `FluentMessage` for validator enums and error structs (feature `fluent`).
- `expand/codegen.rs`: shared helpers for type resolution and argument transformations.

## Feature flags

- `fluent`: enables `KorumaAllFluent` and fluent codegen hooks.
- `internal-showcase`: emits validator registry metadata and uses derive-core showcase parsing.

## Tests

- Snapshot tests live under `crates/koruma-derive/src/tests` using `insta`.
