# Architecture: koruma-collection

## Purpose

`koruma-collection` ships a curated set of validators built on top of `koruma`, organized by domain (`string`, `format`, `numeric`, `collection`, `general`). It optionally integrates with `es-fluent` for localized error messages.

## Modules

- `crates/koruma-collection/src/lib.rs`: re-exports validators and exposes the `i18n` module when `fluent` is enabled.
- `crates/koruma-collection/src/validators/`: category modules (`string`, `format`, `numeric`, `collection`, `general`).
- `crates/koruma-collection/src/i18n.rs`: embedded Fluent module via `es_fluent_manager_embedded::define_i18n_module!()`.
- `crates/koruma-collection/i18n/`: Fluent resources by locale.
- `crates/koruma-collection/src/validators/mod.rs`: declares validator category modules and,
  under `internal-showcase`, calls `koruma_derive::showcase_modules!(...)` to emit a central
  `__link_showcase_validators()` function that registers all showcase validators.

## Validator pattern

- Each validator is a struct annotated with `#[koruma::validator]`.
- One field is marked `#[koruma(value)]` to store the validated value, with a generated getter on
  the validator type for external access.
- Each validator implements `Validate<T>`; optional `Display` impls live behind `fmt`.
- Optional `#[showcase(...)]` metadata registers validators when `internal-showcase` is enabled; showcase registrations must state `input_type` explicitly.

## Feature flags

- `default`: `fmt`.
- `full`: enables all optional validators and dependencies.
- `fluent`: enables Fluent integration and embedded i18n assets.
- `full-fluent`: `full` + `fluent`.
- Per-validator and integration features: `email`, `url`, `phone-number`, `credit-card`, `regex`, `smallvec`, `decimal` (`rust_decimal::Decimal` support for `numeric::Numeric`).
- `internal-showcase`: enables validator registry support via `koruma/internal-showcase`, turns on
  `full-fluent`, keeps `fmt`, and adds `anyhow` for showcase factory closures.

## Extending

- Add validators under `src/validators/<category>` and re-export in `mod.rs`.
- Add localized messages under `i18n/<locale>/koruma-collection.ftl` when `fluent` is in use.
