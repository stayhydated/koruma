# Validate nested data

Use `#[koruma(nested)]` when a field has its own `ValidateExt` implementation and the parent
should retain that field's typed error tree.

## Validate a nested struct

Types that derive `Koruma` already implement `ValidateExt`:

```rust,ignore
use koruma::Koruma;
use koruma_collection::collection;

#[derive(Koruma)]
pub struct Address {
    #[koruma(collection::NonEmptyValidation::<_>)]
    pub city: String,
}

#[derive(Koruma)]
pub struct Customer {
    #[koruma(collection::NonEmptyValidation::<_>)]
    pub name: String,

    #[koruma(nested)]
    pub address: Address,
}

let customer = Customer {
    name: "Alice".to_string(),
    address: Address {
        city: String::new(),
    },
};

let errors = customer.validate().expect_err("address should fail");
if let Some(address_errors) = errors.address() {
    assert!(address_errors.city().non_empty_validation().is_some());
}
```

In an ordinary parent struct, a nested field accessor returns `Option<&NestedError>`. It is
`Some` only when that nested value produced errors, including when the source field itself is
required. This lets another parent field fail while a valid nested field has no stored error.

## Use optional nested fields

An `Option<Nested>` field skips `None` and validates the inner value for `Some`:

```rust,ignore
#[derive(Koruma)]
pub struct Customer {
    #[koruma(nested)]
    pub shipping_address: Option<Address>,
}
```

The generated `shipping_address()` accessor returns `Some(&AddressKorumaValidationError)` only
when a present address fails. Both a missing address and a valid address produce `None`.

## Implement nested validation by hand

A handwritten nested type implements `ValidateExt`. Its associated error must implement both
`ValidationError` and `Default`, because generated parent errors create an empty value while
collecting failures:

```rust,ignore
impl koruma::ValidateExt for CustomAddress {
    type Error = CustomAddressError;

    fn validate(&self) -> Result<(), Self::Error> {
        // Return the complete typed error value.
        # todo!()
    }
}
```

For transparent validation through a single-field wrapper, use
[validated newtypes](newtype_pattern.md) instead.
