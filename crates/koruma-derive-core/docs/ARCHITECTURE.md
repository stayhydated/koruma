# Architecture: koruma-derive-core

## Purpose

`koruma-derive-core` owns the parsing layer for `#[koruma(...)]` attributes. It provides a stable API for derive macros and external tooling to inspect koruma validation metadata without depending on the proc-macro crate.

## Modules

- `crates/koruma-derive-core/src/parse.rs`: parsing logic and data types for validators, field options, struct options, and showcase metadata.
- `crates/koruma-derive-core/src/utils.rs`: type helpers (Option/Vec inference and placeholder substitution).
- `crates/koruma-derive-core/src/tests`: snapshot coverage for parsing behavior.

## Data model

- `ValidatorAttr`: a single direct validator chain (path, setter calls, type inference flags, and setter access via `setter_calls()`).
- `ParsedValidatorUse`: a single validator occurrence on a data field. It carries the parsed validator, an optional label slot for label-aware naming, and the source span used for diagnostics.
- `StructKorumaAttr` / `StructKorumaItem`: struct-level `#[koruma(...)]` grammar for `try_new`, `newtype`, and `newtype(try_from)`.
- `DataFieldKorumaAttr` / `DataFieldKorumaItem`: data-field `#[koruma(...)]` grammar for modifiers, direct field validators, optional `label = Validator` labels, and `each(...)` element validators without a raw token bucket.
- `ValidatorFieldKorumaItem`: validator-field `#[koruma(...)]` grammar for `value`,
  `value(capture = skip)`, and `setter(...)`.
- `ParsedFieldSpec`: normalized field shape for participating fields. It is an enum with `Regular`, `Nested`, `Newtype`, and `Skipped` variants so parser output cannot encode skipped-with-validator, nested-with-validator, or newtype-with-element-validator states.
- `FieldInfo`: per-field metadata derived from `syn::Field`.
- `StructOptions` / `StructConstructor`: normalized struct-level newtype and constructor intent for `try_new` and `newtype(try_from)`.
- `ValueFieldInfo` / `CapturePolicy`: metadata for the validator field marked `#[koruma(value)]`,
  including whether capture clones the borrowed input or uses `capture = skip`.
- `ShowcaseAttr` (feature `internal-showcase`): parsed `#[showcase(...)]` metadata, including required explicit `input_type`.

## Parsing notes

- `parse_field` parses data-field attributes with `DataFieldKorumaAttr`, merges multiple `#[koruma(...)]` attributes into `ParsedFieldSpec`, handles `skip`, `nested`, `newtype`, and `each(...)`, and returns `Result<Option<FieldInfo>, syn::Error>` so unannotated and skipped fields use the standard `None` path. Duplicate generated-name diagnostics are handled by the derive planning layer where the full field scope is known.
- `parse_struct_options` parses struct-level attributes with `StructKorumaAttr` and normalizes `try_new`, `newtype`, and `newtype(try_from)` into `StructOptions`.
- `parse_field` respects `cfg_attr` via `syn-cfg-attr` helpers.
- Generic validator bindings use standard Rust direct validator chains (`Validator::<_>::min(...)`) for type inference and substitution.
- `find_value_field_strict` and `find_value_field_info_strict` parse validator-field attributes separately from data-field attributes, locate `#[koruma(value)]`, and validate capture policy usage.
- `find_showcase_attr` (feature `internal-showcase`) parses showcase metadata on validators and rejects missing or invalid `input_type`.

## Feature flags

- `internal-showcase`: enables parsing of `#[showcase(...)]` metadata used for validator registries.

## Tests

- Snapshot tests under `crates/koruma-derive-core/src/tests` validate parser output.
