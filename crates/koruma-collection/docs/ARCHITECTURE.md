# koruma-collection Architecture

## Overview

The `koruma-collection` crate provides a comprehensive library of pre-built validators for common validation scenarios. It serves as a batteries-included companion to the core koruma framework.

## Role in the Workspace

```
koruma-collection (this crate - validators library)
    │
    └── koruma (uses framework)
```

## Module Structure

```
src/
├── lib.rs                    # Public exports and feature gates
├── i18n.rs                   # Fluent i18n integration
└── validators/
    ├── mod.rs                # Validator module exports
    ├── string/               # String validators
    │   ├── mod.rs
    │   ├── alphanumeric.rs
    │   ├── ascii.rs
    │   ├── contains.rs
    │   ├── matches.rs
    │   ├── prefix.rs
    │   ├── suffix.rs
    │   ├── pattern.rs        # (regex feature)
    │   └── en/
    │       └── case.rs       # Case validation
    ├── format/               # Format validators
    │   ├── mod.rs
    │   ├── email.rs          # (email feature)
    │   ├── url.rs            # (url feature)
    │   ├── phone_number.rs   # (phone-number feature)
    │   ├── credit_card.rs    # (credit-card feature)
    │   └── ip.rs
    ├── numeric/              # Numeric validators
    │   ├── mod.rs
    │   ├── positive.rs
    │   ├── negative.rs
    │   ├── non_positive.rs
    │   ├── non_negative.rs
    │   └── range.rs
    ├── collection/           # Collection validators
    │   ├── mod.rs
    │   ├── len.rs
    │   └── non_empty.rs
    └── general/              # General validators
        ├── mod.rs
        └── required.rs
```

## Validator Categories

### String Validators

| Validator | Description | Feature |
|-----------|-------------|---------|
| `AlphanumericValidation` | Only alphanumeric characters | default |
| `AsciiValidation` | Only ASCII characters | default |
| `ContainsValidation` | Contains substring | default |
| `MatchesValidation` | Exact string match | default |
| `PrefixValidation` | Starts with prefix | default |
| `SuffixValidation` | Ends with suffix | default |
| `PatternValidation` | Regex pattern match | `regex` |
| `CaseValidation` | Case validation (upper/lower/mixed) | default |

### Format Validators

| Validator | Description | Feature |
|-----------|-------------|---------|
| `EmailValidation` | Valid email format | `email` |
| `UrlValidation` | Valid URL format | `url` |
| `PhoneNumberValidation` | Valid phone number | `phone-number` |
| `CreditCardValidation` | Valid credit card | `credit-card` |
| `IpValidation` | Valid IP address (v4/v6) | default |

### Numeric Validators

| Validator | Description | Feature |
|-----------|-------------|---------|
| `PositiveValidation` | Value > 0 | default |
| `NegativeValidation` | Value < 0 | default |
| `NonPositiveValidation` | Value \<= 0 | default |
| `NonNegativeValidation` | Value >= 0 | default |
| `RangeValidation` | min \<= value \<= max | default |

### Collection Validators

| Validator | Description | Feature |
|-----------|-------------|---------|
| `LenValidation` | Length constraints (min/max/exact) | default |
| `NonEmptyValidation` | Collection not empty | default |

### General Validators

| Validator | Description | Feature |
|-----------|-------------|---------|
| `RequiredValidation` | Option field is Some | default |

## Helper Traits

### `StringLike`

Alias for `AsRef<str>`, allows validators to work with `String`, `&str`, `Cow<str>`, etc.

### `Numeric`

Trait bound for numeric validators:

```rust
pub trait Numeric: PartialOrd + Default + Copy + Display {}
```

Implemented for all standard numeric types.

### `HasLen`

Trait for types with length:

```rust
pub trait HasLen {
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
}
```

Implemented for `Vec`, `HashMap`, `String`, slices, arrays, and `SmallVec`.

## Internationalization

The crate provides i18n support via Fluent:

```
i18n/
├── en/
│   └── validators.ftl
└── fr/
    └── validators.ftl
```

### Usage

```rust
use koruma_collection::i18n::FLUENT;

let msg = FLUENT.format("en", "min-length-error", &args);
```

## Features

| Feature | Default | Description |
|---------|---------|-------------|
| `fmt` | Yes | Display message formatting |
| `fluent` | No | i18n via es-fluent |
| `regex` | No | PatternValidation |
| `email` | No | EmailValidation |
| `url` | No | UrlValidation |
| `phone-number` | No | PhoneNumberValidation |
| `credit-card` | No | CreditCardValidation |
| `full` | No | Enable all validators |

## Design Decisions

### Feature-Gated Dependencies

Heavy dependencies (regex, phonenumber, etc.) are feature-gated to:

- Reduce default compile time
- Allow users to opt-in to needed validators
- Keep binary size minimal

### Generic Validators

Most validators are generic over their input type:

- `RangeValidation<T: Numeric>` works with any numeric type
- `LenValidation` works with anything implementing `HasLen`
- String validators work with anything `AsRef<str>`

### Consistent API

All validators follow the same pattern:

1. Struct with configuration fields
1. `#[koruma(value)]` field for the value
1. `validate(&self, value: &T) -> bool` method
1. Optional `Display` implementation for error messages

### Builder Pattern

All validators use `bon` builders for construction:

```rust
MinLength::builder().min(3).build()
```

## Adding New Validators

1. Create file in appropriate category directory
1. Define struct with `#[koruma::validator]`
1. Implement `validate` method
1. Add feature flag if needed
1. Export from category `mod.rs`
1. Add Fluent messages if using i18n
