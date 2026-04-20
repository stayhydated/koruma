# Getting Started

Add `koruma` to your dependencies:

```toml
[dependencies]
koruma = { version = "*" }
```

## Feature flags

- `derive` (default): enables derive/attribute macros (`Koruma`, `KorumaAllDisplay`, `#[validator]`).
- `fluent`: enables localized error support for `KorumaAllFluent` (use with `es-fluent`).
- `internal-showcase`: enables internal validator showcase registry hooks used by workspace demos.

A typical `koruma` workflow looks like this:

1. Define one or more validator types.
2. Implement `Validate<T>` for each validator.
3. Attach validators to fields with `#[koruma(...)]`.
4. Derive `Koruma` on the struct you want to validate.
5. Call `validate()` and inspect the generated error accessors.

A small end-to-end example, using the validator definitions from the next chapter:

```rust
use koruma::{Koruma, KorumaAllDisplay};

// Assume NumberRangeValidation and StringLengthValidation are defined as in the next chapter.
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

if let Err(errors) = item.validate() {
    if let Some(age_err) = errors.age().number_range_validation() {
        println!("age failed: {}", age_err);
    }

    if let Some(name_err) = errors.name().string_length_validation() {
        println!("name failed: {}", name_err);
    }
}
```

The validator definitions themselves come next. If you want to inspect the captured input on a
validator error, call the generated getter that matches the `#[koruma(value)]` field name.

For validators that only care about presence and do not need to store the input, an `Option<T>`
value field can use `#[koruma(value, skip_capture)]` to avoid derive-generated capture clones. If
that field would still impose `Clone` or `Debug` bounds on the validator type, use manual impls
like `RequiredValidation` does.

The following chapters expand this pattern and show how to build richer validators and more useful
error reporting.
