# Architecture: koruma-collection

## Purpose
`koruma-collection` ships a curated set of validators built on top of `koruma`, organized by domain (strings, formats, numbers, collections). It optionally integrates with `es-fluent` for localized error messages.

## Modules
- `crates/koruma-collection/src/lib.rs`: re-exports validators and exposes the `i18n` module when `fluent` is enabled.
- `crates/koruma-collection/src/validators/`: category modules (`string`, `format`, `numeric`, `collection`, `general`).
- `crates/koruma-collection/src/i18n.rs`: embedded Fluent loader via `es_fluent_manager_embedded`.
- `crates/koruma-collection/i18n/`: Fluent resources by locale.

## Validator pattern
- Each validator is a struct annotated with `#[koruma::validator]`.
- One field is marked `#[koruma(value)]` to store the validated value.
- Each validator implements `Validate<T>`; optional `Display` impls live behind `fmt`.
- Optional `#[showcase(...)]` metadata registers validators when `showcase` is enabled.

## Feature flags
- `default`: `fmt`.
- `full`: enables all optional validators and dependencies.
- `fluent`: enables Fluent integration and i18n assets.
- `full-fluent`: `full` + `fluent`.
- Per-validator features: `email`, `email-idna`, `url`, `phone-number`, `credit-card`, `regex`, `rust_decimal`, `smallvec`, `heck`.
- `showcase`: enables validator registry support via `koruma/showcase`.

## Extending
- Add validators under `src/validators/<category>` and re-export in `mod.rs`.
- Add localized messages under `i18n/<locale>/koruma-collection.ftl` when `fluent` is in use.
