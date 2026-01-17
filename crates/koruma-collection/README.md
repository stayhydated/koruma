# koruma-collection

A comprehensive library of pre-built validators for the Koruma validation framework.

## Installation

```toml
[dependencies]
koruma-collection = "0.1"
```

Or with specific features:

```toml
[dependencies]
koruma-collection = { version = "0.1", features = ["email", "url", "regex"] }
```

## Available Validators

### String Validators

| Validator | Description | Usage |
|-----------|-------------|-------|
| `AlphanumericValidation` | Only alphanumeric characters | `AlphanumericValidation::builder()` |
| `AsciiValidation` | Only ASCII characters | `AsciiValidation::builder()` |
| `ContainsValidation` | Contains substring | `ContainsValidation::builder().needle("search")` |
| `MatchesValidation` | Exact string match | `MatchesValidation::builder().expected("value")` |
| `PrefixValidation` | Starts with prefix | `PrefixValidation::builder().prefix("pre")` |
| `SuffixValidation` | Ends with suffix | `SuffixValidation::builder().suffix("suf")` |
| `PatternValidation` | Regex pattern | `PatternValidation::builder().pattern(r"\d+")` |
| `CaseValidation` | Case validation | `CaseValidation::builder().case(Case::Upper)` |

### Format Validators

| Validator | Description | Feature | Usage |
|-----------|-------------|---------|-------|
| `EmailValidation` | Valid email | `email` | `EmailValidation::builder()` |
| `UrlValidation` | Valid URL | `url` | `UrlValidation::builder()` |
| `PhoneNumberValidation` | Valid phone | `phone-number` | `PhoneNumberValidation::builder()` |
| `CreditCardValidation` | Valid card | `credit-card` | `CreditCardValidation::builder()` |
| `IpValidation` | Valid IP | default | `IpValidation::builder().kind(IpKind::V4)` |

### Numeric Validators

| Validator | Description | Usage |
|-----------|-------------|-------|
| `PositiveValidation` | Value > 0 | `PositiveValidation::<i32>::builder()` |
| `NegativeValidation` | Value < 0 | `NegativeValidation::<i32>::builder()` |
| `NonPositiveValidation` | Value \<= 0 | `NonPositiveValidation::<i32>::builder()` |
| `NonNegativeValidation` | Value >= 0 | `NonNegativeValidation::<i32>::builder()` |
| `RangeValidation` | min \<= x \<= max | `RangeValidation::<i32>::builder().min(0).max(100)` |

### Collection Validators

| Validator | Description | Usage |
|-----------|-------------|-------|
| `LenValidation` | Length constraints | `LenValidation::builder().min(1).max(100)` |
| `NonEmptyValidation` | Not empty | `NonEmptyValidation::builder()` |

### General Validators

| Validator | Description | Usage |
|-----------|-------------|-------|
| `RequiredValidation` | Option is Some | `RequiredValidation::builder()` |

## Usage Example

```rust
use koruma::*;
use koruma_collection::*;

#[derive(Koruma)]
pub struct User {
    #[koruma(
        NonEmptyValidation::builder(),
        LenValidation::builder().min(3).max(50)
    )]
    username: String,

    #[koruma(EmailValidation::builder())]
    email: String,

    #[koruma(RangeValidation::<_>::builder().min(13).max(120))]
    age: u8,

    #[koruma(UrlValidation::builder())]
    website: Option<String>,
}
```

## Features

| Feature | Default | Description |
|---------|---------|-------------|
| `fmt` | Yes | Display implementations for error messages |
| `fluent` | No | i18n support via Fluent |
| `regex` | No | `PatternValidation` |
| `email` | No | `EmailValidation` |
| `url` | No | `UrlValidation` |
| `phone-number` | No | `PhoneNumberValidation` |
| `credit-card` | No | `CreditCardValidation` |
| `full` | No | Enable all validators |

### Enable All Validators

```toml
[dependencies]
koruma-collection = { version = "0.1", features = ["full"] }
```

## Internationalization

Enable the `fluent` feature for i18n support:

```toml
[dependencies]
koruma-collection = { version = "0.1", features = ["fluent"] }
```

Currently supported locales:

- English (`en`)
- French (`fr`)

## Helper Traits

### `StringLike`

Works with any type implementing `AsRef<str>`:

- `String`
- `&str`
- `Cow<str>`
- `Box<str>`

### `Numeric`

Works with all standard numeric types:

- `i8`, `i16`, `i32`, `i64`, `i128`, `isize`
- `u8`, `u16`, `u32`, `u64`, `u128`, `usize`
- `f32`, `f64`

### `HasLen`

Works with collections:

- `Vec<T>`
- `HashMap<K, V>`
- `String`
- `&str`
- `[T]` (slices)
- `[T; N]` (arrays)

## Documentation

- [Architecture](docs/ARCHITECTURE.md)
- [API Documentation](https://docs.rs/koruma-collection)
