# koruma-derive

[![Docs](https://docs.rs/koruma-derive/badge.svg)](https://docs.rs/koruma-derive/)
[![Crates.io](https://img.shields.io/crates/v/koruma-derive.svg)](https://crates.io/crates/koruma-derive)

Proc-macro crate for `koruma`. Most users should depend on `koruma` with the `derive` feature
instead of using this crate directly.

Use this crate directly when you need to pin the macro dependency or integrate it into tooling.

## Macros

- `#[koruma::validator]`: generates a `bon` builder and `with_value()` helper for validators.
- `#[derive(Koruma)]`: generates validation error structs and `validate()`.
- `#[derive(KorumaAllDisplay)]`: adds `Display` for `all()` validator enums.
- `#[derive(KorumaAllFluent)]`: adds `ToFluentString` for `all()` validator enums (feature `fluent`).

## Installation

```toml
[dependencies]
koruma-derive = "0.3"
```

## Features

- `fluent`: enables `KorumaAllFluent` derive.
- `showcase`: enables showcase registration for validators.
