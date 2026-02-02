# koruma-collection

[![Docs](https://docs.rs/koruma-collection/badge.svg)](https://docs.rs/koruma-collection/)
[![Crates.io](https://img.shields.io/crates/v/koruma-collection.svg)](https://crates.io/crates/koruma-collection)
[![Crowdin](https://badges.crowdin.net/koruma-collection/localized.svg)](https://crowdin.com/project/koruma-collection)

A curated set of validators built on top of `koruma`, organized by domain:
string, format, numeric, collection, and general-purpose validators.

## Installation

```toml
[dependencies]
koruma-collection = { version = "0.3", features = ["full"] }
```

## Common features

- `full`: enables all validators and optional dependencies.
- `fmt` (default): enables `Display` implementations for validators.
- `fluent`: enables fluent/i18n integration.
- `full-fluent`: `full` + `fluent`.
- `showcase`: enables validator registry support.

## Example

```rust
use koruma::Koruma;
use koruma_collection::string::ContainsValidation;

#[derive(Koruma)]
struct User {
    #[koruma(ContainsValidation::<_>(substring = "@"))]
    email: String,
}
```
