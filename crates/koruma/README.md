# koruma

[![Build Status](https://github.com/stayhydated/koruma/actions/workflows/ci.yml/badge.svg)](https://github.com/stayhydated/koruma/actions/workflows/ci.yml)
[![Docs](https://docs.rs/koruma/badge.svg)](https://docs.rs/koruma/)
[![Crates.io](https://img.shields.io/crates/v/koruma.svg)](https://crates.io/crates/koruma)

`koruma` is a per-field validation framework focused on:

1. **Type Safety**: Strongly typed validation error structs generated at compile time.
1. **Ergonomics**: Derive macros and validator attributes that minimize boilerplate.
1. **Developer Experience**: Optional constructors, nested/newtype validation, and fluent/i18n.

## koruma-collection

[![Docs](https://docs.rs/koruma-collection/badge.svg)](https://docs.rs/koruma-collection/)
[![Crates.io](https://img.shields.io/crates/v/koruma-collection.svg)](https://crates.io/crates/koruma-collection)
[![Crowdin](https://badges.crowdin.net/koruma-collection/localized.svg)](https://crowdin.com/project/koruma-collection)

A curated set of validators built on top of `koruma`, organized by domain:
string, format, numeric, collection, and general-purpose validators.

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

## Fluent/i18n

Enable the `fluent` feature and derive `EsFluent` on validators for localized messages. Use
`KorumaAllFluent` to format `all()` results, and `koruma-collection` with `full-fluent` to get
built-in validators plus translations.

## Examples

- [collection validators](../../examples/collection)
- [user-defined validators](../../examples/user-defined)
