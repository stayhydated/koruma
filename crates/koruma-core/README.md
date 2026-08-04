# koruma-core

[![API docs](https://docs.rs/koruma-core/badge.svg)](https://docs.rs/koruma-core/)
[![Crates.io](https://img.shields.io/crates/v/koruma-core.svg)](https://crates.io/crates/koruma-core)

`koruma-core` contains the public traits and data types shared by Koruma derives and integrations.
Application code should normally depend on [`koruma`](https://crates.io/crates/koruma), which
re-exports this crate's application-facing API.

Integration authors can use:

- `Validate<T>` and `ValidatorMetadata<T>` for validators;
- `ValidateExt`, `ValidationError`, and `ValidationIssues` for validated types and errors;
- `ValidationIssue` and related types for generic issue reporting; and
- `NewtypeValidation`, `NewtypeValue`, and `NewtypeTryFromInner` for validated wrappers.

See the [API reference](https://docs.rs/koruma-core/) for contracts and examples.
