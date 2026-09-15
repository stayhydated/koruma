# koruma-core

[![CI](https://github.com/stayhydated/koruma/actions/workflows/ci.yml/badge.svg?branch=master)](https://github.com/stayhydated/koruma/actions/workflows/ci.yml)
[![Codecov](https://codecov.io/gh/stayhydated/koruma/branch/master/graph/badge.svg)](https://codecov.io/gh/stayhydated/koruma)
[![Book](https://img.shields.io/badge/book-online-blue)](https://stayhydated.github.io/koruma/book/)
[![Crates.io](https://img.shields.io/crates/v/koruma-core.svg)](https://crates.io/crates/koruma-core)

`koruma-core` contains the public traits and data types shared by Koruma derives and integrations.
Application code should normally depend on [`koruma`](https://crates.io/crates/koruma), which
re-exports this crate's application-facing API.

Integration authors can use:

- `Validate<T>` and `ValidatorMetadata<T>` for validators;
- `ValidateExt`, `ValidationError`, and `ValidationIssues` for validated types and errors;
- `ValidationIssue` and related types for generic issue reporting; and
- `NewtypeValidation`, `NewtypeValue`, and `NewtypeTryFromInner` for validated wrappers.
