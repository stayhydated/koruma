# Architecture: koruma-core

## Purpose

`koruma-core` defines the foundational traits used by validators, derived validation error structs, and nested/newtype validation. It intentionally avoids proc-macro or validator implementations so other crates can depend on it with minimal overhead.

## Modules

- `crates/koruma-core/src/lib.rs`: trait definitions and the optional `showcase` module.
- `crates/koruma-core/src/lib.rs::showcase` (feature-gated): type-erased registry types backed by `inventory`.

## Core traits

- `Validate<T>`: implemented by validator structs; returns `true`/`false`.
- `ValidationError`: implemented by generated error structs; supplies `is_empty()`.
- `BuilderWithValue<T>`: implemented by `#[koruma::validator]` builders.
- `ValidateExt`: implemented by `#[derive(Koruma)]` for nested validation.
- `NewtypeValidation`: marker for newtype structs with transparent error access.

## Control flow

- Validator structs implement `Validate<T>`.
- Derive macros emit error types implementing `ValidationError` and `ValidateExt`.
- Nested/newtype validation relies on `ValidateExt::Error` to carry typed error state.

## Feature flags

- `showcase`: enables the showcase registry types and `inventory` integration.

## Tests

- Unit tests live under `crates/koruma-core/tests/` and cover core trait behavior
  (`Validate`, `ValidationError`, `BuilderWithValue`).
