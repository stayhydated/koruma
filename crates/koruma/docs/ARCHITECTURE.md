# Architecture: koruma

## Purpose
`koruma` is the public facade crate. It re-exports the core traits and the derive macros, and provides feature gates that let consumers opt into fluent/i18n or showcase support without pulling those dependencies by default.

## Key entry points
- `crates/koruma/src/lib.rs`: re-exports, feature gating, and README docs (`include_str!`).

## Dependency edges
- Always depends on `koruma-core` for the core traits.
- Always depends on `bon` and re-exports it as `koruma::bon` for builder generation.
- Optionally depends on `koruma-derive` (feature `derive`, enabled by default).
- Optionally depends on `inventory` (feature `showcase`).

## Feature flags
- `derive` (default): re-exports `Koruma`, `KorumaAllDisplay`, and `#[koruma::validator]` from `koruma-derive`.
- `fluent`: re-exports `KorumaAllFluent` (when `derive` is enabled) and enables fluent support in `koruma-derive`.
- `showcase`: enables `inventory` support and forwards the feature to `koruma-core` and `koruma-derive`.

## Control flow
- Users derive `Koruma` and annotate fields with `#[koruma(...)]`.
- The derive macro generates `validate()` and error types referencing `koruma-core` traits.
- Validators use `#[koruma::validator]` which injects a `bon` builder and a `with_value()` helper.

## Extension points
- Add new derives in `koruma-derive` and re-export them here behind a feature gate.
- Add new core traits in `koruma-core`, then re-export from this crate for downstream use.
