# koruma-derive

[![Docs](https://docs.rs/koruma-derive/badge.svg)](https://docs.rs/koruma-derive/)
[![Crates.io](https://img.shields.io/crates/v/koruma-derive.svg)](https://crates.io/crates/koruma-derive)

Proc-macro crate for `koruma`. Most users should depend on `koruma` with the `derive` feature
instead of using this crate directly.

## Macros

- `#[koruma::validator]`: generates a `bon` builder and `with_value()` helper for validators, and
  supports `#[koruma(value, skip_capture)]` on `Option<T>` value fields that should not retain the
  validated input.
- `#[derive(Koruma)]`: generates validation error structs and `validate()`, accepting either
  shorthand validator args like `RangeValidation<_>(min = 0, max = 10)` or Rust-native builder
  chains like `RangeValidation::<_>::builder().min(0).max(10)`.
- `#[derive(KorumaAllDisplay)]`: adds `Display` for `all()` validator enums.
- `#[derive(KorumaAllFluent)]`: adds `ToFluentString` for `all()` validator enums (feature `fluent`).

## Features

- `fluent`: enables `KorumaAllFluent` derive.
