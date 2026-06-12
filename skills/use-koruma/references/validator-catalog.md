# koruma-collection Validator Catalog

Use this reference when selecting built-in validators for application code. Configure feature
flags on the application's `koruma-collection` dependency.

## Feature Flags

- `fmt` (default): enables `Display` messages for validators.
- `full`: enables optional validator dependencies: `url`, `credit-card`, `phone-number`, `email`,
  `regex`, `smallvec`, and `rust_decimal`.
- `fluent`: enables `es-fluent` integration and enables `koruma/fluent`.
- `full-fluent`: enables `full` and `fluent`.

Validator-specific optional flags:

- `credit-card` for `format::CreditCardValidation`
- `email` for `format::EmailValidation`
- `phone-number` for `format::PhoneNumberValidation`
- `url` for `format::UrlValidation`
- `regex` for `string::PatternValidation`
- `smallvec` for `collection::HasLen` support on `SmallVec`
- `rust_decimal` for `numeric::Numeric` support on `rust_decimal::Decimal`

## String Validators

Import with `use koruma_collection::string;`.

- `string::AlphanumericValidation<T>`: input contains only letters and numbers.
- `string::AsciiValidation<T>`: input is ASCII only.
- `string::ContainsValidation<T>`: input contains a substring. Configure with
  `string::ContainsValidation::<_>::substring("abc")`.
- `string::MatchesValidation<T>`: input equals another value. Configure with
  `string::MatchesValidation::<_>::other("secret".to_string())`.
- `string::PatternValidation<T>`: input matches a compiled regex; requires `regex`. Configure with
  `string::PatternValidation::<_>::pattern(regex::Regex::new("...").unwrap())`.
- `string::PrefixValidation<T>`: input starts with a prefix. Configure with
  `string::PrefixValidation::<_>::prefix("usr_")`.
- `string::SuffixValidation<T>`: input ends with a suffix. Configure with
  `string::SuffixValidation::<_>::suffix(".rs")`.

`MatchesValidation` and `PatternValidation` use generic error messages and do not echo the
compared value or regex pattern.

## Format Validators

Import with `use koruma_collection::format;`.

- `format::IpValidation<T>`: valid IP address. Configure with
  `format::IpValidation::<_>::kind(format::IpKind::Any)`,
  `format::IpValidation::<_>::kind(format::IpKind::V4)`, or
  `format::IpValidation::<_>::kind(format::IpKind::V6)`.
- `format::EmailValidation<T>`: valid email address; requires `email`.
- `format::PhoneNumberValidation<T>`: valid phone number; requires `phone-number`.
- `format::UrlValidation<T>`: valid URL; requires `url`.
- `format::CreditCardValidation<T>`: valid credit card number; requires `credit-card`.

## Numeric Validators

Import with `use koruma_collection::numeric;`.

- `numeric::PositiveValidation<T>`: `value > 0`.
- `numeric::NonNegativeValidation<T>`: `value >= 0`.
- `numeric::NonPositiveValidation<T>`: `value <= 0`.
- `numeric::NegativeValidation<T>`: `value < 0`.
- `numeric::RangeValidation<T>`: value is within the configured range. Configure with
  `numeric::RangeValidation::<_>::min(min).max(max)`.

`RangeValidation` is inclusive by default. Chain `.exclusive_min(true)` or
`.exclusive_max(true)` when needed. It renders interval notation such as `[min, max]` or
`(min, max]`.

Primitive integers and floats implement `numeric::Numeric` out of the box. Implement
`numeric::Numeric::zero()` for custom numeric-like types. Enable `rust_decimal` for
`rust_decimal::Decimal`.

## Collection Validators

Import with `use koruma_collection::collection;`.

- `collection::LenValidation<T>`: collection length is within the inclusive range. Configure with
  `collection::LenValidation::<_>::min(1).max(10)`.
- `collection::NonEmptyValidation<T>`: collection or string is not empty.

`collection::HasLen` is implemented for common standard collections, strings, slices, arrays, and
optionally `SmallVec`. For `String` and `str`, length counts Unicode scalar values (`char`s), not
UTF-8 bytes.

## General Validators

Import with `use koruma_collection::general;`.

- `general::RequiredValidation<Option<T>>`: option is `Some`; write it as
  `#[koruma(general::RequiredValidation::<Option<_>>)]` on optional fields.

`RequiredValidation` reports missing values, not empty strings or empty collections. Use
`collection::NonEmptyValidation<_>` for emptiness checks. Its `skip_capture` behavior means
`Option<NonCloneType>` fields do not require `Clone` just to report a missing-value error. Write it
with `Option<_>` so Koruma validates the whole `Option<T>` and can report `None` as a missing value.
