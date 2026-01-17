# koruma-derive-core Architecture

## Overview

The `koruma-derive-core` crate provides shared parsing infrastructure for procedural macros. It enables external consumers (such as tooling, IDEs, or code generators) to analyze koruma validation metadata without depending on the full macro crate.

## Role in the Workspace

```
koruma-derive-core (this crate - parsing)
    ↑
    └── Used by: koruma-derive
```

## Module Structure

```
src/
├── lib.rs      # Public exports
├── parse.rs    # Attribute parsing logic
├── utils.rs    # Type manipulation utilities
└── tests/      # Unit tests
```

## Core Types

### `FieldInfo`

Represents parsed metadata about a struct field:

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

Represents a parsed validator with its configuration:

```rust
pub struct ValidatorAttr {
    pub path: Path,           // e.g., MinLength
    pub args: TokenStream,    // e.g., ::builder().min(3)
    pub infer_type: bool,     // Whether ::<_> was used
}
```

### `StructOptions`

Parsed struct-level configuration:

```rust
pub struct StructOptions {
    pub newtype: bool,
    pub try_new: bool,
    pub error_name: Option<Ident>,
}
```

## Key Functions

### `parse_field(field: &Field) -> Option<FieldInfo>`

Parses a single struct field's `#[koruma(...)]` attributes:

```rust
let field_info = parse_field(&field)?;
for validator in field_info.validators {
    // Process each validator
}
```

### `parse_struct_options(attrs: &[Attribute]) -> StructOptions`

Parses struct-level attributes:

```rust
let options = parse_struct_options(&input.attrs);
if options.newtype {
    // Handle newtype mode
}
```

## Utility Functions

### Type Inference Helpers

```rust
// Extract inner type from Option<T>
fn extract_option_inner(ty: &Type) -> Option<&Type>

// Extract inner type from Vec<T>
fn extract_vec_inner(ty: &Type) -> Option<&Type>

// Check if type is Option
fn is_option_type(ty: &Type) -> bool

// Check if type is Vec
fn is_vec_type(ty: &Type) -> bool
```

### Path Manipulation

```rust
// Get last segment of path (e.g., "MinLength" from "validators::MinLength")
fn path_last_segment(path: &Path) -> &PathSegment

// Convert path to snake_case identifier
fn path_to_snake_case(path: &Path) -> Ident
```

## Parsing Flow

1. **Attribute Discovery**: Find all `#[koruma(...)]` attributes on a field
1. **Token Parsing**: Parse the attribute contents as a comma-separated list
1. **Validator Extraction**: For each item, determine if it's:
   - A validator expression (path + optional builder)
   - The `nested` keyword
   - An `each(...)` wrapper
1. **Type Analysis**: Infer types when `::<_>` syntax is used
1. **Struct Assembly**: Combine field info into `FieldInfo`

## Attribute Syntax Supported

```rust
// Single validator
#[koruma(MinLength)]

// Validator with builder
#[koruma(MinLength::builder().min(3))]

// Multiple validators
#[koruma(MinLength::builder().min(3), MaxLength::builder().max(100))]

// Type inference
#[koruma(Range::<_>::builder().min(0).max(150))]

// Nested validation
#[koruma(nested)]

// Collection validation
#[koruma(each(MinLength::builder().min(1)))]

// Combined
#[koruma(nested, each(MinLength::builder().min(1)))]
```

## Design Decisions

### Separation from Code Generation

Parsing is separate from `koruma-derive` to:

- Enable external tools to understand koruma metadata
- Allow IDE plugins to provide autocomplete/validation
- Reduce coupling between parsing and generation

### syn-Based Parsing

Uses `syn` for robust Rust AST parsing:

- Handles all valid Rust syntax
- Provides good error messages
- Well-maintained ecosystem

### Minimal Processing

This crate only parses - it doesn't:

- Generate any code
- Validate semantic correctness
- Make decisions about code structure

This keeps it focused and reusable.

## Dependencies

| Dependency | Purpose |
|------------|---------|
| `syn` (full) | Rust AST parsing |
| `proc-macro2` | TokenStream manipulation |
| `quote` | Token construction |
| `syn-cfg-attr` | Custom attribute helpers |

## Usage by External Tools

External tools can depend on this crate to analyze koruma metadata:

```rust
use koruma_derive_core::{parse_field, parse_struct_options};
use syn::{parse_file, ItemStruct};

fn analyze_struct(code: &str) {
    let file = parse_file(code).unwrap();
    for item in file.items {
        if let Item::Struct(s) = item {
            let options = parse_struct_options(&s.attrs);
            for field in s.fields {
                if let Some(info) = parse_field(&field) {
                    // Analyze validators, nested fields, etc.
                }
            }
        }
    }
}
```
