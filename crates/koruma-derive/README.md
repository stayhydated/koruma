# koruma-derive

[![Docs](https://docs.rs/koruma-derive/badge.svg)](https://docs.rs/koruma-derive/)
[![Crates.io](https://img.shields.io/crates/v/koruma-derive.svg)](https://crates.io/crates/koruma-derive)

Proc-macro crate for `koruma`. Most users should depend on `koruma` with the `derive` feature
instead of using this crate directly.

## Macros

- `#[koruma::validator]`: generates Koruma-owned builder plumbing, direct setter entrypoints such
  as `RangeValidation::min(value)`, and `with_value()` helpers for validators. It supports
  `#[koruma(setter(into))]`, `#[koruma(setter(required))]`,
  `#[koruma(setter(name = ...))]`, and `#[koruma(setter(default = ...))]` on direct setter fields,
  plus `#[koruma(skip_capture)]` on `Option<T>` value fields that should not retain the
  validated input during derived validation.
- `#[derive(Koruma)]`: generates validation error structs and `validate()`, accepting Rust-native
  direct validator chains like `RangeValidation::<_>::min(0).max(10)`.
- `#[derive(KorumaAllDisplay)]`: adds `Display` for `all()` validator enums.
- `#[derive(KorumaAllFluent)]`: adds `FluentMessage` for `all()` validator enums (feature `fluent`).

## Features

- `fluent`: enables `KorumaAllFluent` derive.
