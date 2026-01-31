# Architecture: koruma-derive-core

## Purpose
`koruma-derive-core` owns the parsing layer for `#[koruma(...)]` attributes. It provides a stable API for derive macros and external tooling to inspect koruma validation metadata without depending on the proc-macro crate.

## Modules
- `crates/koruma-derive-core/src/parse.rs`: parsing logic and data types for validators and attributes.
- `crates/koruma-derive-core/src/utils.rs`: type helpers (Option/Vec inference, turbofish substitution).
- `crates/koruma-derive-core/src/tests`: snapshot coverage for parsing behavior.

## Data model
- `ValidatorAttr`: a single validator invocation (path, args, type inference flags).
- `KorumaAttr`: a field attribute grouping validators plus modifiers (`each`, `nested`, `newtype`, `skip`).
- `FieldInfo`: per-field metadata derived from `syn::Field`.
- `StructOptions`: struct-level flags like `try_new` and `newtype`.

## Parsing notes
- `parse_field` respects `cfg_attr` via `syn-cfg-attr` helpers.
- Turbofish syntax (`Validator::<_>`) drives type inference and substitution.
- `find_value_field` locates `#[koruma(value)]` for validator structs.

## Feature flags
- `showcase`: parses `#[showcase(...)]` metadata used for validator registries.

## Tests
- Snapshot tests under `crates/koruma-derive-core/src/tests` validate parser output.
