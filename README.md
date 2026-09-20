# koruma

[![CI][ci-badge]][ci]
[![Codecov][codecov-badge]][codecov]
[![Book][book-badge]][book]
[![crates.io: koruma][koruma-badge]][koruma-crate]

Koruma adds reusable validators to Rust struct fields and generates strongly typed validation
errors. Use `koruma` for derives and core traits, then add `koruma-collection` for built-in
string, format, numeric, collection, and presence rules.

Koruma requires Rust 1.98 or newer.

## Crates

| Crate | Purpose | Source |
| --- | --- | --- |
| `koruma` | Application-facing derives and validation traits | [README][koruma-readme] |
| `koruma-collection` | Reusable validators and optional localization support | [README][koruma-collection-readme] |
| `koruma-core` | Integration traits and validation error types | [README][koruma-core-readme] |
| `koruma-derive` | Procedural macros and generated validation APIs | [README][koruma-derive-readme] |
| `koruma-derive-core` | Attribute parsers and typed models for tooling and macros | [README][koruma-derive-core-readme] |

## Example

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
    let errors = Signup {
        username: String::new(),
        age: 8,
    }
    .validate()
    .expect_err("invalid signup should fail");

    assert!(errors.username().non_empty_validation().is_some());
    assert!(errors.age().range_validation().is_some());
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
