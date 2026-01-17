# koruma-derive Architecture

## Overview

The `koruma-derive` crate provides procedural macros for automatic validation code generation. It transforms user-annotated structs into fully functional validated types with type-safe error handling.

## Role in the Workspace

```
koruma-derive (this crate - code generation)
    │
    └── koruma-derive-core (parsing logic)
    ↑
    └── Used by: koruma (facade)
```

## Module Structure

```
src/
├── lib.rs              # Macro entry points
├── expand/
│   ├── mod.rs          # Module exports
│   ├── derive.rs       # #[derive(Koruma)] implementation
│   ├── validator.rs    # #[koruma::validator] implementation
│   ├── display.rs      # #[derive(KorumaAllDisplay)] implementation
│   ├── fluent.rs       # #[derive(KorumaAllFluent)] implementation
│   └── codegen.rs      # Shared code generation utilities
└── tests/
    ├── snapshot_tests.rs      # Snapshot-based regression tests
    ├── attr_parsing_tests.rs  # Attribute parsing validation
    ├── error_tests.rs         # Error handling tests
    └── helper_tests.rs        # Helper function tests
```

## Exported Macros

### `#[koruma::validator]`

Attribute macro that marks a struct as a validator:

```rust
#[koruma::validator]
pub struct MinLength {
    min: usize,
    #[koruma(value)]
    value: String,
}
```

**Generated code:**

- `Validate<T>` trait implementation
- Builder pattern via `bon`
- Optional `inventory` registration for showcase

### `#[derive(Koruma)]`

Derive macro for structs that need validation:

```rust
#[derive(Koruma)]
pub struct User {
    #[koruma(MinLength::builder().min(3))]
    name: String,
    #[koruma(Range::builder().min(0).max(150))]
    age: u8,
}
```

**Generated code:**

- `UserValidationError` struct with per-field errors
- `ValidateExt` trait implementation
- Field accessor methods on error struct
- Support for `newtype` and `try_new` modes

### `#[derive(KorumaAllDisplay)]`

Generates `Display` implementation for validator enum types:

```rust
#[derive(KorumaAllDisplay)]
pub enum NameError {
    MinLength(MinLength),
    MaxLength(MaxLength),
}
```

### `#[derive(KorumaAllFluent)]`

Generates Fluent i18n support for validator enums (requires `fluent` feature).

## Code Generation Flow

### Validator Definition

1. Parse struct with `#[koruma::validator]`
1. Identify the `#[koruma(value)]` field
1. Generate `Validate<T>` impl calling user's `validate` method
1. Generate builder via `#[bon::builder]`
1. Optionally register with inventory

### Struct Validation

1. Parse struct with `#[derive(Koruma)]`
1. For each field with `#[koruma(...)]` attributes:
   - Parse validator expressions
   - Generate error enum for field
   - Generate validation logic
1. Generate error struct with:
   - `Option<FieldError>` for each validated field
   - Accessor methods
   - `ValidationError` trait impl
1. Generate `ValidateExt` impl with `validate()` method

## Attribute Syntax

### Field-Level Attributes

```rust
#[koruma(Validator1, Validator2)]           // Multiple validators
#[koruma(Validator::<_>)]                   // Type inference
#[koruma(nested)]                           // Nested struct validation
#[koruma(each(Validator))]                  // Collection element validation
```

### Struct-Level Attributes

```rust
#[koruma(newtype)]                          // Single-field wrapper
#[koruma(try_new)]                          // Generate validated constructor
#[koruma(error = "CustomError")]            // Custom error type name
```

## Error Struct Generation

For a struct like:

```rust
#[derive(Koruma)]
pub struct User {
    #[koruma(MinLength::builder().min(3))]
    name: String,
}
```

Generates:

```rust
pub struct UserValidationError {
    name: Option<NameError>,
}

pub enum NameError {
    MinLength(MinLength),
}

impl UserValidationError {
    pub fn name(&self) -> &NameError { ... }
}
```

## Design Decisions

### Separation of Parsing

Parsing logic is in `koruma-derive-core` to:

- Allow external tools to analyze koruma metadata
- Keep macro crate focused on code generation
- Enable reuse of parsing logic

### Type-Safe Errors

Generated error types are strongly typed:

- Each field gets its own error enum
- Validators are captured in errors for inspection
- No stringly-typed errors

### Builder Integration

Uses `bon` for builder patterns:

- Consistent API across validators
- Compile-time validation of required fields
- Fluent configuration syntax

## Features

| Feature | Default | Description |
|---------|---------|-------------|
| `showcase` | No | Enables `inventory` registration for validators |

## Testing

The crate uses snapshot testing via `insta` to ensure generated code remains correct across changes. Tests cover:

- Basic derive functionality
- Attribute parsing edge cases
- Error message generation
- Feature combinations
