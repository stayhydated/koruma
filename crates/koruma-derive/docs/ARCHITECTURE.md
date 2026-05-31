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
1. Build parsed field metadata plus struct-level `StructOptions`, then normalize them into a `ValidationPlan` with one ordered `FieldPlan` per participating field.
1. Generate error structs, `all()` enums, `validate()` implementations, optional `try_new` constructors, and optional `TryFrom<Inner>` impls for `newtype(try_from)`.
1. Emit token streams via `expand/*` modules.

## Modules

- `expand/validator.rs`: generates Koruma-owned validator builders, direct setter entrypoints,
  `with_value`, the hidden `CaptureValueRef` capture hook, and optional showcase registration
  while preserving the validator's original bounds in the generated showcase impl. Direct setter
  projection uses
  field-level `#[koruma(setter(...))]` metadata with `into`, `required`, `name`, and `default`
  options.
- `expand/error_bag.rs`: combines independent `syn::Error` values so parsing and planning can
  report multiple field, validator-name, and setter-argument diagnostics in one macro expansion.
- `showcase_modules.rs` (feature `internal-showcase`): generates the linker shim used for
  pulling in all showcase-annotated validators from `koruma-collection` without requiring
  per-module `pub mod` generation in macro output.
- `lib.rs` (feature `internal-showcase`): also exports the `showcase_module_enum!` macro that
  expands `koruma::showcase::ValidatorModule` in a shared crate feature pass.
- `expand/derive.rs`: generates error structs, `validate()`, `try_new`, `TryFrom`, nested/newtype handling, and element validator errors for `each(...)`.
- `expand/plan.rs`: normalizes parsed metadata into struct shape, one planned field node that owns source field data, field shape, field cardinality, typed `each(...)` collection classification, `TargetPlan` validation target decisions, and generated-name decisions before rendering. Renderers iterate the planned field list directly rather than zipping separate parser and planner vectors.
- `TargetPlan`: centralizes whether a validator targets a field or element, whether it receives the full optional target or the unwrapped inner value, the raw and validate types, optional cardinality, and how generated validation code should borrow the target expression.
- `expand/display.rs`: implements `Display` for field and element validator enums.
- `expand/fluent.rs`: implements `FluentMessage` for validator enums and error structs (feature `fluent`).
- `expand/codegen.rs`: shared helpers for type resolution and argument transformations.
- `expand/names.rs`: centralized construction of generated public names for error structs,
  validator reference enums, validator getter slots, enum variants, and tuple-field fallback
  identifiers. Validator labels provide explicit getter and enum variant names; unlabeled
  validators keep the simple type-name stem only when it is unique in the field scope.

## Feature flags

- `fluent`: enables `KorumaAllFluent` and fluent codegen hooks.
- `internal-showcase`: emits validator registry metadata and uses derive-core showcase parsing.

## Tests

- Snapshot tests live under `crates/koruma-derive/src/tests` using `insta`.
