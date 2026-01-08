# koruma

[![Build Status](https://github.com/stayhydated/koruma/actions/workflows/ci.yml/badge.svg)](https://github.com/stayhydated/koruma/actions/workflows/ci.yml)
[![Docs](https://docs.rs/koruma/badge.svg)](https://docs.rs/koruma/)
[![Crates.io](https://img.shields.io/crates/v/koruma.svg)](https://crates.io/crates/koruma)

A per-field validation library for Rust with struct-based errors.

## Features

- Per-field validation with strongly-typed error structs
- Multiple validators per field
- Generic validator support with type inference
- Optional field support (skips validation when `None`)

## koruma-collection

[![Docs](https://docs.rs/koruma-collection/badge.svg)](https://docs.rs/koruma-collection/)
[![Crates.io](https://img.shields.io/crates/v/koruma-collection.svg)](https://crates.io/crates/koruma-collection)

provides a collection of common validators, with partial i18n support (`en`, `fr`).

## Installation

```toml
[dependencies]
koruma = { version = "*", features = ["derive"] }
```

## Examples

- [koruma-collection](examples/collection)
- [custom](examples/custom)

## Quick Start

### Defining Validators

Use `#[koruma::validator]` to define validation rules. Each validator must have a field marked with `#[koruma(value)]` to capture the validated value:

```rs
use koruma::{Validate, validator};

#[koruma::validator]
#[derive(Clone, Debug)]
pub struct NumberRangeValidation {
    min: i32,
    max: i32,
    #[koruma(value)]
    pub actual: i32,  // The type matches what you're validating
}

impl Validate<i32> for NumberRangeValidation {
    fn validate(&self, value: &i32) -> bool {
        *value >= self.min && *value <= self.max
    }
}
```

### Generic Validators

For validators that work with multiple types, use generics with a blanket impl:

```rs
#[koruma::validator]
#[derive(Clone, Debug)]
pub struct RangeValidation<T> {
    pub min: T,
    pub max: T,
    #[koruma(value)]
    pub actual: T,
}

// Use a blanket impl with trait bounds
impl<T: PartialOrd + Clone> Validate<T> for RangeValidation<T> {
    fn validate(&self, value: &T) -> bool {
        *value >= self.min && *value <= self.max
    }
}
```

### Validating Structs

Apply validators to struct fields using `#[derive(Koruma)]` and the `#[koruma(...)]` attribute:

```rs
use koruma::Koruma;

#[derive(Koruma)]
pub struct User {
    #[koruma(NumberRangeValidation(min = 0, max = 150))]
    pub age: i32,

    #[koruma(StringLengthValidation(min = 1, max = 100))]
    pub name: String,

    // Fields without #[koruma(...)] are not validated
    pub internal_id: u64,
}

// Use <_> to infer the type from the field
#[derive(Koruma)]
pub struct Measurements {
    #[koruma(RangeValidation<_>(min = 0.0, max = 100.0))]
    pub temperature: f64,

    #[koruma(RangeValidation<_>(min = 0, max = 1000))]
    pub pressure: i32,
}
```

### Accessing Validation Errors

The generated error struct provides typed access to each field's validation errors:

```rs
let user = User {
    age: 200,  // Invalid
    name: "".to_string(),  // Invalid
    internal_id: 1,
};

match user.validate() {
    Ok(()) => println!("Valid!"),
    Err(errors) => {
        // Access errors by field, then by validator
        if let Some(age_err) = errors.age().number_range_validation() {
            println!("Age {} is out of range", age_err.actual);
        }
        if let Some(name_err) = errors.name().string_length_validation() {
            println!("Name is invalid: {:?}", name_err.input);
        }
    }
}
```

### Multiple Validators Per Field

Apply multiple validators to a single field by separating them with commas:

```rs
#[derive(Koruma)]
pub struct Item {
    // Must be in range 0-100 AND be even
    #[koruma(NumberRangeValidation(min = 0, max = 100), EvenNumberValidation)]
    pub value: i32,
}

// Access individual validators
let err = item.validate().unwrap_err();
if let Some(range_err) = err.value().number_range_validation() {
    // Handle range error
}
if let Some(even_err) = err.value().even_number_validation() {
    // Handle even number error
}

// Or get all failed validators at once
let all_errors = err.value().all();  // Vec<ItemValueValidator>
```

### Collection Validation

Use the `each(...)` syntax to validate each element in a `Vec`:

```rs
#[derive(Koruma)]
pub struct Order {
    // Each score must be in range 0-100
    #[koruma(each(RangeValidation<_>(min = 0.0, max = 100.0)))]
    pub scores: Vec<f64>,
}

// Errors include the index of the failing element
let order = Order {
    scores: vec![50.0, 150.0, 75.0],  // 150 is out of range
};
let err = order.validate().unwrap_err();

// Returns &[(usize, OrderScoresError)]
for (index, element_error) in err.scores() {
    if let Some(range_err) = element_error.generic_range_validation() {
        println!("Score at index {} is invalid: {}", index, range_err.actual);
    }
}
```

### Optional Field Validation

Fields of type `Option<T>` are automatically handled:

- **`None`**: Validation is skipped entirely
- **`Some(value)`**: The inner value is validated

```rs
#[derive(Koruma)]
pub struct UserProfile {
    #[koruma(StringLengthValidation(min = 1, max = 50))]
    pub username: String,  // Required field

    #[koruma(StringLengthValidation(min = 1, max = 200))]
    pub bio: Option<String>,  // Optional - skipped when None

    #[koruma(NumberRangeValidation(min = 0, max = 150))]
    pub age: Option<i32>,  // Optional - skipped when None
}

// None fields are skipped
let profile = UserProfile {
    username: "alice".to_string(),
    bio: None,  // Not validated
    age: None,  // Not validated
};
assert!(profile.validate().is_ok());

// Some fields are validated
let profile = UserProfile {
    username: "bob".to_string(),
    bio: Some("".to_string()),  // Invalid: too short
    age: Some(200),  // Invalid: out of range
};
let err = profile.validate().unwrap_err();

// Error captures the inner value
let bio_err = err.bio().string_length_validation().unwrap();
assert_eq!(bio_err.input, "".to_string());
```

## Error Messages

### Basic String Messages

For simple error messages, implement `Display` or a custom method on your validators:

```rs
impl std::fmt::Display for NumberRangeValidation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Value {} must be between {} and {}",
            self.actual,
            self.min,
            self.max
        )
    }
}

// Usage
if let Some(err) = errors.age().number_range_validation() {
    println!("{}", err);  // "Value 200 must be between 0 and 150"
}
```

### Fluent Integration

For internationalized error messages, use [es-fluent](https://crates.io/crates/es-fluent):

```toml
[dependencies]
koruma = { version = "0.1" }
es-fluent = "0.4"
```

Derive `EsFluent` on your validators:

```rs
use es_fluent::EsFluent;

#[koruma::validator]
#[derive(Clone, Debug, EsFluent)]
pub struct NumberRangeValidation {
    min: i32,
    max: i32,
    #[koruma(value)]
    pub actual: i32,
}
```

Create corresponding Fluent files:

```ftl
# locales/en/main.ftl
number-range-validation = Value { $actual } must be between { $min } and { $max }
```

Use `to_fluent_string()` to get localized messages:

```rs
use es_fluent::ToFluentString;

if let Some(err) = errors.age().number_range_validation() {
    println!("{}", err.to_fluent_string());
}
```

### Fluent with `all()` Method

When using the `all()` method to get all failed validators, you can derive `KorumaFluentEnum` on the generated enum to implement `ToFluentString`:

```rs
use es_fluent::ToFluentString;
use koruma::KorumaFluentEnum;

// Derive KorumaFluentEnum on the generated validator enum
// This requires all inner validators to implement ToFluentString
#[derive(KorumaFluentEnum)]
pub enum ItemValueKorumaValidator {
    NumberRangeValidation(NumberRangeValidation),
    EvenNumberValidation(EvenNumberValidation),
}

// Now you can iterate over all errors
for validator in errors.value().all() {
    println!("{}", validator.to_fluent_string());
}
```

Note: `KorumaFluentEnum` requires the `fluent` feature to be enabled and all variant types must implement `ToFluentString`.
