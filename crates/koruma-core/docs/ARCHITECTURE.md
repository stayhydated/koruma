# Architecture: koruma-core

## Purpose

`koruma-core` defines the foundational traits used by validators, derived validation error structs, and nested/newtype validation. It intentionally avoids proc-macro or validator implementations so other crates can depend on it with minimal overhead.

## Modules

- `crates/koruma-core/src/lib.rs`: trait definitions and the optional `showcase` module.
- `crates/koruma-core/src/lib.rs::showcase` (feature-gated): type-erased registry types backed by `inventory`.

## Core traits

- `Validate<T>`: implemented by validator structs; returns `true`/`false`.
- `ValidationError`: implemented by generated error structs; supplies `is_empty()` and `has_errors()`.
- `BuilderWithValue<T>`: implemented by `#[koruma::validator]` builders.
- `BuilderWithValueRef<T>`: hidden borrowed-value hook used by derived validation to pass field values into builders while letting validators decide whether to clone or skip capture.
- `ValidateExt`: implemented by `#[derive(Koruma)]` for nested/newtype validation.
- `NewtypeValidation`: marker for newtype structs with transparent error access.

## Showcase registry (feature `internal-showcase`)

- `DynValidator`: type-erased validator interface. Showcase impls preserve the
  validator's original generic bounds, require `Display` for user-facing text,
  and only emit Fluent hooks when the macro expansion has fluent support.
- `InputType`: expected input classification (text or numeric).
- `ValidatorShowcase`: metadata collected via `inventory`.
- `validators()`: returns all registered validators.

## Control flow

- Validator structs implement `Validate<T>`.
- Derive macros emit error types implementing `ValidationError` and `ValidateExt`.
- Derived validation code uses `BuilderWithValueRef` to feed borrowed field values into validator builders.
- Nested/newtype validation relies on `ValidateExt::Error` for typed error state.

## Feature flags

- `internal-showcase`: enables the showcase registry types and `inventory` integration.

## Tests

- Unit tests live under `crates/koruma-core/tests/` and cover core trait behavior.
