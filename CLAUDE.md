# Koruma - Project Documentation Index

This document provides an overview of the Koruma workspace for AI assistants and developers.

## Crate Overview

| Crate | Link to Architecture Doc | Purpose |
| -------------------- | -------------------------------------------------------------- | -------------------------------------- |
| `koruma` | [ARCHITECTURE](crates/koruma/docs/ARCHITECTURE.md) | Main facade crate - unified public API |
| `koruma-core` | [ARCHITECTURE](crates/koruma-core/docs/ARCHITECTURE.md) | Core traits and types |
| `koruma-derive` | [ARCHITECTURE](crates/koruma-derive/docs/ARCHITECTURE.md) | Procedural macros for code generation |
| `koruma-derive-core` | [ARCHITECTURE](crates/koruma-derive-core/docs/ARCHITECTURE.md) | Shared parsing infrastructure |
| `koruma-collection` | [ARCHITECTURE](crates/koruma-collection/docs/ARCHITECTURE.md) | Pre-built validators library |

## Crate Descriptions

### koruma

The main facade crate that users depend on. Re-exports functionality from `koruma-core` and `koruma-derive`, providing a single unified API. Users add only this crate to their dependencies and access all framework features through it.

**Key features:**

- `derive` (default) - Enables procedural macros
- `fluent` - i18n support
- `showcase` - Validator discovery

### koruma-core

Foundation crate containing the core traits that define the validation contract:

- `Validate<T>` - Primary validation trait
- `ValidationError` - Error type contract
- `ValidateExt` - Extension trait for validated structs
- `NewtypeValidation` - Newtype wrapper support
- `BuilderWithValue<T>` - Builder pattern support

Has no dependencies on other workspace crates.

### koruma-derive

Procedural macro crate providing:

- `#[koruma::validator]` - Define custom validators
- `#[derive(Koruma)]` - Generate validation code for structs
- `#[derive(KorumaAllDisplay)]` - Display for error enums
- `#[derive(KorumaAllFluent)]` - Fluent i18n for error enums

Generates type-safe error structs with per-field error handling.

### koruma-derive-core

Shared parsing infrastructure for `#[koruma(...)]` attributes. Provides:

- `parse_field()` - Parse field-level attributes
- `parse_struct_options()` - Parse struct-level attributes
- `FieldInfo`, `ValidatorAttr`, `StructOptions` types

Designed for reuse by external tools.

### koruma-collection

Comprehensive library of ~20 pre-built validators:

**String:** Alphanumeric, ASCII, Contains, Matches, Prefix, Suffix, Pattern, Case

**Format:** Email, URL, Phone Number, Credit Card, IP Address

**Numeric:** Positive, Negative, NonPositive, NonNegative, Range

**Collection:** Len, NonEmpty

**General:** Required

Many validators are feature-gated to minimize dependencies.

## Dependency Graph

```
koruma (facade)
├── koruma-core (traits)
└── koruma-derive (macros)
    └── koruma-derive-core (parsing)

koruma-collection (validators)
└── koruma
```

## Quick Reference

### Define a Validator

```rust
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

### Validate a Struct

```rust
#[derive(Koruma)]
pub struct User {
    #[koruma(MinLength::builder().min(3))]
    name: String,
}
```

### Attribute Syntax

- `#[koruma(Validator)]` - Single validator
- `#[koruma(V1, V2)]` - Multiple validators
- `#[koruma(V::builder().item1(...).item2(...))]` or `#[koruma(V(item1 = ... item2 = ...))]` - With configuration
- `#[koruma(V::<_>)]` - Type inference
- `#[koruma(nested)]` - Nested validation
- `#[koruma(each(V))]` - Collection element validation
- `#[koruma(newtype)]` - Newtype mode (struct-level)
- `#[koruma(try_new)]` - Validated constructor (struct-level)
