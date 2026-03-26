# koruma-collection

[![Docs](https://docs.rs/koruma-collection/badge.svg)](https://docs.rs/koruma-collection/)
[![Crates.io](https://img.shields.io/crates/v/koruma-collection.svg)](https://crates.io/crates/koruma-collection)
[![Crowdin](https://badges.crowdin.net/koruma-collection/localized.svg)](https://crowdin.com/project/koruma-collection)

- [Demos](https://stayhydated.github.io/koruma/demos)

A curated set of validators built on top of `koruma`, organized by domain:
string, format, numeric, collection, and general-purpose validators.

```toml
[dependencies]
koruma-collection = { version = "*", features = ["full"] }
```

## Features

### Standard

- `full`: enables all validators and optional dependencies.
- `fmt` (default): enables `Display` implementations for validators.
- `fluent`: enables fluent/i18n integration and embedded translations.
- `full-fluent`: `full` + `fluent`.

### Format validators

- `credit-card`: validates a credit card number.
- `email`: validates an email address.
- `phone-number`: validates a phone number.
- `url`: validates a URL.
