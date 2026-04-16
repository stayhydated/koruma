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
1. Build per-field `FieldInfo` plus struct-level `StructOptions` (`try_new`, `newtype`).
1. Generate error structs, `all()` enums, `validate()` implementations, and optional `try_new` constructors.
1. Emit token streams via `expand/*` modules.

## Modules

- `expand/validator.rs`: adds `bon` builders, `with_value`, and optional
  showcase registration while preserving the validator's original bounds in the
  generated showcase impl.
- `expand/derive.rs`: generates error structs, `validate()`, `try_new`, nested/newtype handling, and element validator errors for `each(...)`.
- `expand/display.rs`: implements `Display` for field and element validator enums.
- `expand/fluent.rs`: implements `ToFluentString` for validator enums and error structs (feature `fluent`).
- `expand/codegen.rs`: shared helpers for type resolution and argument transformations.

## Feature flags

- `fluent`: enables `KorumaAllFluent` and fluent codegen hooks.
- `internal-showcase`: emits validator registry metadata and uses derive-core showcase parsing.

## Tests

- Snapshot tests live under `crates/koruma-derive/src/tests` using `insta`.
