# Architecture: koruma-derive-core

## Purpose

`koruma-derive-core` owns the parsing layer for `#[koruma(...)]` attributes. It provides a stable API for derive macros and external tooling to inspect koruma validation metadata without depending on the proc-macro crate.

## Modules

- `crates/koruma-derive-core/src/parse.rs`: parsing logic and data types for validators, field options, struct options, and showcase metadata.
- `crates/koruma-derive-core/src/utils.rs`: type helpers (Option/Vec inference and placeholder substitution).
- `crates/koruma-derive-core/src/tests`: snapshot coverage for parsing behavior.

## Data model

- `ValidatorAttr`: a single validator invocation (path, shorthand args or builder setter calls, type inference flags, and normalized setter access via `setter_calls()`).
- `KorumaAttr`: a field attribute grouping validators plus modifiers (`each`, `nested`, `newtype`, `skip`).
- `ValidationInfo`: merged validators and modifier flags for a field.
- `FieldInfo`: per-field metadata derived from `syn::Field`.
- `StructOptions`: struct-level flags like `try_new`, `newtype`, and `newtype(try_from)`.
- `ValueFieldInfo` / `ValueFieldCapture`: metadata for the validator field marked `#[koruma(value)]`, including whether capture uses the normal borrowed-value path or `skip_capture`.
- `ShowcaseAttr` (feature `internal-showcase`): parsed `#[showcase(...)]` metadata, including required explicit `input_type`.

## Parsing notes

- `parse_field` merges multiple `#[koruma(...)]` attributes, handles `skip`, `nested`, `newtype`, and `each(...)`, and detects duplicate validators.
- `parse_struct_options` reads struct-level `#[koruma(...)]` options (`try_new`, `newtype`, `newtype(try_from)`).
- `parse_field` respects `cfg_attr` via `syn-cfg-attr` helpers.
- Generic validator bindings support shorthand angle brackets (`Validator<_>`) and standard Rust builder chains (`Validator::<_>::builder().min(...)`) for type inference and substitution.
- `find_value_field*` helpers locate `#[koruma(value)]` for validator structs and validate `skip_capture` usage.
- `find_showcase_attr` (feature `internal-showcase`) parses showcase metadata on validators and rejects missing or invalid `input_type`.

## Feature flags

- `internal-showcase`: enables parsing of `#[showcase(...)]` metadata used for validator registries.

## Tests

- Snapshot tests under `crates/koruma-derive-core/src/tests` validate parser output.
