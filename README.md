# koruma

[![Build status](https://github.com/stayhydated/koruma/actions/workflows/ci.yml/badge.svg)](https://github.com/stayhydated/koruma/actions/workflows/ci.yml)
[![Book](https://img.shields.io/badge/docs-book-black)](https://stayhydated.github.io/koruma/book/)
[![API docs](https://docs.rs/koruma/badge.svg)](https://docs.rs/koruma/)
[![Crates.io](https://img.shields.io/crates/v/koruma.svg)](https://crates.io/crates/koruma)

Koruma adds reusable validators to Rust struct fields and generates strongly typed validation
errors. Use `koruma` for derives and core traits, then add `koruma-collection` for built-in
string, format, numeric, collection, and presence rules.

Koruma 0.10 requires Rust 1.96 or newer.

## Quick start

```toml
[dependencies]
koruma = "0.11"
koruma-collection = "0.11"
```

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

## Documentation

- [Get started](https://stayhydated.github.io/koruma/book/getting_started.html)
- [Choose built-in validators](https://stayhydated.github.io/koruma/book/koruma_collection.html)
- [Define custom validators](https://stayhydated.github.io/koruma/book/declare_validators.html)
- [API reference](https://docs.rs/koruma/)

Most applications depend only on `koruma` and optionally `koruma-collection`.
`koruma-core`, `koruma-derive`, and `koruma-derive-core` are public integration crates for
tooling and macro authors.

## License

Licensed under either Apache-2.0 or MIT.
