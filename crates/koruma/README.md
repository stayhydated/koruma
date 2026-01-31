# koruma

[![Build Status](https://github.com/stayhydated/koruma/actions/workflows/ci.yml/badge.svg)](https://github.com/stayhydated/koruma/actions/workflows/ci.yml)
[![Docs](https://docs.rs/koruma/badge.svg)](https://docs.rs/koruma/)
[![Crates.io](https://img.shields.io/crates/v/koruma.svg)](https://crates.io/crates/koruma)

# Project Overview

`koruma` is a per-field validation framework written in **Rust**, focused on:

1. **Type Safety**: Strongly typed error structs generated at compile-time.
1. **Ergonomics**: Derive macros and validator attributes that minimize boilerplate.
1. **Developer Experience**: Optional `try_new` constructors, nested/newtype validation, and fluent/i18n hooks.

## Installation

```toml
[dependencies]
koruma = { version = "0.3", features = ["derive"] } # derive is the default
```

Optional validator collection:

```toml
[dependencies]
koruma-collection = { version = "0.3", features = ["full"] }
```

## Quick Start

```rust
use koruma::{Koruma, Validate, validator};

#[validator]
#[derive(Clone, Debug)]
pub struct RangeValidation<T> {
    pub min: T,
    pub max: T,
    #[koruma(value)]
    pub actual: T,
}

impl<T: PartialOrd + Clone> Validate<T> for RangeValidation<T> {
    fn validate(&self, value: &T) -> bool {
        *value >= self.min && *value <= self.max
    }
}

#[derive(Koruma)]
pub struct User {
    #[koruma(RangeValidation::<_>(min = 0, max = 150))]
    pub age: i32,
}

let user = User { age: 200 };
let err = user.validate().unwrap_err();
if let Some(range_err) = err.age().range_validation() {
    println!("age out of range: {}", range_err.actual);
}
```

## Core Concepts

- **Validators**: Annotate structs with `#[koruma::validator]` and implement `Validate<T>`.
- **Derive**: `#[derive(Koruma)]` generates typed error structs and `validate()`.
- **Multiple validators**: Use comma-separated validators per field.
- **Collections**: Use `each(...)` to validate elements of `Vec<T>`.
- **Optional fields**: `Option<T>` is skipped when `None`.
- **Nested/newtype**: Use `#[koruma(nested)]` or `#[koruma(newtype)]`.
- **Constructors**: `#[koruma(try_new)]` generates a validated constructor.
- **Formatting**: Derive `KorumaAllDisplay` for `Display` on `all()` enums; add `KorumaAllFluent` (feature `fluent`) for `ToFluentString`.

## Fluent/i18n

Enable the `fluent` feature and derive `EsFluent` on validators for localized messages. Use
`koruma-collection` with `full-fluent` to get built-in validators plus translations.

## Showcase registry (optional)

Enable the `showcase` feature to register validators in an `inventory`-backed registry. This can
power UIs or tooling that need to discover validators and their metadata.

## Crates

- [`koruma`](https://docs.rs/koruma) - public facade crate, re-exports traits and derive macros
- [`koruma-core`](https://docs.rs/koruma-core) - core traits and showcase registry types
- [`koruma-derive`](https://docs.rs/koruma-derive) - proc-macros for `Koruma` and validators
- [`koruma-derive-core`](https://docs.rs/koruma-derive-core) - attribute parsing utilities
- [`koruma-collection`](https://docs.rs/koruma-collection) - common validators with i18n resources

## Examples

- [collection validators](https://github.com/stayhydated/koruma/tree/master/examples/collection)
- [user-defined validators](https://github.com/stayhydated/koruma/tree/master/examples/user-defined)
- [i18n with fluent](https://github.com/stayhydated/koruma/tree/master/examples/i18n)
- [shared-lib workspace example](https://github.com/stayhydated/koruma/tree/master/examples/shared-lib)

## Development

See `CONTRIBUTING.md` for local workflows, testing, and style conventions.
