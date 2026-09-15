# koruma-collection

[![CI](https://github.com/stayhydated/koruma/actions/workflows/ci.yml/badge.svg?branch=master)](https://github.com/stayhydated/koruma/actions/workflows/ci.yml)
[![Codecov](https://codecov.io/gh/stayhydated/koruma/branch/master/graph/badge.svg)](https://codecov.io/gh/stayhydated/koruma)
[![Book](https://img.shields.io/badge/book-online-blue)](https://stayhydated.github.io/koruma/book/)
[![Crates.io](https://img.shields.io/crates/v/koruma-collection.svg)](https://crates.io/crates/koruma-collection)
[![Crowdin](https://badges.crowdin.net/koruma-collection/localized.svg)](https://crowdin.com/project/koruma-collection)

`koruma-collection` provides reusable validators for strings, formats, numbers, collections, and
optional values.

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
```

## Features

- `fmt` (default): implements `Display` for validator errors.
- `full`: enables all optional validators and type integrations.
- `fluent`: enables localized messages through es-fluent.
- `full-fluent`: combines `full` and `fluent`.

Optional validators can also be enabled individually with `credit-card`, `email`,
`phone-number`, `regex`, or `url`. The `smallvec` and `rust_decimal` features add support
for those value types.
