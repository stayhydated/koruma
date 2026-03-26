# Nested Validation

When your data model contains structs inside other structs, you can use the `#[koruma(nested)]` attribute to validate them hierarchically. 

This attribute tells `koruma` to call `validate()` on the nested field and include its errors in the parent's error type if any occur. This allows the parent struct's `Errors` struct to provide strongly typed access not just to its own fields, but also to the nested struct's fields.

```rust
use koruma::{Koruma, KorumaAllDisplay};

#[derive(Clone, Koruma)]
pub struct Address {
    #[koruma(StringLengthValidation(min = 1, max = 100))]
    pub street: String,

    #[koruma(StringLengthValidation(min = 1, max = 50))]
    pub city: String,
    
    // Imagine ZipCodeValidation is defined somewhere
    #[koruma(ZipCodeValidation)]
    pub zip_code: String,
}

#[derive(Koruma)]
pub struct Customer {
    #[koruma(StringLengthValidation(min = 1, max = 100))]
    pub name: String,

    // Nested struct - validation cascades automatically
    #[koruma(nested)]
    pub address: Address,
}

let customer = Customer {
    name: "".to_string(), // Invalid: empty name
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
2. When `customer.validate()` is called, it verifies `name` normally and also calls `address.validate()`.
3. If `address.validate()` fails, the resulting errors are wrapped inside `customer`'s overall `Errors` struct.
4. You access the nested errors using the corresponding accessor (`errors.address()`), which returns an `Option<AddressErrors>`. If there are any errors in the `address`, this returns `Some`, containing the exact error tree of the nested type.

This nested pattern seamlessly integrates with all `koruma` features including `es-fluent` localisation and `newtype` validation.
