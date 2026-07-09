# koruma-derive

[![Docs](https://docs.rs/koruma-derive/badge.svg)](https://docs.rs/koruma-derive/)
[![Crates.io](https://img.shields.io/crates/v/koruma-derive.svg)](https://crates.io/crates/koruma-derive)

Proc-macro crate for `koruma`. Most users should depend on `koruma` with the `derive` feature
instead of using this crate directly.

## Macros

- `#[koruma::validator]`: generates Koruma-owned builder plumbing used by attribute setter syntax
  such as `#[koruma(RangeValidation.min(value))]`, plus `with_value()` helpers for validators. It supports
  inferred captured value fields named `actual`, `input`, or `value`, plus single unmarked
  value fields when no conventional name is present,
  `#[koruma(setter)]` for default setter behavior on fields that would otherwise be inferred,
  `#[koruma(setter(into))]`, `#[koruma(setter(required))]`,
  `#[koruma(setter(name = ...))]`, and `#[koruma(setter(default = ...))]` on direct setter fields,
  `#[koruma(value)]` for explicitly named captured value fields,
  plus `#[koruma(skip_capture)]` on `Option<T>` value fields that should not retain the
  validated input during derived validation. It also emits `ValidatorMetadata<T>` with a
  static descriptor and runtime parameter values; primitive, string, and `Option` parameters are
  represented directly, while generic or otherwise unconstrained values are reported as opaque.
- `#[derive(Koruma)]`: generates validation error structs and `validate()`, accepting
  dot-chain validator syntax like `RangeValidation::<_>.min(0).max(10)`, lower-snake labels,
  `each(...)` element validators, `full(...)` and `unwrapped(...)` target selectors, `skip`,
  `nested`, `newtype`, and struct-level `try_new`/`try_from` options. Generated aggregate error
  structs implement `ValidationIssues` for field- and element-scoped issue enumeration.
- `#[derive(KorumaAllDisplay)]`: adds `Display` for `all()` validator enums.
- `#[derive(KorumaAllFluent)]`: adds `FluentMessage` for `all()` validator enums (feature `fluent`).

## Features

- `fluent`: enables `KorumaAllFluent` derive.
- `internal-showcase`: enables internal showcase helper macros and validator
  registry metadata used by workspace demos.
