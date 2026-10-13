# koruma-core

[![Codecov: koruma-core][codecov-badge]][codecov]
[![crates.io: koruma-core][crate-badge]][crate]

`koruma-core` provides integration traits and typed validation-error data for validator and tooling
authors in the [Koruma project][project]. Application code normally uses [`koruma`][koruma], which
re-exports its application-facing API.

Requires Rust 1.99 or newer.

## Overview

Integration authors can use:

- `Validate<T>` and `ValidatorMetadata<T>` to define validators and expose their metadata;
- `ValidateExt`, `ValidationError`, and `ValidationIssues` to work with validated types and errors;
- `ValidationIssue` and related types for generic issue reporting; and
- `NewtypeValidation`, `NewtypeValue`, and `NewtypeTryFromInner` for validated wrappers.

See the [API reference][api] for the runtime contracts and issue model.

## Features

No features are enabled by default.

| Feature | Default | Purpose |
| --- | --- | --- |
| `default` | Yes | Enables no additional features. |
| `fluent` | No | Enables `es-fluent` contracts used by localized validation and showcase rendering. |
| `internal-showcase` | No | Internal site support: enables dynamic validator registration with `anyhow`, `inventory`, and `koruma-derive/internal-showcase`. |

## Example

A handwritten validator can implement `Validate<T>` without enabling any
features. The trait checks a borrowed value and returns whether it passes.

```rust
use koruma_core::Validate;

struct MinimumAge(u8);

impl Validate<u8> for MinimumAge {
    fn validate(&self, value: &u8) -> bool {
        *value >= self.0
    }
}

let rule = MinimumAge(13);
assert!(rule.validate(&18));
assert!(!rule.validate(&8));
```

[codecov-badge]: https://codecov.io/gh/stayhydated/koruma/branch/master/graph/badge.svg?component=koruma-core
[codecov]: https://codecov.io/gh/stayhydated/koruma
[crate-badge]: https://img.shields.io/crates/v/koruma-core.svg?label=koruma-core
[crate]: https://crates.io/crates/koruma-core
[project]: https://github.com/stayhydated/koruma
[koruma]: https://crates.io/crates/koruma
[api]: https://docs.rs/koruma-core/latest/koruma_core/
