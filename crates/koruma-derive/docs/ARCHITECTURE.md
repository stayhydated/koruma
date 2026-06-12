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
1. Generate error structs, `all()` enums, `validate()` implementations, optional `try_new` constructors, and optional `TryFrom<Inner>` impls for `try_from`.
1. Emit token streams via `expand/*` modules.

## Modules

- `expand/validator.rs`: generates Koruma-owned validator builders, direct setter entrypoints,
  `with_value`, hidden `CaptureValueRef` and `BuildValidator` glue, and optional showcase
  registration while preserving the validator's original bounds in the generated showcase impl.
  `ValidatorBuilderPlan` consumes `koruma-derive-core`'s typed validator-field spec for
  inferred or explicit value roles and `setter`/`setter(...)` metadata, then materializes distinct
  captured-value, skipped-value, required-setter, optional-setter, and defaulted-setter slots.
  Setter slots precompute
  `SetterSignature` values so generated builder methods and direct validator entrypoints share
  the same input type and value-conversion plan. Generated direct methods, `maybe_*` helpers,
  capture policy, `_state`, and required-state generics are planned and checked for collisions
  before code generation.
- `expand/error_bag.rs`: combines independent `syn::Error` values so parsing and planning can
  report multiple field, validator-name, and setter-argument diagnostics in one macro expansion.
- `showcase_modules.rs` (feature `internal-showcase`): generates the linker shim used for
  pulling in all showcase-annotated validators from `koruma-collection` without requiring
  per-module `pub mod` generation in macro output.
- `lib.rs` (feature `internal-showcase`): also exports the `showcase_module_enum!` macro that
  expands `koruma::showcase::ValidatorModule` in a shared crate feature pass.
- `expand/derive.rs`: generates error structs, `validate()`, `try_new`, `TryFrom`, nested/newtype handling, and element validator errors for `each(...)`.
- `expand/derive_field_errors.rs`: renders per-field error structs from the field-error render plan.
  `FieldErrorShape` encodes exact newtype, field-only, element-only, and combined field/element
  layouts before rendering. Field, element, and newtype validator docs, storage fields, getters,
  enum variants, and `all()` pushes are grouped through a `ValidatorGroupRenderPlan` so those
  generated surfaces stay in sync.
- `expand/plan.rs`: normalizes parsed metadata into struct shape, one planned field node that owns source field data, field shape, shared cardinality, typed `ValueShape` and `CollectionShape` data for `each(...)`, `ValidationTarget` decisions, generated API decisions, and render-ready validation/error plan nodes. Validation operations encode required vs. optional field handling and explicit collection/element iteration variants so renderers consume a single planned shape instead of recomputing those branches from raw field data.
- Field error storage is derived from `FieldPlan::shape` through render-plan methods; `FieldPlan` does not cache a separate storage classification that can disagree with the shape.
- `ValidationTarget`: models the four valid target shapes directly: full field, unwrapped field, full element, and unwrapped element. Each variant carries the raw and validation types plus explicit borrow behavior needed for that target, so validation rendering consumes planned access metadata instead of recomputing it from raw field data. Optional targets are unwrapped by default; `Validator::<Option<_>>` infers the full optional field or element target, `full(Validator::<_>)` selects it explicitly, and `unwrapped(Validator::<_>)` forces the default unwrapped target.
- `expand/display.rs`: implements `Display` for field and element validator enums.
- `expand/fluent.rs`: implements `FluentMessage` for validator enums and error structs (feature `fluent`).
- `expand/codegen.rs`: shared helpers for type resolution and argument transformations.
- `expand/generated_api.rs`: owns generated API namespace registration for error structs, validator
  reference enums, validator getter names, enum variants, validator builder types and modules,
  validator builder methods, optional `maybe_*` methods, and helper state generics so collision
  policy is shared before rendering.
- `expand/names.rs`: centralized construction of generated public names for error structs,
  validator reference enums, validator getter slots, enum variants, and tuple-field fallback
  identifiers. `GeneratedDeriveApi` registers derive output names before `ValidationPlan` hands
  them to renderers. Validator labels provide explicit getter and enum variant names; unlabeled
  validators keep the simple type-name stem only when it is unique in the field scope.

## Feature flags

- `fluent`: enables `KorumaAllFluent` and fluent codegen hooks.
- `internal-showcase`: emits validator registry metadata and uses derive-core showcase parsing.

## Tests

- Snapshot tests live under `crates/koruma-derive/src/tests` using `insta`.
