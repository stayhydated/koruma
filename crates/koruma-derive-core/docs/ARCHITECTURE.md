# Architecture: koruma-derive-core

## Purpose

`koruma-derive-core` owns the parsing layer for `#[koruma(...)]` attributes. It provides a stable API for derive macros and external tooling to inspect koruma validation metadata without depending on the proc-macro crate.

## Modules

- `crates/koruma-derive-core/src/parse.rs`: parsing logic and data types for validators, field options, struct options, and showcase metadata.
- `crates/koruma-derive-core/src/utils.rs`: type helpers (Option/Vec inference and placeholder substitution).
- `crates/koruma-derive-core/src/tests`: snapshot coverage for parsing behavior.

## Data model

- `ValidatorAttr`: a single validator invocation (path, args, type inference flags).
- `KorumaAttr`: a field attribute grouping validators plus modifiers (`each`, `nested`, `newtype`, `skip`).
- `ValidationInfo`: merged validators and modifier flags for a field.
- `FieldInfo`: per-field metadata derived from `syn::Field`.
- `StructOptions`: struct-level flags like `try_new` and `newtype`.
- `ShowcaseAttr` (feature `internal-showcase`): parsed `#[showcase(...)]` metadata.

## Parsing notes

- `parse_field` merges multiple `#[koruma(...)]` attributes, handles `skip`, `nested`, `newtype`, and `each(...)`, and detects duplicate validators.
- `parse_struct_options` reads struct-level `#[koruma(...)]` options (`try_new`, `newtype`).
- `parse_field` respects `cfg_attr` via `syn-cfg-attr` helpers.
- Generic validator bindings use shorthand angle brackets (`Validator<_>`) for type inference and substitution.
- `find_value_field` locates `#[koruma(value)]` for validator structs.
- `find_showcase_attr` (feature `internal-showcase`) parses showcase metadata on validators.

## Feature flags

- `internal-showcase`: enables parsing of `#[showcase(...)]` metadata used for validator registries.

## Tests

- Snapshot tests under `crates/koruma-derive-core/src/tests` validate parser output.
