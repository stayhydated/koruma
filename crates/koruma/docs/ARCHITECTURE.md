# Architecture: koruma

## Purpose

`koruma` is the public facade crate. It re-exports the core traits from `koruma-core`, the derive macros from `koruma-derive`, and the `bon` builder API as `koruma::bon`. It also owns feature gates that enable optional i18n with [Project Fluent](https://projectfluent.org/) and the internal showcase registry support used by workspace demos.

## Key entry points

- `crates/koruma/src/lib.rs`: re-exports, feature gating, and README docs via `include_str!`.

## Dependency edges

- Always depends on `koruma-core` for the core traits.
- Always depends on `bon` and re-exports it as `koruma::bon` for builder generation.
- Optionally depends on `koruma-derive` (feature `derive`, enabled by default).
- Optionally depends on `inventory` (feature `internal-showcase`) and re-exports it for registry submission.

## Feature flags

- `derive` (default): re-exports `Koruma`, `KorumaAllDisplay`, and `#[koruma::validator]` from `koruma-derive`.
- `fluent`: re-exports `KorumaAllFluent` when `derive` is enabled and forwards fluent support to `koruma-derive`.
- `internal-showcase`: enables `koruma_core::showcase`, forwards showcase support to `koruma-core` and `koruma-derive`, and re-exports `inventory` for registry submission.

  The showcase feature also enables macro-generated registry wiring: `koruma-collection` uses
  `koruma_derive::showcase_modules!` to emit a module linker and `koruma-core` uses
  `koruma_derive::showcase_module_enum!` for `ValidatorModule`.

## Control flow

- Users derive `Koruma` and annotate fields with `#[koruma(...)]`.
- The derive macro generates `validate()` implementations and typed error structs using `koruma-core` traits.
- Validators annotated with `#[koruma::validator]` get hidden `bon` builder plumbing, direct setter
  entrypoints, and `with_value()` helpers.
- When `internal-showcase` is enabled, validators can register metadata for discovery via `inventory`.

## Extension points

- Add new derives in `koruma-derive` and re-export them behind a feature gate here.
- Add new core traits in `koruma-core`, then re-export them from this crate for downstream use.
