# koruma

[![CI](https://github.com/stayhydated/koruma/actions/workflows/ci.yml/badge.svg?branch=master)](https://github.com/stayhydated/koruma/actions/workflows/ci.yml)
[![Codecov](https://codecov.io/gh/stayhydated/koruma/branch/master/graph/badge.svg)](https://codecov.io/gh/stayhydated/koruma)
[![Book](https://img.shields.io/badge/book-online-blue)](https://stayhydated.github.io/koruma/book/)
[![Crates.io](https://img.shields.io/crates/v/koruma.svg)](https://crates.io/crates/koruma)

Koruma adds reusable validators to Rust struct fields and generates strongly typed validation
errors. Use `koruma` for derives and core traits, then add `koruma-collection` for built-in
string, format, numeric, collection, and presence rules.

Koruma 0.12 requires Rust 1.98 or newer.

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

## Crates

Most applications depend only on `koruma` and optionally `koruma-collection`.
`koruma-core`, `koruma-derive`, and `koruma-derive-core` are public integration crates for
tooling and macro authors.

## License

Licensed under either Apache-2.0 or MIT.
