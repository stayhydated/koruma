# koruma

[![Build Status](https://github.com/stayhydated/koruma/actions/workflows/ci.yml/badge.svg)](https://github.com/stayhydated/koruma/actions/workflows/ci.yml)
[![codecov](https://codecov.io/github/stayhydated/koruma/graph/badge.svg?token=34CV04UOU1)](https://codecov.io/github/stayhydated/koruma)
[![Docs](https://docs.rs/koruma/badge.svg)](https://docs.rs/koruma/)
[![Crates.io](https://img.shields.io/crates/v/koruma.svg)](https://crates.io/crates/koruma)

`koruma` is a per-field validation framework focused on:

1. **Type Safety**: Strongly typed validation error structs generated at compile time.
1. **Ergonomics**: Derive macros and validator attributes that minimize boilerplate.
1. **Developer Experience**: Optional constructors, nested/newtype validation, and fluent/i18n.

## Installation

```toml
[dependencies]
koruma = { version = "*" }
```

## koruma-collection ([online demos](https://stayhydated.github.io/koruma/))

[![Docs](https://docs.rs/koruma-collection/badge.svg)](https://docs.rs/koruma-collection/)
[![Crates.io](https://img.shields.io/crates/v/koruma-collection.svg)](https://crates.io/crates/koruma-collection)
[![Crowdin](https://badges.crowdin.net/koruma-collection/localized.svg)](https://crowdin.com/project/koruma-collection)

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
    pub actual: T,
}

impl<T: PartialOrd + Copy + fmt::Display> Validate<T> for NumberRangeValidation<T> {
    fn validate(&self, value: &T) -> bool {
        *value >= self.min && *value <= self.max
    }
}

impl<T: PartialOrd + Copy + fmt::Display + Clone> fmt::Display for NumberRangeValidation<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} is not in [{}, {}]", self.actual, self.min, self.max)
    }
}

#[validator]
#[derive(Clone, Debug)]
pub struct NonEmptyStringValidation {
    #[koruma(value)]
    pub input: String,
}

impl Validate<String> for NonEmptyStringValidation {
    fn validate(&self, value: &String) -> bool {
        !value.is_empty()
    }
}

impl fmt::Display for NonEmptyStringValidation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "string must not be empty")
    }
}
```

### 2. Use `#[derive(Koruma)]` on a struct + individual validator getters

```rs
use koruma::{Koruma, KorumaAllDisplay, Validate};

#[derive(Koruma, KorumaAllDisplay)]
pub struct SignupInput {
    #[koruma(NumberRangeValidation::<_>(min = 18, max = 120))]
    pub age: i32,

    #[koruma(NonEmptyStringValidation)]
    pub username: String,

    // No #[koruma(...)] attribute -> not validated
    pub internal_id: u64,
}

let input = SignupInput {
    age: 10,
    username: "".to_string(),
    internal_id: 42,
};

let err = input.validate().unwrap_err();

if let Some(age_err) = err.age().number_range_validation() {
    println!("age failed: {}", age_err);
}

if let Some(name_err) = err.username().non_empty_string_validation() {
    println!("username failed: {}", name_err);
}
```

### 3. Use `all()` getter (`KorumaAllDisplay`)

```rs
// Continue from `err` above:
for failed in err.age().all() {
    println!("age validator: {}", failed);
}

for failed in err.username().all() {
    println!("username validator: {}", failed);
}
```

### 4. Use `all()` getter with Fluent/i18n (`KorumaAllFluent`)

```toml
[dependencies]
koruma = { version = "*", features = ["derive", "fluent"] }
es-fluent = { version = "*", features = ["derive"] }
```

Assumes your i18n manager is initialized and a locale is selected.

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
    pub actual: T,
}

impl<T: Copy + std::fmt::Display + std::ops::Rem<Output = T> + From<u8> + PartialEq> Validate<T>
    for IsEvenNumberValidation<T>
{
    fn validate(&self, value: &T) -> bool {
        *value % T::from(2u8) == T::from(0u8)
    }
}

#[derive(Koruma, KorumaAllFluent)]
pub struct FluentUser {
    #[koruma(IsEvenNumberValidation::<_>)]
    pub id: i32,
}

let user = FluentUser { id: 3 };
let err = user.validate().unwrap_err();

if let Some(id_err) = err.id().is_even_number_validation() {
    println!("{}", id_err.to_fluent_string());
}

for failed in err.id().all() {
    println!("{}", failed.to_fluent_string());
}
```

## Newtype pattern (`#[koruma(try_new, newtype)]`)

Use `#[koruma(try_new, newtype)]` when you want a checked constructor (`try_new`) and
transparent newtype error access. You can still layer `derive_more` traits on top for wrapper
ergonomics.

```rs
use koruma::{Koruma, KorumaAllDisplay, Validate};

#[derive(Clone, Koruma, KorumaAllDisplay)]
#[koruma(try_new, newtype)]
pub struct Email {
    #[koruma(NonEmptyStringValidation)]
    pub value: String,
}

#[derive(Koruma, KorumaAllDisplay)]
pub struct SignupForm {
    #[koruma(NonEmptyStringValidation)]
    pub username: String,

    #[koruma(newtype)]
    pub email: Email,
}

let form = SignupForm {
    username: "alice".to_string(),
    email: Email {
        value: "".to_string(),
    },
};
let err = form.validate().unwrap_err();

if let Some(email_err) = err.email().non_empty_string_validation() {
    println!("email failed: {}", email_err);
}

for failed in err.email().all() {
    println!("email validator: {}", failed);
}

// Constructor-time validation path
if let Err(err) = Email::try_new("".to_string()) {
    if let Some(email_err) = err.non_empty_string_validation() {
        println!("email::try_new failed: {}", email_err);
    }
    for failed in err.all() {
        println!("email::try_new validator: {}", failed);
    }
}
```
