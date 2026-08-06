# Get started

This tutorial validates a signup value with two built-in rules and inspects the generated,
field-specific errors.

## Prerequisites

Install Rust 1.96 or newer and start in a Rust package where you can edit `Cargo.toml` and
`src/main.rs`.

## Add Koruma

Add the facade and built-in validator collection:

```toml
[dependencies]
koruma = "0.11"
koruma-collection = "0.11"
```

The default features enable Koruma's derives and `Display` messages for the validators used here.

## Validate a struct

Replace `src/main.rs` with:

```rust
use koruma::Koruma;
use koruma_collection::{collection, numeric};

#[derive(Koruma)]
struct SignupInput {
    #[koruma(collection::NonEmptyValidation::<_>)]
    username: String,

    #[koruma(numeric::RangeValidation::<_>.min(13_u8).max(120_u8))]
    age: u8,
}

fn main() {
    let invalid = SignupInput {
        username: String::new(),
        age: 8,
    };

    let errors = invalid.validate().expect_err("the input should fail");
    if let Some(error) = errors.username().non_empty_validation() {
        println!("username: {error}");
    }
    if let Some(error) = errors.age().range_validation() {
        println!("age: {error}");
    }

    let valid = SignupInput {
        username: "alice".to_string(),
        age: 30,
    };
    assert!(valid.validate().is_ok());
}
```

Run the package:

```console
cargo run
```

The invalid value prints `username: Must not be empty.` and
`age: Must be in the range [13, 120].`. The final assertion confirms that the valid value passes.

Koruma generates `validate()`, the aggregate error type, and field-specific accessors from
`#[derive(Koruma)]`. Continue with [Declare custom validators](declare_validators.md) when a
built-in rule does not fit, or use the [koruma-collection catalog](koruma_collection.md) to choose
another built-in validator.
