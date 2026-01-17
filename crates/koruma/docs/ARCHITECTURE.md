# koruma Architecture

## Overview

The `koruma` crate is the main facade/umbrella crate for the Koruma validation framework. It serves as the primary public API entry point, re-exporting functionality from the underlying crates while providing a unified interface for users.

## Role in the Workspace

```
koruma (this crate - facade)
├── koruma-core (re-exports traits & types)
└── koruma-derive (re-exports proc-macros when enabled)
```

## Module Structure

```
src/
└── lib.rs    # Re-exports and feature coordination
```

The crate is intentionally minimal, consisting primarily of re-exports rather than original code. This design:

- Provides a single dependency for users
- Allows internal refactoring without breaking the public API
- Enables feature-gated functionality

## Features

| Feature | Default | Description |
|---------|---------|-------------|
| `derive` | Yes | Enables procedural macros from `koruma-derive` |
| `fluent` | No | Enables i18n support via `es-fluent` |
| `showcase` | No | Enables validator discovery via `inventory` |

## Re-exports

### From `koruma-core`

- `Validate<T>` - Core validation trait
- `ValidationError` - Error trait for generated error structs
- `BuilderWithValue<T>` - Builder pattern support
- `ValidateExt` - Extension trait for validated structs
- `NewtypeValidation` - Marker trait for newtype wrappers

### From `koruma-derive` (when `derive` feature enabled)

- `#[koruma::validator]` - Attribute macro for defining validators
- `#[derive(Koruma)]` - Derive macro for validation code generation
- `#[derive(KorumaAllDisplay)]` - Display implementation for error enums
- `#[derive(KorumaAllFluent)]` - Fluent i18n for error enums

### Additional Re-exports

- `bon` - Builder pattern library (always available)
- `es_fluent` types (when `fluent` feature enabled)
- `inventory` (when `showcase` feature enabled)

## Usage Pattern

Users add only `koruma` to their dependencies:

```toml
[dependencies]
koruma = "0.1"
```

Then access all functionality through a single import:

```rust
use koruma::*;
```

## Design Decisions

### Facade Pattern

The crate uses the facade pattern to:

1. Hide internal crate boundaries from users
1. Provide stable public API independent of internal organization
1. Allow semver-compatible internal refactoring

### Feature Propagation

Features are propagated to underlying crates:

- `derive` enables `koruma-derive` as optional dependency
- `fluent` enables `es-fluent` and `koruma-core/fluent`
- `showcase` enables `inventory` and `koruma-core/showcase`

### Minimal Footprint

The facade adds no runtime overhead - all re-exports are compile-time only.
