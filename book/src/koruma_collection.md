# Use koruma-collection

Use `koruma-collection` when a built-in string, format, numeric, collection, or presence rule
matches the application's requirement.

## Add the dependency

Default features include `Display` messages and every validator marked `default` below:

```toml
[dependencies]
koruma = "0.12"
koruma-collection = "0.12"
```

Enable every optional validator and type integration with `full`:

```toml
koruma-collection = { version = "0.12", features = ["full"] }
```

## Choose features

| Feature | Adds |
| --- | --- |
| `fmt` (default) | `Display` messages |
| `full` | All optional validators plus `SmallVec` and `rust_decimal::Decimal` support |
| `fluent` | es-fluent messages and `koruma/fluent` |
| `full-fluent` | `full` and `fluent` |

Optional capabilities can also be enabled individually:

- `credit-card`, `email`, `phone-number`, and `url` enable their format validators.
- `regex` enables `string::PatternValidation`.
- `smallvec` extends `collection::HasLen`.
- `rust_decimal` extends `numeric::Numeric`.

## String validators

Import these validators from `koruma_collection::string`.

| Validator | Rule | Feature |
| --- | --- | --- |
| `AlphanumericValidation<T>` | Contains only Unicode letters and numbers | default |
| `AsciiValidation<T>` | Contains only ASCII | default |
| `CanonicalFormValidation<T>` | Satisfies a caller-provided canonical predicate | default |
| `ContainsValidation<T>` | Contains a substring | default |
| `MatchesValidation<T>` | Equals another value | default |
| `PatternValidation<T>` | Matches a compiled regular expression | `regex` |
| `PrefixValidation<T>` | Starts with a prefix | default |
| `SuffixValidation<T>` | Ends with a suffix | default |

Configure each validator with its dot-chain setter:

| Validator | Configuration |
| --- | --- |
| `CanonicalFormValidation::<_>` | `.predicate(is_canonical)` |
| `ContainsValidation::<_>` | `.substring("abc")` |
| `MatchesValidation::<_>` | `.other("expected".to_string())` |
| `PatternValidation::<_>` | `.pattern(compiled_regex)` |
| `PrefixValidation::<_>` | `.prefix("usr_")` |
| `SuffixValidation::<_>` | `.suffix(".rs")` |

Put multiple rules for a field in one attribute:

```rust
# extern crate koruma;
# extern crate koruma_collection;
use koruma::Koruma;
use koruma_collection::{collection, string};

#[derive(Koruma)]
struct Handle {
    #[koruma(
        collection::NonEmptyValidation::<_>,
        string::AsciiValidation::<_>,
        string::AlphanumericValidation::<_>,
    )]
    value: String,
}

assert!(Handle { value: "alice42".to_string() }.validate().is_ok());
assert!(Handle { value: "alice-42".to_string() }.validate().is_err());
```

`CanonicalFormValidation` checks the predicate without modifying the value.
`AlphanumericValidation` is Unicode-aware and accepts an empty string; combine it with
`AsciiValidation` or `collection::NonEmptyValidation` when those are separate requirements.
`PatternValidation` takes a compiled `regex::Regex`. Handle invalid user-supplied patterns when
constructing the regex. For a fixed pattern in an attribute, use a construction expression such
as `regex::Regex::new(r"^[a-z0-9_]+$").expect("valid handle pattern")`.

## Format validators

Import these validators from `koruma_collection::format`.

| Validator | Rule | Feature |
| --- | --- | --- |
| `IpValidation<T>` | Parses as any, IPv4, or IPv6 address | default |
| `EmailValidation<T>` | Parses as an email address | `email` |
| `PhoneNumberValidation<T>` | Parses as a phone number and passes its validity check | `phone-number` |
| `UrlValidation<T>` | Parses as a URL | `url` |
| `CreditCardValidation<T>` | Passes credit-card validation | `credit-card` |

Choose an IP kind with `.kind(format::IpKind::Any)`, `V4`, or `V6`.
`UrlValidation` accepts every scheme supported by `url::Url::parse`; add an application rule
when only selected schemes are allowed.

## Numeric validators

Import these validators from `koruma_collection::numeric`.

| Validator | Rule | Feature |
| --- | --- | --- |
| `PositiveValidation<T>` | `value > 0` | default |
| `NonNegativeValidation<T>` | `value >= 0` | default |
| `NonPositiveValidation<T>` | `value <= 0` | default |
| `NegativeValidation<T>` | `value < 0` | default |
| `RangeValidation<T>` | Falls between configured bounds | default |

Configure a range with `.min(...).max(...)`. Bounds are inclusive unless
`.exclusive_min(true)` or `.exclusive_max(true)` is set, and messages use matching interval
notation such as `(1, 10]`.

Primitive integers and floats implement `numeric::Numeric`. Implement `Numeric::zero()` for a
custom numeric-like type, or enable `rust_decimal` for `rust_decimal::Decimal`.

## Collection validators

Import these validators from `koruma_collection::collection`.

| Validator | Rule | Feature |
| --- | --- | --- |
| `LenValidation<T>` | Length falls within an inclusive range | default |
| `NonEmptyValidation<T>` | Length is greater than zero | default |

Configure length with `.min(...).max(...)`. `collection::HasLen` supports strings, slices,
arrays, `Vec`, `VecDeque`, standard maps and sets, and `SmallVec` with its feature. String
length counts Unicode scalar values rather than UTF-8 bytes.

## Presence validation

Import `RequiredValidation` from `koruma_collection::general` and target the whole option:

```rust,ignore
#[koruma(general::RequiredValidation::<Option<_>>)]
pub display_name: Option<String>,
```

This rule rejects `None`; it does not reject an empty string or collection. Combine it with
`NonEmptyValidation` when presence and non-empty content are separate requirements.
