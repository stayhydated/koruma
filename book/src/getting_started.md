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

A minimal example:

```rust
use koruma::{Koruma, KorumaAllDisplay, Validate, validator};
use std::fmt;

#[validator]
#[derive(Clone, Debug)]
pub struct NonEmptyStringValidation {
    #[koruma(value)]
    input: String,
}

impl Validate<String> for NonEmptyStringValidation {
    fn validate(&self, value: &String) -> bool {
        !value.is_empty()
    }
}

impl fmt::Display for NonEmptyStringValidation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "value must not be empty")
    }
}

#[derive(Koruma, KorumaAllDisplay)]
pub struct User {
    #[koruma(NonEmptyStringValidation)]
    pub username: String,
}

let user = User {
    username: "".to_string(),
};

if let Err(errors) = user.validate() {
    if let Some(err) = errors.username().non_empty_string_validation() {
        println!("{}", err);
    }
}
```

If you want to inspect the captured input on a validator error, call the generated getter that
matches the `#[koruma(value)]` field name.

The following chapters expand this pattern and show how to build richer validators and more useful
error reporting.
