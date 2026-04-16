# Newtype Pattern & TryFrom

Use `#[koruma(try_new, newtype(try_from))]` when you need:

- `try_new` - a checked constructor function (`fn try_new(value: Inner) -> Result<Self, Error>`)
- `newtype(try_from)` - a `TryFrom<Inner>` impl for checked conversions from the inner type
- `newtype` - transparent error access to the inner field's error (`Deref` for non-optional fields, `Option<&InnerError>` accessors for `Option<Newtype>` fields)

You can layer `derive_more` traits on top for additional wrapper ergonomics (for example, `Deref`
to inner value).

```rust
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

#[derive(Koruma, KorumaAllFluent)]
pub struct OptionalSignupForm {
    #[koruma(newtype)]
    pub email: Option<Email>,
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

let optional_form = OptionalSignupForm { email: None };
assert!(optional_form.validate().is_ok());

let invalid_optional_form = OptionalSignupForm {
    email: Some(Email {
        value: "".to_string(),
    }),
};
if let Err(errors) = invalid_optional_form.validate()
    && let Some(email_errors) = errors.email()
    && let Some(email_err) = email_errors.non_empty_string_validation()
{
    println!("optional email failed: {}", email_err.to_fluent_string());
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

## Unnamed newtype (tuple struct)

The same pattern works with tuple structs:

```rust
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

## TryFrom integration (`#[koruma(newtype(try_from))]`)

Add `try_from` inside `newtype(...)` to generate a `TryFrom<Inner>` impl:

```rust
use std::convert::TryFrom;
use es_fluent::ToFluentString as _;
use koruma::{Koruma, KorumaAllFluent, Validate};

#[derive(Clone, Koruma, KorumaAllFluent)]
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
