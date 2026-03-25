# Newtype Pattern & TryFrom

For single-value domain types, `koruma` supports a newtype pattern that performs validation at
construction time. Add `#[koruma(try_new, newtype(try_from))]` to generate a checked constructor
and, when requested, a `TryFrom<Inner>` implementation.

```rust
use es_fluent::ToFluentString as _;
use koruma::{Koruma, KorumaAllFluent};

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

if let Err(errors) = Email::try_new("".to_string()) {
    if let Some(email_err) = errors.non_empty_string_validation() {
        println!("email::try_new failed: {}", email_err.to_fluent_string());
    }

    for failed in errors.all() {
        println!("email::try_new validator: {}", failed.to_fluent_string());
    }
}
```

Use this pattern when you want invalid values to be impossible to construct accidentally. For
container structs, `#[koruma(newtype)]` lets outer validation reuse the inner newtype's validator
set and typed error accessors.
