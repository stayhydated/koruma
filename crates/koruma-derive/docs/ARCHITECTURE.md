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
1. Build `FieldInfo` and `StructOptions` describing validation intent.
1. Emit token streams via `expand/*` modules.

## Modules

- `expand/validator.rs`: adds `bon` builders, `with_value`, and showcase registration.
- `expand/derive.rs`: generates error structs, accessors, `validate()` impls, and `try_new` when requested.
- `expand/display.rs`: implements `Display` for the `all()` validator enums.
- `expand/fluent.rs`: implements `ToFluentString` for the `all()` validator enums (feature `fluent`).
- `expand/codegen.rs`: shared helpers for type resolution and argument transformations.

## Feature flags

- `fluent`: enables `KorumaAllFluent` and fluent codegen hooks.
- `showcase`: emits validator registry metadata and uses derive-core showcase parsing.

## Tests

- Snapshot tests live under `crates/koruma-derive/src/tests` using `insta`.
