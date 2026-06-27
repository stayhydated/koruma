# koruma

[![Build Status](https://github.com/stayhydated/koruma/actions/workflows/ci.yml/badge.svg)](https://github.com/stayhydated/koruma/actions/workflows/ci.yml)
[![Codecov](https://codecov.io/github/stayhydated/koruma/graph/badge.svg?token=34CV04UOU1)](https://codecov.io/github/stayhydated/koruma)
[![mdBook](https://img.shields.io/badge/docs-mdBook-black)](https://stayhydated.github.io/koruma/book/)
[![llms.txt](https://img.shields.io/badge/docs-llms.txt-blue)](https://stayhydated.github.io/koruma/llms.txt)
[![llms-full.txt](https://img.shields.io/badge/docs-llms--full.txt-blue)](https://stayhydated.github.io/koruma/llms-full.txt)
[![Docs](https://docs.rs/koruma/badge.svg)](https://docs.rs/koruma/)
[![Crates.io](https://img.shields.io/crates/v/koruma.svg)](https://crates.io/crates/koruma)

`koruma` is a per-field validation framework focused on:

1. **Type Safety**: Strongly typed validation error structs generated at compile time.
1. **Ergonomics**: Derive macros and validator attributes that minimize boilerplate.
1. **Developer Experience**: Optional constructors, nested/newtype validation, and i18n with [Project Fluent](https://projectfluent.org/).

## Installation

```toml
[dependencies]
koruma = { version = "*" }
```

## Feature flags

- `derive` (default): enables derive/attribute macros (`Koruma`, `KorumaAllDisplay`, `#[validator]`).
- `fluent`: enables localized error support for `KorumaAllFluent` (use with `es-fluent`).
- `internal-showcase`: internal validator registry hooks used by workspace demos; normal application code does not need it.

## koruma-collection

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

## Usage

### 1. Declare validators (generic + type-specific)

```rs
use koruma::{Validate, validator};
use std::fmt;

#[validator]
#[derive(Clone, Debug)]
pub struct NumberRangeValidation<T: PartialOrd + fmt::Display + Clone> {
    min: T,
    max: T,
    actual: T,
}

impl<T: PartialOrd + fmt::Display + Clone> Validate<T> for NumberRangeValidation<T> {
    fn validate(&self, value: &T) -> bool {
        value >= &self.min && value <= &self.max
    }
}

impl<T: PartialOrd + fmt::Display + Clone> fmt::Display for NumberRangeValidation<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Value {} must be between {} and {}",
            self.actual, self.min, self.max
        )
    }
}

#[validator]
#[derive(Clone, Debug)]
pub struct StringLengthValidation {
    min: usize,
    max: usize,
    input: String,
}

impl Validate<String> for StringLengthValidation {
    fn validate(&self, value: &String) -> bool {
        let len = value.chars().count();
        len >= self.min && len <= self.max
    }
}

impl fmt::Display for StringLengthValidation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "String length {} must be between {} and {} characters",
            self.input.chars().count(),
            self.min,
            self.max
        )
    }
}
```

`#[validator]` generates direct setter entrypoints, a hidden builder value hook, and a getter on the
validator type with the same name as the captured value field. An unmarked private field named
`actual`, `input`, or `value` is inferred as the captured value. If no conventional name exists,
Koruma can infer the value field when exactly one field is unmarked. Use `#[koruma(value)]` when
multiple unmarked fields remain.

Unannotated configuration fields generate direct setters. Use bare `#[koruma(setter)]`
when a configuration field is named `actual`, `input`, or `value`, or when marking configuration
fields lets Koruma infer the only remaining unmarked value field. Use `#[koruma(setter(...))]`
when a setter needs options.
Supported setter options are `into`, `required`, `name`, and `default`:

```rs
#[validator]
pub struct PrefixValidation<T> {
    #[koruma(setter(into))]
    prefix: String,
    #[koruma(skip_capture)]
    actual: Option<T>,
}
```

Setter names that collide with generated builder APIs are rejected, including `new`, `build`,
`with_value`, `builder`, `__koruma_builder`, `build_validator`, `capture_value_ref`, and the
generated `maybe_` prefix.

For optional non-required configuration fields, `Option<T>` setters take `T` directly and wrap it
in `Some(...)`. Use the generated `maybe_*` setter when you already have an `Option<T>`. Mark an
`Option<T>` setter as `required` when `None` is a meaningful explicit configuration value.

If a validator does not need to retain the failing input, you can opt out of capture on an
`Option<T>` value field:

```rs
#[validator]
pub struct RequiredValidation<T> {
    #[koruma(skip_capture)]
    actual: Option<T>,
}
```

The `skip_capture` marker keeps the stored field at its default `None` during derived
validation, which avoids clone requirements for validators whose messages do not need the original input. If your validator
still derives traits like `Clone` or `Debug` through that field, use manual impls to avoid
reintroducing type bounds.

`#[validator]` also emits `ValidatorMetadata<T>` for tooling. The metadata
descriptor lists configurable setter fields, and runtime params expose bool,
numeric, string, and optional values directly. Generic or otherwise
unconstrained values are reported as opaque so metadata does not add trait
bounds to your validator.

### 2. Use `#[derive(Koruma)]` on a struct + individual validator getters

Validators in `#[koruma(...)]` use direct setter syntax generated by `#[validator]`:

```rs
#[koruma(NumberRangeValidation::<_>::min(0).max(100))]
```

Each setter call takes exactly one argument. Put generic arguments on the validator path
(`Validator::<_>::min(0)`), not on the setter method.

Put all validators and field modifiers for a field in one `#[koruma(...)]`
attribute, separated by commas.

Use `#[koruma(skip)]` to explicitly exclude a field from validation when a marker is useful for
readability. Fields without `#[koruma(...)]` are ignored by default.

For `Option<T>` fields and optional `each(...)` elements, validators normally run only for `Some`
values and receive the inner `T`. Use an explicit `Option<_>` validator type or `full(...)` when
the validator should receive the whole optional value instead. Use `unwrapped(...)` only when the
code should state the default target explicitly:

```rs
#[koruma(koruma_collection::general::RequiredValidation::<Option<_>>)]
// Equivalent explicit target selection:
// #[koruma(full(koruma_collection::general::RequiredValidation::<_>))]
pub display_name: Option<String>;

#[koruma(unwrapped(NumberRangeValidation::<_>::min(0).max(100)))]
pub optional_score: Option<i32>;
```

Give validators lower-snake labels when you want stable, descriptive accessors or when two
validators would otherwise generate the same accessor name:

```rs
#[koruma(
    age_min = NumberRangeValidation::<_>::min(18),
    age_max = NumberRangeValidation::<_>::max(120),
)]
pub age: i32;
```

Labeled validators use the label for the generated getter (`age_min()`) and for `all()` enum
variants (`AgeMin`). The same syntax works inside `each(...)`.

```rs
use koruma::{Koruma, KorumaAllDisplay, Validate};

#[derive(Koruma, KorumaAllDisplay)]
pub struct Item {
    #[koruma(NumberRangeValidation::<_>::min(0).max(100))]
    pub age: i32,

    #[koruma(StringLengthValidation::min(1).max(67))]
    pub name: String,

    // No #[koruma(...)] attribute -> not validated
    pub internal_id: u64,
}

let item = Item {
    age: 150,
    name: "".to_string(),
    internal_id: 1,
};

match item.validate() {
    Ok(()) => println!("Item is valid!"),
    Err(errors) => {
        if let Some(age_err) = errors.age().number_range_validation() {
            println!("age failed: {}", age_err);
        }

        if let Some(name_err) = errors.name().string_length_validation() {
            println!("name failed: {}", name_err);
        }
    },
}
```

For per-element validation, `each(...)` supports `Vec<T>`, borrowed slices like
`&[T]`, arrays like `[T; N]`, and optional variants of those. This recognition is syntactic:
type aliases and custom collection types are not expanded or resolved by the macro. A recognized
`Vec<T>` path must resolve to `std::vec::Vec<T>`.

```rs
#[derive(Koruma)]
pub struct Order {
    #[koruma(each(NumberRangeValidation::<_>::min(1).max(5)))]
    pub quantities: Vec<i32>,
}

#[derive(Koruma)]
pub struct BorrowedOrder<'a> {
    #[koruma(each(NumberRangeValidation::<_>::min(1).max(5)))]
    pub quantities: &'a [i32],
}
```

### 3. Iterate failed validators (`KorumaAllDisplay`)

```rs
if let Err(errors) = item.validate() {
    for failed in errors.age().all() {
        println!("age validator: {}", failed);
    }

    for failed in errors.name().all() {
        println!("name validator: {}", failed);
    }
}
```

`all()` borrows the stored failed validators, so inspection does not require
validator types to implement `Clone`.

### Generated Validation Surface For UI Code

Derived error structs expose stable borrowed accessors intended for UI
generators:

- `errors.field()` returns the generated field error container.
- `errors.field().validator_name()` returns `Option<&Validator>` for a direct
  field validator.
- `errors.field().element_errors()` returns indexed element errors for
  `each(...)` validation.
- nested fields use the nested type's generated error container, and newtype
  validation exposes the inner field error through the generated newtype error.
- `errors.field().all()` returns borrowed `*ValidatorRef<'_>` enum values for
  that field, while `errors.all()` returns borrowed top-level validator refs
  when `KorumaAllDisplay` or `KorumaAllFluent` is derived.
- aggregate error structs implement `ValidationIssues`, which returns
  structured field and element issues with typed `ValidationFieldName` values,
  validator type names, labels, element indices, and messages for tooling that
  does not need the strongly typed accessor surface.

For display-based UI, derive `KorumaAllDisplay` and make every stored validator
implement `Display`. For localized UI, derive `KorumaAllFluent` and make every
stored validator implement `es_fluent::FluentMessage`, usually with
`#[derive(es_fluent::EsFluent)]`. Both paths preserve borrowed validator refs,
so generated UIs can render errors without cloning validator state.

### 4. Iterate failed validators with localized messages (`KorumaAllFluent`)

```toml
[dependencies]
koruma = { version = "*", features = ["derive", "fluent"] }
es-fluent = "0.16"
```

This setup assumes:

- `koruma` is built with `derive` + `fluent`.
- your application owns an `es-fluent` localizer, such as `EmbeddedI18n`.
- a locale is selected on that localizer before rendering messages.

Rendering is explicit: `KorumaAllFluent` produces `FluentMessage` values, and
your application chooses the localizer used to turn them into strings. The
examples expose a small `i18n::localize(...)` helper around an app-owned
`EmbeddedI18n`; an application can instead pass that localizer through its own
state.

```rs
use es_fluent::EsFluent;
use koruma::{Koruma, KorumaAllFluent, Validate, validator};

#[validator]
#[derive(Clone, Debug, EsFluent)]
pub struct IsEvenNumberValidation<
    T: Clone + Copy + std::fmt::Display + std::ops::Rem<Output = T> + From<u8> + PartialEq,
> {
    #[fluent(value = |x: &T| x.to_string())]
    actual: T,
}

impl<T: Copy + std::fmt::Display + std::ops::Rem<Output = T> + From<u8> + PartialEq> Validate<T>
    for IsEvenNumberValidation<T>
{
    fn validate(&self, value: &T) -> bool {
        *value % T::from(2u8) == T::from(0u8)
    }
}

#[validator]
#[derive(Clone, Debug, EsFluent)]
pub struct NonEmptyStringValidation {
    input: String,
}

impl Validate<String> for NonEmptyStringValidation {
    fn validate(&self, value: &String) -> bool {
        !value.is_empty()
    }
}

#[derive(Koruma, KorumaAllFluent)]
pub struct User {
    #[koruma(IsEvenNumberValidation::<_>)]
    pub id: i32,

    #[koruma(NonEmptyStringValidation)]
    pub username: String,
}

let user = User {
    id: 3,
    username: "".to_string(),
};

if let Err(errors) = user.validate() {
    if let Some(id_err) = errors.id().is_even_number_validation() {
        println!("{}", i18n::localize(id_err));
    }

    if let Some(username_err) = errors.username().non_empty_string_validation() {
        println!("{}", i18n::localize(username_err));
    }

    for failed in errors.id().all() {
        println!("{}", i18n::localize(failed));
    }

    for failed in errors.username().all() {
        println!("{}", i18n::localize(failed));
    }
}
```

## Newtype pattern (`#[koruma(newtype)]`, optional `try_new` / `try_from`)

Use `#[koruma(newtype)]`, adding `try_new` and `try_from` as needed, when you want:

- `newtype` - transparent error access to the inner field's error (`Deref` for non-optional fields, `Option<&InnerError>` accessors for `Option<Newtype>` fields)
- `NewtypeValue` / `NewtypeTryFromInner` - public inner-value borrow, consume, validation, and checked reconstruction methods that work even when the wrapper field is private
- `try_new` - a checked constructor function (`fn try_new(value: Inner) -> Result<Self, Error>`)
- `try_from` - a `TryFrom<Inner>` impl for checked conversions from the inner type

You can layer `derive_more` traits on top for additional wrapper ergonomics (e.g., `Deref` to inner value).

For normalized string newtypes, keep normalization outside validation. Parse or
normalize user input before constructing the newtype, then let Koruma validate
that the stored value is already canonical:

- use `#[koruma(newtype, try_new, try_from)]` when the wrapper should expose
  checked constructors and `TryFrom<Inner>`;
- put parsing, trimming, case conversion, or Unicode normalization in an
  application-owned parser or constructor before calling `try_new` or
  `try_from`;
- use `koruma_collection::string::CanonicalFormValidation::<_>::predicate(...)`
  when a storage or API boundary must reject values that are not already in
  canonical form;
- keep `FromStr`, `TryFrom`, serde `try_from`/`into`, `Display`, and localized
  validation messages aligned so every ingress path accepts the same canonical
  representation.

```rust,ignore
use koruma::{Koruma, KorumaAllDisplay};
use koruma_collection::string::CanonicalFormValidation;

#[derive(Clone, Debug, Koruma, KorumaAllDisplay)]
#[koruma(newtype, try_from)]
pub struct ProviderId {
    #[koruma(CanonicalFormValidation::<_>::predicate(is_provider_id_canonical))]
    value: String,
}

fn normalize_provider_id(value: &str) -> String {
    value.trim().to_ascii_lowercase()
}

fn is_provider_id_canonical(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

let provider_id = ProviderId::try_from(normalize_provider_id(" Provider-1 "))?;
```

A field can combine `newtype` with ordinary field validators when the wrapper field itself also
needs rules. The transparent newtype error access is preserved, and the extra validators use the
same optional, `full(...)`, and `unwrapped(...)` target selection rules as any other field:

```rs
#[koruma(newtype, koruma_collection::general::RequiredValidation::<Option<_>>)]
pub email: Option<Email>;
```

```rs
use es_fluent::EsFluent;
use koruma::{Koruma, KorumaAllFluent, NewtypeTryFromInner, NewtypeValue, Validate};

#[derive(Clone, Koruma, KorumaAllFluent)]
#[koruma(try_new, newtype)]
pub struct Email {
    #[koruma(NonEmptyStringValidation)]
    pub value: String,
}

#[derive(Koruma, KorumaAllFluent)]
pub struct SignupForm {
    #[koruma(NonEmptyStringValidation)]
    pub username: String,

    #[koruma(newtype)]
    pub email: Email,
}

#[derive(Koruma, KorumaAllFluent)]
pub struct OptionalSignupForm {
    #[koruma(newtype)]
    pub email: Option<Email>,
}

let form = SignupForm {
    username: "".to_string(),
    email: Email {
        value: "".to_string(),
    },
};
if let Err(errors) = form.validate() {
    if let Some(username_err) = errors.username().non_empty_string_validation() {
        println!("username failed: {}", i18n::localize(username_err));
    }
    if let Some(email_err) = errors.email().non_empty_string_validation() {
        println!("email failed: {}", i18n::localize(email_err));
    }

    for failed in errors.email().all() {
        println!("email validator: {}", i18n::localize(failed));
    }
}

let optional_form = OptionalSignupForm { email: None };
assert!(optional_form.validate().is_ok());

let invalid_optional_form = OptionalSignupForm {
    email: Some(Email {
        value: "".to_string(),
    }),
};
if let Err(errors) = invalid_optional_form.validate()
    && let Some(email_errors) = errors.email()
    && let Some(email_err) = email_errors.non_empty_string_validation()
{
    println!("optional email failed: {}", i18n::localize(email_err));
}

// Constructor-time validation path
if let Err(errors) = Email::try_new("".to_string()) {
    if let Some(email_err) = errors.non_empty_string_validation() {
        println!("email::try_new failed: {}", i18n::localize(email_err));
    }
    for failed in errors.all() {
        println!("email::try_new validator: {}", i18n::localize(failed));
    }
}

// Generic UI and adapter code can use the trait surface without relying on
// public wrapper fields.
let email = Email::try_from_inner("hello@example.com".to_string()).unwrap();
assert_eq!(email.as_inner(), "hello@example.com");
let raw_email = email.into_inner();
assert!(Email::validate_inner(&raw_email).is_ok());
```

### Unnamed newtype (tuple struct)

The same pattern works with tuple structs:

```rs
use es_fluent::EsFluent;
use koruma::{Koruma, KorumaAllFluent, Validate};

#[derive(Clone, Koruma, KorumaAllFluent)]
#[koruma(try_new, newtype)]
pub struct Username(#[koruma(NonEmptyStringValidation)] pub String);

#[derive(Koruma, KorumaAllFluent)]
pub struct LoginForm {
    #[koruma(newtype)]
    pub username: Username,
}

let login = LoginForm {
    username: Username("".to_string()),
};
if let Err(errors) = login.validate() {
    if let Some(username_err) = errors.username().non_empty_string_validation() {
        println!("username failed: {}", i18n::localize(username_err));
    }
}

if let Ok(username) = Username::try_new("alice".to_string()) {
    println!("username created: {}", username.0);
}
```

### TryFrom integration (`#[koruma(newtype, try_from)]`)

Every struct-level `#[koruma(newtype)]` wrapper implements
`NewtypeTryFromInner::try_from_inner`. Add flat `try_from` alongside `newtype`
only when you also want the standard-library `TryFrom<Inner>` impl:

```rs
use std::convert::TryFrom;
use es_fluent::EsFluent;
use koruma::{Koruma, KorumaAllFluent, Validate};

#[derive(Clone, Koruma, koruma::KorumaAllFluent)]
#[koruma(newtype, try_from)]
pub struct Only67u8(#[koruma(Only67Validation::<_>)] u8);

match Only67u8::try_from(69) {
    Ok(n) => println!("{}!", n.0),
    Err(errors) => {
        for failed in errors.all() {
            println!("validation failed: {}", i18n::localize(failed));
        }
    }
}
```

For exactly-one-field structs that should keep the regular error surface, use
`#[koruma(try_from)]` without `newtype`.
