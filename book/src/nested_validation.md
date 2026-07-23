# Nested Validation

When your data model contains structs inside other structs, you can use the `#[koruma(nested)]` attribute to validate them hierarchically.

This attribute tells `koruma` to call `validate()` on the nested field and include its errors in the
parent's generated error type if any occur. For example, `Customer` produces
`CustomerKorumaValidationError`, with strongly typed access not just to its own fields, but also to
the nested struct's fields.

Types produced by `#[derive(Koruma)]` already satisfy the nested validation
contract. Handwritten `ValidateExt` implementations must use an error type that
implements both `ValidationError` and `Default`, because parent error structs
need an empty nested error value while they collect failures.

```rust
use koruma::Koruma;

#[derive(Clone, Koruma)]
pub struct Address {
    #[koruma(StringLengthValidation.min(1).max(100))]
    pub street: String,

    #[koruma(StringLengthValidation.min(1).max(50))]
    pub city: String,

    #[koruma(ZipCodeValidation)]
    pub zip_code: String,
}

#[derive(Koruma)]
pub struct Customer {
    #[koruma(StringLengthValidation.min(1).max(100))]
    pub name: String,

    #[koruma(NumberRangeValidation::<_>.min(18).max(120))]
    pub age: i32,

    // Nested struct - validation cascades automatically
    #[koruma(nested)]
    pub address: Address,
}

let customer = Customer {
    name: "".to_string(), // Invalid: empty name
    age: 15,              // Invalid: too young (min 18)
    address: Address {
        street: "123 Main St".to_string(),
        city: "".to_string(),        // Invalid: empty city
        zip_code: "ABC".to_string(), // Invalid: not 5 digits
    },
};

match customer.validate() {
    Ok(()) => println!("Customer is valid!"),
    Err(errors) => {
        // Access top-level field errors
        if let Some(name_err) = errors.name().string_length_validation() {
            println!("name: {}", name_err);
        }

        if let Some(age_err) = errors.age().number_range_validation() {
            println!("age: {}", age_err);
        }

        // Access nested struct errors
        if let Some(address_err) = errors.address() {
            if let Some(street_err) = address_err.street().string_length_validation() {
                println!("street: {}", street_err);
            }

            if let Some(city_err) = address_err.city().string_length_validation() {
                println!("city: {}", city_err);
            }

            if let Some(zip_err) = address_err.zip_code().zip_code_validation() {
                println!("zip_code: {}", zip_err);
            }
        }
    }
}
```

## How It Works

1. Both the parent (`Customer`) and nested (`Address`) structs must derive `Koruma`.
2. When `customer.validate()` is called, it verifies `name` and `age` normally and also calls `address.validate()`.
3. If `address.validate()` fails, the resulting `AddressKorumaValidationError` is stored inside the
   customer's `CustomerKorumaValidationError`.
4. You access the nested errors using the corresponding accessor (`errors.address()`), which
   returns `Option<&AddressKorumaValidationError>`. If the address has errors, this returns `Some`
   with the exact error tree of the nested type.

This nested pattern seamlessly integrates with all `koruma` features including [es-fluent](https://github.com/stayhydated/es-fluent) localisation and `newtype` validation.
