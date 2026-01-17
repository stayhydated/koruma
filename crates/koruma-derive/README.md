# koruma-derive

Procedural macros for the Koruma validation framework.

## Overview

This crate provides the procedural macros that power Koruma's code generation. Most users should depend on the main `koruma` crate with the `derive` feature (enabled by default), which re-exports these macros.

## Installation

```toml
[dependencies]
koruma-derive = "0.1"
```

Or via the main crate (recommended):

```toml
[dependencies]
koruma = "0.1"  # derive feature is on by default
```

## Provided Macros

### `#[koruma::validator]`

Define a custom validator:

```rust
use koruma::*;

#[koruma::validator]
pub struct MinLength {
    min: usize,
    #[koruma(value)]
    value: String,
}

impl MinLength {
    fn validate(&self) -> bool {
        self.value.len() >= self.min
    }
}
```

The macro generates:

- `Validate<T>` implementation
- Builder pattern via `bon`
- Optional inventory registration

### `#[derive(Koruma)]`

Generate validation for a struct:

```rust
use koruma::*;

#[derive(Koruma)]
pub struct User {
    #[koruma(MinLength::builder().min(3))]
    name: String,

    #[koruma(Range::<_>::builder().min(0).max(150))]
    age: u8,
}
```

Generates:

- `UserValidationError` struct
- Per-field error enums
- `ValidateExt` implementation
- Field accessor methods

### `#[derive(KorumaAllDisplay)]`

Generate `Display` for error enums:

```rust
#[derive(KorumaAllDisplay)]
pub enum NameError {
    MinLength(MinLength),
    MaxLength(MaxLength),
}
```

### `#[derive(KorumaAllFluent)]`

Generate Fluent i18n support (requires `fluent` feature):

```rust
#[derive(KorumaAllFluent)]
pub enum NameError {
    MinLength(MinLength),
    MaxLength(MaxLength),
}
```

## Attribute Syntax

### Field Attributes

```rust
#[koruma(Validator)]                           // Single validator
#[koruma(Validator1, Validator2)]              // Multiple validators
#[koruma(Validator::builder().config(value))]  // With configuration
#[koruma(Validator::<_>)]                      // Type inference
#[koruma(nested)]                              // Nested struct validation
#[koruma(each(Validator))]                     // Validate collection elements
```

### Struct Attributes

```rust
#[koruma(newtype)]                    // Single-field wrapper mode
#[koruma(try_new)]                    // Generate validated constructor
#[koruma(error = "CustomErrorName")]  // Custom error type name
```

## Features

| Feature | Default | Description |
|---------|---------|-------------|
| `showcase` | No | Enable `inventory` registration |

## Documentation

- [Architecture](docs/ARCHITECTURE.md)
- [API Documentation](https://docs.rs/koruma-derive)
