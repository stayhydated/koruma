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
pub struct NumberRangeValidation<T: PartialOrd + Copy + fmt::Display + Clone> {
    min: T,
    max: T,
    #[koruma(value)]
    actual: T,
}

impl<T: PartialOrd + Copy + fmt::Display> Validate<T> for NumberRangeValidation<T> {
    fn validate(&self, value: &T) -> bool {
        *value >= self.min && *value <= self.max
    }
}

impl<T: PartialOrd + Copy + fmt::Display + Clone> fmt::Display for NumberRangeValidation<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "yo number {} aint in [{}, {}]",
            self.actual, self.min, self.max
        )
    }
}

#[validator]
#[derive(Clone, Debug)]
pub struct StringLengthValidation {
    min: usize,
    max: usize,
    #[koruma(value)]
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

`#[validator]` generates `with_value(...)` on the builder and a getter on the validator type with
the same name as the `#[koruma(value)]` field. That field is expected to stay private; use the
generated getter for reads.

### 2. Use `#[derive(Koruma)]` on a struct + individual validator getters

```rs
use koruma::{Koruma, KorumaAllDisplay, Validate};

#[derive(Koruma, KorumaAllDisplay)]
pub struct Item {
    #[koruma(NumberRangeValidation<_>(min = 0, max = 100))]
    pub age: i32,

    #[koruma(StringLengthValidation(min = 1, max = 67))]
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
`&[T]`, arrays like `[T; N]`, and optional variants of those:

```rs
#[derive(Koruma)]
pub struct Order {
    #[koruma(each(NumberRangeValidation<_>(min = 1, max = 5)))]
    pub line_item_scores: Vec<i32>,
}
```

### 3. Use `all()` getter (`KorumaAllDisplay`)

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

### 4. Use `all()` getter with localized messages (`KorumaAllFluent`)

```toml
[dependencies]
koruma = { version = "*", features = ["derive", "fluent"] }
es-fluent = { version = "*", features = ["derive"] }
```

This setup assumes:

- `koruma` is built with `derive` + `fluent`.
- your `es-fluent` manager is initialized.
- a locale is selected before rendering messages.

```rs
use es_fluent::{EsFluent, ToFluentString as _};
use koruma::{Koruma, KorumaAllFluent, Validate, validator};

#[validator]
#[derive(Clone, Debug, EsFluent)]
pub struct IsEvenNumberValidation<
    T: Clone + Copy + std::fmt::Display + std::ops::Rem<Output = T> + From<u8> + PartialEq,
> {
    #[koruma(value)]
    #[fluent(value(|x: &T| x.to_string()))]
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
    #[koruma(value)]
    input: String,
}

impl Validate<String> for NonEmptyStringValidation {
    fn validate(&self, value: &String) -> bool {
        !value.is_empty()
    }
}

#[derive(Koruma, KorumaAllFluent)]
pub struct User {
    #[koruma(IsEvenNumberValidation<_>)]
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
        println!("{}", id_err.to_fluent_string());
    }

    if let Some(username_err) = errors.username().non_empty_string_validation() {
        println!("{}", username_err.to_fluent_string());
    }

    for failed in errors.id().all() {
        println!("{}", failed.to_fluent_string());
    }

    for failed in errors.username().all() {
        println!("{}", failed.to_fluent_string());
    }
}
```

## Newtype pattern (`#[koruma(try_new, newtype(try_from))]`)

Use `#[koruma(try_new, newtype(try_from))]` when you need:

- `try_new` - a checked constructor function (`fn try_new(value: Inner) -> Result<Self, Error>`)
- `newtype(try_from)` - a `TryFrom<Inner>` impl for `From`/`try_from` calls
- `newtype` - transparent error access via `Deref` to the inner field's error

You can layer `derive_more` traits on top for additional wrapper ergonomics (e.g., `Deref` to inner value).

```rs
use es_fluent::ToFluentString as _;
use koruma::{Koruma, KorumaAllFluent, Validate};

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

let form = SignupForm {
    username: "".to_string(),
    email: Email {
        value: "".to_string(),
    },
};
if let Err(errors) = form.validate() {
    if let Some(username_err) = errors.username().non_empty_string_validation() {
        println!("username failed: {}", username_err.to_fluent_string());
    }
    if let Some(email_err) = errors.email().non_empty_string_validation() {
        println!("email failed: {}", email_err.to_fluent_string());
    }

    for failed in errors.email().all() {
        println!("email validator: {}", failed.to_fluent_string());
    }
}

// Constructor-time validation path
if let Err(errors) = Email::try_new("".to_string()) {
    if let Some(email_err) = errors.non_empty_string_validation() {
        println!("email::try_new failed: {}", email_err.to_fluent_string());
    }
    for failed in errors.all() {
        println!("email::try_new validator: {}", failed.to_fluent_string());
    }
}
```

### Unnamed newtype (tuple struct)

The same pattern works with tuple structs:

```rs
use es_fluent::ToFluentString as _;
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
        println!("username failed: {}", username_err.to_fluent_string());
    }
}

if let Ok(username) = Username::try_new("alice".to_string()) {
    println!("username created: {}", username.0);
}
```

### TryFrom integration (`#[koruma(newtype(try_from))]`)

Add `try_from` inside `newtype(...)` to generate a `TryFrom<Inner>` impl:

```rs
use std::convert::TryFrom;
use es_fluent::ToFluentString as _;
use koruma::{Koruma, KorumaAllFluent, Validate};

#[derive(Clone, Koruma, koruma::KorumaAllFluent)]
#[koruma(newtype(try_from))]
pub struct Only67u8(#[koruma(Only67Validation<_>)] u8);

match Only67u8::try_from(69) {
    Ok(n) => println!("{}!", n.0),
    Err(errors) => {
        for failed in errors.all() {
            println!("validation failed: {}", failed.to_fluent_string());
        }
    }
}
```
