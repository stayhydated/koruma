# koruma-core

Core traits and types for the Koruma validation framework.

## Overview

This crate provides the foundational traits that define the validation contract in Koruma. Most users should depend on the main `koruma` crate instead, which re-exports everything from this crate.

## Installation

```toml
[dependencies]
koruma-core = "0.1"
```

## Core Traits

### `Validate<T>`

The primary trait for validators:

```rust
pub trait Validate<T> {
    fn validate(&self, value: &T) -> bool;
}
```

Implement this trait to create custom validators:

```rust
use koruma_core::Validate;

struct MinLength {
    min: usize,
}

impl Validate<String> for MinLength {
    fn validate(&self, value: &String) -> bool {
        value.len() >= self.min
    }
}
```

### `ValidationError`

Trait for validation error types:

```rust
pub trait ValidationError {
    fn is_empty(&self) -> bool;
    fn has_errors(&self) -> bool;
}
```

### `ValidateExt`

Extension trait for validated structs:

```rust
pub trait ValidateExt {
    type Error: ValidationError;
    fn validate(&self) -> Result<(), Self::Error>;
}
```

### `NewtypeValidation`

Marker trait for newtype validation:

```rust
pub trait NewtypeValidation {
    type Inner;
    type Error: ValidationError;
    fn validate(value: &Self::Inner) -> Result<(), Self::Error>;
}
```

### `BuilderWithValue<T>`

Builder pattern support:

```rust
pub trait BuilderWithValue<T> {
    fn value(self, value: T) -> Self;
}
```

## Features

| Feature | Default | Description |
|---------|---------|-------------|
| `showcase` | No | Validator discovery via `inventory` |

## When to Use This Crate

Use `koruma-core` directly when:

- Building tooling that needs koruma's traits
- Creating a custom derive macro
- You need minimal dependencies

For application code, use the main `koruma` crate instead.

## Documentation

- [Architecture](docs/ARCHITECTURE.md)
- [API Documentation](https://docs.rs/koruma-core)
