# koruma

[![CI][ci-badge]][ci]
[![Codecov][codecov-badge]][codecov]
[![Book][book-badge]][book]
[![crates.io: koruma][koruma-badge]][koruma-crate]

Koruma adds reusable validators to Rust struct fields and generates typed validation errors.
Applications start with [`koruma`][koruma-readme] for derives and core traits, then use
[`koruma-collection`][koruma-collection-readme] for built-in string, format, numeric, collection,
and presence rules.

The five public packages below require Rust 1.99 or newer.

## Crates

| Crate | Purpose |
| --- | --- |
| [`koruma`][koruma-readme] | Application facade with validation traits and optional macro re-exports. |
| [`koruma-collection`][koruma-collection-readme] | Built-in validators with optional format, value-type, and localization integrations. |
| [`koruma-core`][koruma-core-readme] | Runtime contracts and error types for handwritten integrations. |
| [`koruma-derive`][koruma-derive-readme] | Procedural macros, normally accessed through `koruma`'s `derive` feature. |
| [`koruma-derive-core`][koruma-derive-core-readme] | Attribute parsers and typed models for macro and analysis tooling. |

## Example

This complete example uses `koruma` with its default `derive` feature and
`koruma-collection`; these two validators need no optional integration features.
An empty username and an age below 13 produce separate typed failures. The same
example is checked as a doctest in the collection package.

```rust
use koruma::Koruma;
use koruma_collection::{collection, numeric};

#[derive(Koruma)]
struct Signup {
    #[koruma(collection::NonEmptyValidation::<_>)]
    username: String,

    #[koruma(numeric::RangeValidation::<_>.min(13_u8).max(120_u8))]
    age: u8,
}

fn main() {
    let signup = Signup {
        username: String::new(),
        age: 8,
    };

    match signup.validate() {
        Ok(()) => println!("Signup is valid"),
        Err(errors) => {
            assert!(errors.username().non_empty_validation().is_some());
            assert!(errors.age().range_validation().is_some());
        }
    }
}
```

[ci-badge]: https://github.com/stayhydated/koruma/actions/workflows/ci.yml/badge.svg?branch=master
[ci]: https://github.com/stayhydated/koruma/actions/workflows/ci.yml
[codecov-badge]: https://codecov.io/gh/stayhydated/koruma/branch/master/graph/badge.svg
[codecov]: https://codecov.io/gh/stayhydated/koruma
[book-badge]: https://img.shields.io/badge/Book-mdBook-blue
[book]: https://stayhydated.github.io/koruma/book/
[koruma-badge]: https://img.shields.io/crates/v/koruma.svg?label=koruma
[koruma-crate]: https://crates.io/crates/koruma
[koruma-readme]: crates/koruma/README.md
[koruma-collection-readme]: crates/koruma-collection/README.md
[koruma-core-readme]: crates/koruma-core/README.md
[koruma-derive-readme]: crates/koruma-derive/README.md
[koruma-derive-core-readme]: crates/koruma-derive-core/README.md
