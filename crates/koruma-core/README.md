# koruma-core

[![Docs](https://docs.rs/koruma-core/badge.svg)](https://docs.rs/koruma-core/)
[![Crates.io](https://img.shields.io/crates/v/koruma-core.svg)](https://crates.io/crates/koruma-core)

Core traits and types for the `koruma` validation ecosystem. Most users should depend on
`koruma` instead of this crate directly.

## What it provides

- `Validate<T>`: implemented by validator structs.
- `ValidationError`: implemented by generated error structs.
- `BuilderWithValue<T>`: helper trait for owned-value builder APIs.
- `ValidateExt`: used for nested/newtype validation.
- `NewtypeValidation`: marker for newtype structs with transparent error access.
