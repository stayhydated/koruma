# koruma

[![Build Status](https://github.com/stayhydated/koruma/actions/workflows/ci.yml/badge.svg)](https://github.com/stayhydated/koruma/actions/workflows/ci.yml)
[![Docs](https://docs.rs/koruma/badge.svg)](https://docs.rs/koruma/)
[![Crates.io](https://img.shields.io/crates/v/koruma.svg)](https://crates.io/crates/koruma)

A per-field validation library for Rust with struct-based errors.

## Features

- Per-field validation with strongly-typed error
- Multiple validators per field
- Generic validator support with type inference
- Optional field support (skips validation when `None`)
- Nested struct validation with `#[koruma(nested)]`
- Newtype wrapper support with `#[koruma(newtype)]`
- Validated constructors with `#[koruma(try_new)]`

## koruma-collection

[![Docs](https://docs.rs/koruma-collection/badge.svg)](https://docs.rs/koruma-collection/)
[![Crates.io](https://img.shields.io/crates/v/koruma-collection.svg)](https://crates.io/crates/koruma-collection)

provides a collection of common validators, with partial i18n support.

currently supported: `en`, `fr`

## Installation

```toml
[dependencies]
koruma = { version = "*", features = ["derive"] }
bon = { version = "*" } # internally used by koruma
```

## Examples

- [koruma-collection](../../examples/collection)
- [user-defined](../../examples/user-defined)

## Quick Start

### Defining Validators

Use `#[koruma::validator]` to define validation rules. Each validator must have a field marked with `#[koruma(value)]` to capture the validated value:

### Generic Validators

For validators that work with multiple types, use generics with a blanket impl:

```rs
#[koruma::validator]
#[derive(Clone, Debug)]
pub struct RangeValidation::<T> {
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

### Type-specific Validators

```rs
use koruma::{Validate as _, validator};

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

// Use `::<_>` (turbofish) to infer the type from the field
#[derive(Koruma)]
pub struct Measurements {
    #[koruma(RangeValidation::<_>(min = 0.0, max = 100.0))]
    pub temperature: f64,

    #[koruma(RangeValidation::<_>(min = 0, max = 1000))]
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
    #[koruma(each(RangeValidation::<_>(min = 0.0, max = 100.0)))]
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

### Nested Struct Validation

For fields that are themselves structs deriving `Koruma`, use `#[koruma(nested)]` to automatically validate them:

```rs
#[derive(Koruma)]
pub struct Address {
    #[koruma(StringLengthValidation(min = 1, max = 100))]
    pub street: String,

    #[koruma(StringLengthValidation(min = 1, max = 50))]
    pub city: String,

    #[koruma(StringLengthValidation(min = 2, max = 10))]
    pub zip_code: String,
}

#[derive(Koruma)]
pub struct Customer {
    #[koruma(StringLengthValidation(min = 1, max = 100))]
    pub name: String,

    // Nested struct - will call Address::validate() automatically
    #[koruma(nested)]
    pub address: Address,
}

// Validation cascades through nested structs
let customer = Customer {
    name: "Alice".to_string(),
    address: Address {
        street: "".to_string(),  // Invalid: empty
        city: "Springfield".to_string(),
        zip_code: "12345".to_string(),
    },
};

match customer.validate() {
    Ok(()) => println!("Valid!"),
    Err(errors) => {
        // Access nested errors through the field getter
        if let Some(address_err) = errors.address() {
            if let Some(street_err) = address_err.street().string_length_validation() {
                println!("Street is invalid: {:?}", street_err.input);
            }
        }
    }
}
```

Nested validation also works with optional fields:

```rs
#[derive(Koruma)]
pub struct CustomerWithOptionalAddress {
    #[koruma(StringLengthValidation(min = 1, max = 100))]
    pub name: String,

    // Optional nested struct - skipped when None, validated when Some
    #[koruma(nested)]
    pub shipping_address: Option<Address>,
}

// None is skipped
let customer = CustomerWithOptionalAddress {
    name: "Bob".to_string(),
    shipping_address: None,  // Not validated
};
assert!(customer.validate().is_ok());
```

Nesting can be arbitrarily deep - nested structs can themselves contain nested structs:

```rs
#[derive(Koruma)]
pub struct Company {
    #[koruma(StringLengthValidation(min = 1, max = 200))]
    pub company_name: String,

    #[koruma(nested)]
    pub headquarters: Address,
}

#[derive(Koruma)]
pub struct Employee {
    #[koruma(StringLengthValidation(min = 1, max = 100))]
    pub employee_name: String,

    #[koruma(nested)]
    pub employer: Company,  // Company contains nested Address
}

// Access deeply nested errors
let err = employee.validate().unwrap_err();
if let Some(company_err) = err.employer() {
    if let Some(address_err) = company_err.headquarters() {
        if let Some(city_err) = address_err.city().string_length_validation() {
            println!("Company HQ city is invalid");
        }
    }
}
```

### Newtype Wrappers

For single-field wrapper structs (newtypes), use `#[koruma(newtype)]` at both the struct level and field level to get transparent error access.

#### Defining a Newtype

Use `#[koruma(newtype)]` at the struct level to mark a single-field struct as a newtype:

```rs
#[derive(Koruma)]
#[koruma(newtype)]
pub struct PositiveNumber {
    #[koruma(RangeValidation::<_>(min = 0, max = 1000))]
    pub value: i32,
}

// The error struct implements Deref, so you can access .all() directly
let num = PositiveNumber { value: -5 };
let err = num.validate().unwrap_err();

// Access validators directly via Deref
let all_errors = err.all();  // No need to go through .value()
if let Some(range_err) = err.range_validation() {
    println!("Value {} is out of range", range_err.actual);
}
```

#### Using Newtypes as Fields

When using a newtype as a field in another struct, use `#[koruma(newtype)]` instead of `#[koruma(nested)]` to get transparent error access:

```rs
#[derive(Koruma)]
pub struct Order {
    #[koruma(StringLengthValidation(min = 1, max = 100))]
    pub description: String,

    // Use newtype instead of nested for single-field wrappers
    #[koruma(newtype)]
    pub quantity: PositiveNumber,
}

let order = Order {
    description: "Widget".to_string(),
    quantity: PositiveNumber { value: -10 },
};
let err = order.validate().unwrap_err();

// Access nested newtype errors directly via Deref
// No need for .unwrap() or pattern matching on Option
let all_qty_errors = err.quantity().all();
if let Some(range_err) = err.quantity().range_validation() {
    println!("Quantity {} is invalid", range_err.actual);
}
```

The difference between `nested` and `newtype`:

| Attribute | Use Case | Error Access |
|-----------|----------|--------------|
| `#[koruma(nested)]` | Multi-field structs | `err.field()` returns `Option<&InnerError>` |
| `#[koruma(newtype)]` | Single-field wrappers | `err.field()` returns `&Wrapper` with `Deref` |

### Validated Constructors with `try_new`

Use `#[koruma(try_new)]` at the struct level to generate a `try_new` constructor that validates on creation:

```rs
#[derive(Koruma)]
#[koruma(try_new)]
pub struct ValidatedUser {
    #[koruma(StringLengthValidation(min = 1, max = 50))]
    pub username: String,

    #[koruma(RangeValidation::<_>(min = 18, max = 150))]
    pub age: i32,
}

// Use try_new instead of struct literal + validate
match ValidatedUser::try_new("alice".to_string(), 25) {
    Ok(user) => println!("Created user: {}", user.username),
    Err(errors) => {
        if let Some(name_err) = errors.username().string_length_validation() {
            println!("Invalid username");
        }
    }
}

// Equivalent to:
// let user = ValidatedUser { username: "alice".to_string(), age: 25 };
// user.validate()?;
```

You can combine `try_new` with `newtype` for validated wrapper types:

```rs
#[derive(Koruma)]
#[koruma(try_new, newtype)]
pub struct Email {
    #[koruma(EmailValidation)]
    pub value: String,
}

// Create validated email
let email = Email::try_new("user@example.com".to_string())?;

// Invalid emails are rejected at construction
let result = Email::try_new("not-an-email".to_string());
assert!(result.is_err());
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
use es_fluent::ToFluentString as _;

if let Some(err) = errors.age().number_range_validation() {
    println!("{}", err.to_fluent_string());
}
```

### Fluent with `all()` Method

When using the `all()` method to get all failed validators, you can derive `KorumaFluentEnum` on the generated enum to implement `ToFluentString`:

```rs
use es_fluent::ToFluentString as _;
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
