# koruma-collection

[![Docs](https://docs.rs/koruma-collection/badge.svg)](https://docs.rs/koruma-collection/)
[![Crates.io](https://img.shields.io/crates/v/koruma-collection.svg)](https://crates.io/crates/koruma-collection)
[![Crowdin](https://badges.crowdin.net/koruma-collection/localized.svg)](https://crowdin.com/project/koruma-collection)

- [Demos](https://stayhydated.github.io/koruma/demos)

A curated set of validators built on top of `koruma`, organized by domain.

## Installation

```toml
[dependencies]
koruma-collection = { version = "*", features = ["full"] }
```

## Modules at a glance

```rust
use koruma_collection::{collection, format, general, numeric, string};
```

## Feature flags

- `fmt` (default): `Display` messages for validators.
- `full`: enables optional validator dependencies (`url`, `credit-card`, `phone-number`, `email`, `regex`, `smallvec`).
- `fluent`: enables i18n integration with [es-fluent](https://github.com/stayhydated/es-fluent).
- `full-fluent`: `full` + `fluent`.

Validator-specific optional flags:

- `credit-card` for `format::CreditCardValidation`
- `email` for `format::EmailValidation`
- `phone-number` for `format::PhoneNumberValidation`
- `url` for `format::UrlValidation`
- `regex` for `string::PatternValidation`
- `smallvec` for `collection::HasLen` support on `SmallVec`

## Complete validator catalog

### String validators (`koruma_collection::string`)

| Validator                   | Rule                     | Example attribute                                                                                | Feature |
| --------------------------- | ------------------------ | ------------------------------------------------------------------------------------------------ | ------- |
| `AlphanumericValidation<T>` | Only letters and numbers | `#[koruma(string::AlphanumericValidation::<_>::builder())]`                                                   | always  |
| `AsciiValidation<T>`        | ASCII-only input         | `#[koruma(string::AsciiValidation::<_>::builder())]`                                                          | always  |
| `ContainsValidation<T>`     | Contains substring       | `#[koruma(string::ContainsValidation::<_>::builder().substring("abc"))]`                                    | always  |
| `MatchesValidation<T>`      | Equals expected value    | `#[koruma(string::MatchesValidation::<_>::builder().other("secret".to_string()))]`                          | always  |
| `PatternValidation<T>`      | Matches regex pattern    | `#[koruma(string::PatternValidation::<_>::builder().pattern(regex::Regex::new(r"^[a-z0-9_]+$").unwrap()))]` | `regex` |
| `PrefixValidation<T>`       | Starts with prefix       | `#[koruma(string::PrefixValidation::<_>::builder().prefix("usr_"))]`                                        | always  |
| `SuffixValidation<T>`       | Ends with suffix         | `#[koruma(string::SuffixValidation::<_>::builder().suffix(".rs"))]`                                         | always  |

`MatchesValidation` and `PatternValidation` use generic error messages and do not echo the
compared value or regex pattern. `PatternValidation` stores a compiled `regex::Regex`, so invalid
patterns fail during construction instead of during validation.

### Format validators (`koruma_collection::format`)

| Validator                  | Rule                         | Example attribute                                               | Feature        |
| -------------------------- | ---------------------------- | --------------------------------------------------------------- | -------------- |
| `IpValidation<T>`          | Valid IP (`Any`, `V4`, `V6`) | `#[koruma(format::IpValidation::<_>::builder().kind(format::IpKind::V4))]` | always         |
| `EmailValidation<T>`       | Valid email address          | `#[koruma(format::EmailValidation::<_>::builder())]`                         | `email`        |
| `PhoneNumberValidation<T>` | Valid phone number           | `#[koruma(format::PhoneNumberValidation::<_>::builder())]`                   | `phone-number` |
| `UrlValidation<T>`         | Valid URL                    | `#[koruma(format::UrlValidation::<_>::builder())]`                           | `url`          |
| `CreditCardValidation<T>`  | Valid credit card number     | `#[koruma(format::CreditCardValidation::<_>::builder())]`                    | `credit-card`  |

### Numeric validators (`koruma_collection::numeric`)

| Validator                  | Rule                                           | Example attribute                                                                  | Feature |
| -------------------------- | ---------------------------------------------- | ---------------------------------------------------------------------------------- | ------- |
| `PositiveValidation<T>`    | `value > 0`                                    | `#[koruma(numeric::PositiveValidation::<_>::builder())]`                                        | always  |
| `NonNegativeValidation<T>` | `value >= 0`                                   | `#[koruma(numeric::NonNegativeValidation::<_>::builder())]`                                     | always  |
| `NonPositiveValidation<T>` | `value <= 0`                                   | `#[koruma(numeric::NonPositiveValidation::<_>::builder())]`                                     | always  |
| `NegativeValidation<T>`    | `value < 0`                                    | `#[koruma(numeric::NegativeValidation::<_>::builder())]`                                        | always  |
| `RangeValidation<T>`       | Between `min` and `max` (inclusive by default) | `#[koruma(numeric::RangeValidation::<_>::builder().min(0).max(100).exclusive_max(true))]` | always  |

Primitive integers and floats implement `numeric::Numeric` out of the box. Enable the `decimal`
feature to add `rust_decimal::Decimal`. Custom numeric-like types can opt in by implementing
`numeric::Numeric::zero()`.

`RangeValidation` messages use interval notation such as `[min, max]` or `(min, max]` so exclusive
bounds are reflected directly in the rendered error.

### Collection validators (`koruma_collection::collection`)

| Validator               | Rule                           | Example attribute                                            | Feature |
| ----------------------- | ------------------------------ | ------------------------------------------------------------ | ------- |
| `LenValidation<T>`      | Length within `[min, max]`     | `#[koruma(collection::LenValidation::<_>::builder().min(1).max(10))]` | always  |
| `NonEmptyValidation<T>` | Collection/string is not empty | `#[koruma(collection::NonEmptyValidation::<_>::builder())]`               | always  |

`collection::HasLen` is implemented for common standard types (`String`, `str`,
arrays/slices, `Vec`, sets/maps, etc.) and optionally for `SmallVec` with the
`smallvec` feature. For `String` and `str`, `LenValidation` counts Unicode scalar
values (`char`s), not UTF-8 bytes.

### General validators (`koruma_collection::general`)

| Validator                       | Rule                  | Example attribute                                   | Feature |
| ------------------------------- | --------------------- | --------------------------------------------------- | ------- |
| `RequiredValidation<Option<T>>` | Option must be `Some` | `#[koruma(general::RequiredValidation::<Option<_>>::builder())]` | always  |

`RequiredValidation` reports missing values, not empty strings or empty collections. Use
`collection::NonEmptyValidation::<_>::builder()` when you need an emptiness check. It uses
`#[koruma(value, skip_capture)]` internally, so `Option<NonCloneType>` fields do not need `Clone`
just to report a missing-value error.

## Example

```rust
use koruma::{Koruma, KorumaAllDisplay};
use koruma_collection::{collection, general, numeric, string};

#[derive(Koruma, KorumaAllDisplay)]
struct SignupInput {
    #[koruma(collection::NonEmptyValidation::<_>::builder())]
    username: String,

    #[koruma(string::AsciiValidation::<_>::builder(), string::AlphanumericValidation::<_>::builder())]
    handle: String,

    #[koruma(numeric::RangeValidation::<_>::builder().min(13_u8).max(120_u8))]
    age: u8,

    #[koruma(general::RequiredValidation::<Option<_>>::builder())]
    display_name: Option<String>,
}

let input = SignupInput {
    username: "".to_string(),
    handle: "bad-handle".to_string(),
    age: 8,
    display_name: None,
};

if let Err(errors) = input.validate() {
    if let Some(err) = errors.username().non_empty_validation() {
        println!("username: {err}");
    }

    if let Some(err) = errors.handle().ascii_validation() {
        println!("handle(ascii): {err}");
    }

    for err in errors.handle().all() {
        println!("handle(any): {err}");
    }
}
```

Validator configuration uses standard Rust builder chains, so IDE completion works on each setter method.
