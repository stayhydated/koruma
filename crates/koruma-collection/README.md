# koruma-collection

[![Codecov: koruma-collection][codecov-badge]][codecov]
[![crates.io: koruma-collection][crate-badge]][crate]

`koruma-collection` provides reusable string, format, numeric, collection, and presence validators
for applications built with the [Koruma project][project]. Depend on it directly alongside
[`koruma`][koruma] when using these rules; the facade does not re-export this collection.

Requires Rust 1.99 or newer.

The [API reference][api] lists the validators and their configuration methods.

## Features

The default feature set is `fmt`. The `full` bundle enables the seven optional
validator and value-type integrations; it does not enable localization or `fmt`
when default features are disabled.

| Feature | Default | Purpose |
| --- | --- | --- |
| `default` | Yes | Enables `fmt`. |
| `credit-card` | No | Enables `format::CreditCardValidation` through `card-validate`. |
| `email` | No | Enables `format::EmailValidation` through `email_address`. |
| `fluent` | No | Enables localized messages, embedded Fluent resources, and `koruma/fluent` through `es-fluent`, `es-fluent-manager-embedded`, and `rust-embed`. |
| `fmt` | Yes | Implements `Display` for validator errors. |
| `full` | No | Enables `url`, `credit-card`, `phone-number`, `email`, `regex`, `smallvec`, and `rust_decimal`. |
| `full-fluent` | No | Enables `full` and `fluent`. |
| `internal-showcase` | No | Internal site support: enables `full-fluent`, `fmt`, `anyhow`, and showcase support in `koruma` and `koruma-derive`. |
| `phone-number` | No | Enables `format::PhoneNumberValidation` through `phonenumber`. |
| `regex` | No | Enables `string::PatternValidation` with compiled regular expressions. |
| `rust_decimal` | No | Implements `numeric::Numeric` for `rust_decimal::Decimal`. |
| `smallvec` | No | Implements `collection::HasLen` for `smallvec::SmallVec`. |
| `url` | No | Enables `format::UrlValidation` through `url`. |

## Example

This complete example uses `koruma` with its default `derive` feature and
`koruma-collection`; these two validators need no optional integration features.
An empty username and an age below 13 produce separate typed failures.

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

[codecov-badge]: https://codecov.io/gh/stayhydated/koruma/branch/master/graph/badge.svg?component=koruma-collection
[codecov]: https://codecov.io/gh/stayhydated/koruma
[crate-badge]: https://img.shields.io/crates/v/koruma-collection.svg?label=koruma-collection
[crate]: https://crates.io/crates/koruma-collection
[project]: https://github.com/stayhydated/koruma
[koruma]: https://crates.io/crates/koruma
[api]: https://docs.rs/koruma-collection/latest/koruma_collection/
