# koruma

A type-safe validation framework for Rust with compile-time error generation.

## Installation

```toml
[dependencies]
koruma = "0.1"
```

## Quick Start

Define a validated struct:

```rust
use koruma::*;

#[derive(Koruma)]
pub struct User {
    #[koruma(MinLength::builder().min(3), MaxLength::builder().max(50))]
    name: String,

    #[koruma(Range::<_>::builder().min(0).max(150))]
    age: u8,
}

fn main() {
    let user = User {
        name: "Jo".to_string(),
        age: 25,
    };

    match user.validate() {
        Ok(()) => println!("Valid!"),
        Err(e) => {
            if let Some(name_err) = e.name() {
                println!("Name error: {}", name_err);
            }
        }
    }
}
```

## Features

### Default Features

- `derive` - Procedural macros for `#[derive(Koruma)]` and `#[koruma::validator]`

### Optional Features

- `fluent` - i18n support via Fluent
- `showcase` - Validator discovery for tooling

Enable features in your `Cargo.toml`:

```toml
[dependencies]
koruma = { version = "0.1", features = ["fluent"] }
```

## Creating Custom Validators

```rust
use koruma::*;

#[koruma::validator]
pub struct NotEmpty {
    #[koruma(value)]
    value: String,
}

impl NotEmpty {
    fn validate(&self) -> bool {
        !self.value.is_empty()
    }
}
```

## Validation Modes

### Standard Validation

```rust
#[derive(Koruma)]
pub struct Config {
    #[koruma(MinLength::builder().min(1))]
    name: String,
}
```

### Newtype Validation

```rust
#[derive(Koruma)]
#[koruma(newtype)]
pub struct Username(
    #[koruma(MinLength::builder().min(3))]
    String
);
```

### Validated Constructor

```rust
#[derive(Koruma)]
#[koruma(try_new)]
pub struct Email(
    #[koruma(EmailValidation::builder())]
    String
);

// Usage: Email::try_new("user@example.com")?
```

### Nested Validation

```rust
#[derive(Koruma)]
pub struct Address {
    #[koruma(MinLength::builder().min(1))]
    street: String,
}

#[derive(Koruma)]
pub struct Person {
    #[koruma(nested)]
    address: Address,
}
```

### Collection Validation

```rust
#[derive(Koruma)]
pub struct Team {
    #[koruma(each(MinLength::builder().min(1)))]
    members: Vec<String>,
}
```

## Pre-built Validators

For a collection of ready-to-use validators, see [koruma-collection](../koruma-collection).

## Documentation

- [Architecture](docs/ARCHITECTURE.md)
- [API Documentation](https://docs.rs/koruma)
