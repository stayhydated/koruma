# koruma-core

[![Codecov: koruma-core][codecov-badge]][codecov]
[![crates.io: koruma-core][crate-badge]][crate]

`koruma-core` provides integration traits and typed validation-error data for validator and tooling
authors in the [Koruma project][project]. Application code normally uses [`koruma`][koruma], which
re-exports its application-facing API.

## Overview

Integration authors can use:

- `Validate<T>` and `ValidatorMetadata<T>` to define validators and expose their metadata;
- `ValidateExt`, `ValidationError`, and `ValidationIssues` to work with validated types and errors;
- `ValidationIssue` and related types for generic issue reporting; and
- `NewtypeValidation`, `NewtypeValue`, and `NewtypeTryFromInner` for validated wrappers.

[codecov-badge]: https://codecov.io/gh/stayhydated/koruma/branch/master/graph/badge.svg?component=koruma-core
[codecov]: https://codecov.io/gh/stayhydated/koruma
[crate-badge]: https://img.shields.io/crates/v/koruma-core.svg?label=koruma-core
[crate]: https://crates.io/crates/koruma-core
[project]: https://github.com/stayhydated/koruma
[koruma]: https://crates.io/crates/koruma
