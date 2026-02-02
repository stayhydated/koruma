# koruma-core

[![Docs](https://docs.rs/koruma-core/badge.svg)](https://docs.rs/koruma-core/)
[![Crates.io](https://img.shields.io/crates/v/koruma-core.svg)](https://crates.io/crates/koruma-core)

Core traits and types for the `koruma` validation ecosystem. Most users should depend on
`koruma` instead of this crate directly.

Use this crate when you are:

- Implementing custom validators without the proc-macro layer
- Building tooling that only needs the trait definitions

## What it provides

- `Validate<T>`: implemented by validator structs.
- `ValidationError`: implemented by generated error structs.
- `BuilderWithValue<T>`: implemented by `#[koruma::validator]` builders for `with_value()`.
- `ValidateExt`: used for nested/newtype validation.
- `NewtypeValidation`: marker for newtype structs with transparent error access.
- Optional `showcase` registry types.

## Installation

```toml
[dependencies]
koruma-core = "*"
```
