# Architecture: koruma-core

## Purpose

`koruma-core` defines the foundational traits used by validators, derived validation error structs, and nested/newtype validation. It intentionally avoids proc-macro or validator implementations so other crates can depend on it with minimal overhead.

## Modules

- `crates/koruma-core/src/lib.rs`: trait definitions, hidden macro glue under
  `__private`, and the optional `showcase` module.
- `crates/koruma-core/src/lib.rs::showcase` (feature-gated): type-erased registry types backed by `inventory`.

## Core traits

- `Validate<T>`: implemented by validator structs; returns `true`/`false`.
- `ValidationError`: implemented by generated error structs; supplies `is_empty()` and `has_errors()`.
- `ValidateExt`: implemented by `#[derive(Koruma)]` for nested/newtype validation.
- `NewtypeValidation`: marker for newtype structs with transparent error access.

## Hidden macro glue

- `__private::BuildValidator`: hidden trait implemented for ready generated validator-builder
  states so generated validation code has a typed build boundary.
- `__private::CaptureValueRef<T>`: hidden borrowed-value hook used by derived validation to
  apply field values to validator builders while letting generated capture-policy impls decide
  whether to clone or skip capture. Its output must implement `BuildValidator`.

## Showcase registry (feature `internal-showcase`)

- `DynValidator`: type-erased validator interface. Showcase impls preserve the
  validator's original generic bounds, require `Display` for user-facing text,
  and only emit Fluent hooks when the macro expansion has fluent support.
- `InputType`: expected input classification (text or numeric).
- `ValidatorShowcase`: metadata collected via `inventory`.
- `validators()`: returns all registered validators.
- `ValidatorModule`: generated via `koruma_derive::showcase_module_enum!(...)` under
  `internal-showcase` to keep module names centralized with validator declarations.

## Control flow

- Validator structs implement `Validate<T>`.
- Derive macros emit error types implementing `ValidationError` and `ValidateExt`.
- `#[koruma::validator]` emits inherent `with_value(...)` methods for validator
  builders that capture the validated value.
- Derived validation code reaches these hidden hooks through the facade path
  `koruma::__private`, which re-exports `koruma_core::__private`.
- Nested/newtype validation relies on `ValidateExt::Error` for typed error state.

## Validator Error Model

Koruma currently keeps the failed validator instance as the typed error payload:
`Validate<T>` returns `bool`, and derived validation stores the configured validator when validation
fails. This preserves concrete accessor types, `all()` borrowing, `Display`, and Fluent rendering
without introducing a second associated error type on every validator. Validators that do not
render or expose the failing input should use `#[koruma(value(capture = skip))]` on an `Option<T>`
value field to avoid clone requirements for non-`Clone` payloads.

## Feature flags

- `internal-showcase`: enables the showcase registry types, `inventory` integration, and
  `ValidatorModule` generation.

## Tests

- Unit tests live under `crates/koruma-core/tests/` and cover core trait behavior.
