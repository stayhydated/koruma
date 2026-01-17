# koruma-derive-core

Shared parsing infrastructure for Koruma procedural macros.

## Overview

This crate provides the attribute parsing logic used by `koruma-derive`. It's designed to be reusable by external tools that need to analyze Koruma validation metadata without depending on the full macro crate.

## Installation

```toml
[dependencies]
koruma-derive-core = "0.1"
```

## Use Cases

- IDE plugins for autocomplete/validation
- Code generators that need to understand Koruma structs
- Static analysis tools
- Documentation generators

## Core Types

### `FieldInfo`

Parsed metadata about a struct field:

```rust
pub struct FieldInfo {
    pub field_name: Ident,
    pub field_type: Type,
    pub validators: Vec<ValidatorAttr>,
    pub is_nested: bool,
    pub each_validators: Vec<ValidatorAttr>,
}
```

### `ValidatorAttr`

A parsed validator with its configuration:

```rust
pub struct ValidatorAttr {
    pub path: Path,           // e.g., MinLength
    pub args: TokenStream,    // e.g., ::builder().min(3)
    pub infer_type: bool,     // Whether ::<_> was used
}
```

### `StructOptions`

Struct-level configuration:

```rust
pub struct StructOptions {
    pub newtype: bool,
    pub try_new: bool,
    pub error_name: Option<Ident>,
}
```

## Key Functions

### `parse_field`

Parse a struct field's `#[koruma(...)]` attributes:

```rust
use koruma_derive_core::parse_field;
use syn::Field;

fn analyze(field: &Field) {
    if let Some(info) = parse_field(field) {
        for validator in &info.validators {
            println!("Validator: {:?}", validator.path);
        }
    }
}
```

### `parse_struct_options`

Parse struct-level attributes:

```rust
use koruma_derive_core::parse_struct_options;
use syn::Attribute;

fn analyze(attrs: &[Attribute]) {
    let options = parse_struct_options(attrs);
    if options.newtype {
        println!("This is a newtype wrapper");
    }
}
```

## Utility Functions

```rust
// Type inspection
fn is_option_type(ty: &Type) -> bool;
fn is_vec_type(ty: &Type) -> bool;
fn extract_option_inner(ty: &Type) -> Option<&Type>;
fn extract_vec_inner(ty: &Type) -> Option<&Type>;

// Path manipulation
fn path_last_segment(path: &Path) -> &PathSegment;
fn path_to_snake_case(path: &Path) -> Ident;
```

## Example: Analyzing a Koruma Struct

```rust
use koruma_derive_core::{parse_field, parse_struct_options};
use syn::{parse_file, Item};

fn analyze_file(source: &str) {
    let file = parse_file(source).unwrap();

    for item in file.items {
        if let Item::Struct(s) = item {
            let options = parse_struct_options(&s.attrs);
            println!("Struct: {}", s.ident);
            println!("  newtype: {}", options.newtype);

            for field in s.fields {
                if let Some(info) = parse_field(&field) {
                    println!("  Field: {}", info.field_name);
                    println!("    validators: {}", info.validators.len());
                    println!("    nested: {}", info.is_nested);
                }
            }
        }
    }
}
```

## Dependencies

- `syn` - Rust AST parsing
- `proc-macro2` - TokenStream handling
- `quote` - Token construction

## Documentation

- [Architecture](docs/ARCHITECTURE.md)
- [API Documentation](https://docs.rs/koruma-derive-core)
