# koruma-collection

[![Codecov: koruma-collection][codecov-badge]][codecov]
[![crates.io: koruma-collection][crate-badge]][crate]

`koruma-collection` provides reusable string, format, numeric, collection, and presence validators
for applications built with the [Koruma project][project].

## Overview

The default `fmt` feature provides `Display` implementations for validator errors. Use `full` for
all optional validators and value-type integrations, `fluent` for localized messages through
es-fluent, or `full-fluent` for both; individual integrations can be enabled separately.

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

[codecov-badge]: https://codecov.io/gh/stayhydated/koruma/branch/master/graph/badge.svg?component=koruma-collection
[codecov]: https://codecov.io/gh/stayhydated/koruma
[crate-badge]: https://img.shields.io/crates/v/koruma-collection.svg?label=koruma-collection
[crate]: https://crates.io/crates/koruma-collection
[project]: https://github.com/stayhydated/koruma
