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
- `full`: enables optional validator dependencies (`url`, `credit-card`, `phone-number`, `email`, `regex`, `smallvec`, `rust_decimal`).
- `fluent`: enables i18n integration with [es-fluent](https://github.com/stayhydated/es-fluent).
- `full-fluent`: `full` + `fluent`.

Validator-specific optional flags:

- `credit-card` for `format::CreditCardValidation`
- `email` for `format::EmailValidation`
- `phone-number` for `format::PhoneNumberValidation`
- `url` for `format::UrlValidation`
- `regex` for `string::PatternValidation`
- `smallvec` for `collection::HasLen` support on `SmallVec`
- `rust_decimal` for `numeric::Numeric` support on `rust_decimal::Decimal`

## Complete validator catalog

### String validators (`koruma_collection::string`)

| Validator                   | Rule                     | Example attribute                                                                                | Feature |
| --------------------------- | ------------------------ | ------------------------------------------------------------------------------------------------ | ------- |
| `AlphanumericValidation<T>` | Only letters and numbers | `#[koruma(string::AlphanumericValidation::<_>)]`                                                   | `default`  |
| `AsciiValidation<T>`        | ASCII-only input         | `#[koruma(string::AsciiValidation::<_>)]`                                                          | `default`  |
| `ContainsValidation<T>`     | Contains substring       | `#[koruma(string::ContainsValidation::<_>::substring("abc"))]`                                    | `default`  |
| `MatchesValidation<T>`      | Equals expected value    | `#[koruma(string::MatchesValidation::<_>::other("secret".to_string()))]`                          | `default`  |
| `PatternValidation<T>`      | Matches regex pattern    | `#[koruma(string::PatternValidation::<_>::pattern(regex::Regex::new(r"^[a-z0-9_]+$").unwrap()))]` | `regex` |
| `PrefixValidation<T>`       | Starts with prefix       | `#[koruma(string::PrefixValidation::<_>::prefix("usr_"))]`                                        | `default`  |
| `SuffixValidation<T>`       | Ends with suffix         | `#[koruma(string::SuffixValidation::<_>::suffix(".rs"))]`                                         | `default`  |

`MatchesValidation` and `PatternValidation` use generic error messages and do not echo the
compared value or regex pattern. `PatternValidation` stores a compiled `regex::Regex`, so invalid
patterns fail during construction instead of during validation.

### Format validators (`koruma_collection::format`)

| Validator                  | Rule                         | Example attribute                                               | Feature        |
| -------------------------- | ---------------------------- | --------------------------------------------------------------- | -------------- |
| `IpValidation<T>`          | Valid IP (`Any`, `V4`, `V6`) | `#[koruma(format::IpValidation::<_>::kind(format::IpKind::V4))]` | `default`       |
| `EmailValidation<T>`       | Valid email address          | `#[koruma(format::EmailValidation::<_>)]`                         | `email`        |
| `PhoneNumberValidation<T>` | Valid phone number           | `#[koruma(format::PhoneNumberValidation::<_>)]`                   | `phone-number` |
| `UrlValidation<T>`         | Valid URL                    | `#[koruma(format::UrlValidation::<_>)]`                           | `url`          |
| `CreditCardValidation<T>`  | Valid credit card number     | `#[koruma(format::CreditCardValidation::<_>)]`                    | `credit-card`  |

### Numeric validators (`koruma_collection::numeric`)

| Validator                  | Rule                                           | Example attribute                                                                  | Feature |
| -------------------------- | ---------------------------------------------- | ---------------------------------------------------------------------------------- | ------- |
| `PositiveValidation<T>`    | `value > 0`                                    | `#[koruma(numeric::PositiveValidation::<_>)]`                                        | `default`  |
| `NonNegativeValidation<T>` | `value >= 0`                                   | `#[koruma(numeric::NonNegativeValidation::<_>)]`                                     | `default`  |
| `NonPositiveValidation<T>` | `value <= 0`                                   | `#[koruma(numeric::NonPositiveValidation::<_>)]`                                     | `default`  |
| `NegativeValidation<T>`    | `value < 0`                                    | `#[koruma(numeric::NegativeValidation::<_>)]`                                        | `default`  |
| `RangeValidation<T>`       | Between `min` and `max` (inclusive by default) | `#[koruma(numeric::RangeValidation::<_>::min(0).max(100).exclusive_max(true))]` | `default`  |

Primitive integers and floats implement `numeric::Numeric` out of the box. Enable the `decimal`
feature to add `rust_decimal::Decimal`. Custom numeric-like types can opt in by implementing
`numeric::Numeric::zero()`.

`RangeValidation` messages use interval notation such as `[min, max]` or `(min, max]` so exclusive
bounds are reflected directly in the rendered error.

### Collection validators (`koruma_collection::collection`)

| Validator               | Rule                           | Example attribute                                            | Feature |
| ----------------------- | ------------------------------ | ------------------------------------------------------------ | ------- |
| `LenValidation<T>`      | Length within `[min, max]`     | `#[koruma(collection::LenValidation::<_>::min(1).max(10))]` | `default`  |
| `NonEmptyValidation<T>` | Collection/string is not empty | `#[koruma(collection::NonEmptyValidation::<_>)]`               | `default`  |

`collection::HasLen` is implemented for common standard types (`String`, `str`,
arrays/slices, `Vec`, sets/maps, etc.) and optionally for `SmallVec` with the
`smallvec` feature. For `String` and `str`, `LenValidation` counts Unicode scalar
values (`char`s), not UTF-8 bytes.

Built-in validators in the collection, string, format, and numeric modules that
do not render the failing input use `#[koruma(value(capture = skip))]` internally,
so derived validation does not require input types to implement `Clone` just to
store an error.

### General validators (`koruma_collection::general`)

| Validator                       | Rule                  | Example attribute                                   | Feature |
| ------------------------------- | --------------------- | --------------------------------------------------- | ------- |
| `RequiredValidation<Option<T>>` | Option must be `Some` | `#[koruma(full(general::RequiredValidation::<_>))]` | `default`  |

`RequiredValidation` reports missing values, not empty strings or empty collections. Use
`collection::NonEmptyValidation::<_>` when you need an emptiness check. It also uses
`#[koruma(value(capture = skip))]` internally, so `Option<NonCloneType>` fields do not need `Clone`
just to report a missing-value error. Wrap it in `full(...)` so koruma validates the whole
`Option<T>` instead of unwrapping `Some(T)`.

## Example

```rust
use koruma::{Koruma, KorumaAllDisplay};
use koruma_collection::{collection, general, numeric, string};

#[derive(Koruma, KorumaAllDisplay)]
struct SignupInput {
    #[koruma(collection::NonEmptyValidation::<_>)]
    username: String,

    #[koruma(string::AsciiValidation::<_>, string::AlphanumericValidation::<_>)]
    handle: String,

    #[koruma(numeric::RangeValidation::<_>::min(13_u8).max(120_u8))]
    age: u8,

    #[koruma(full(general::RequiredValidation::<_>))]
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

Validator configuration uses direct validator setter chains, so IDE completion works after the first setter method.
