# i18n Integration with [es-fluent](https://github.com/stayhydated/es-fluent)

`koruma` can integrate with [es-fluent](https://github.com/stayhydated/es-fluent) to localise validation errors. Enable the `fluent` feature
and add the matching [es-fluent](https://github.com/stayhydated/es-fluent) dependency:

```toml
[dependencies]
koruma = { version = "*", features = ["derive", "fluent"] }
es-fluent = { version = "*", features = ["derive"] }
```

This setup assumes:

- `koruma` is built with `derive` + `fluent`.
- your `es-fluent` manager is initialized.
- a locale is selected before rendering messages.

Validators intended for localisation derive `EsFluent`. When the validated value needs custom
conversion, annotate it with `#[fluent(value(|x| ...))]`. Then derive `KorumaAllFluent` on the
consumer type.

```rust
use es_fluent::{EsFluent, ToFluentString as _};
use koruma::{Koruma, KorumaAllFluent, Validate, validator};

#[validator]
#[derive(Clone, Debug, EsFluent)]
pub struct IsEvenNumberValidation<
    T: Clone + Copy + std::fmt::Display + std::ops::Rem<Output = T> + From<u8> + PartialEq,
> {
    #[koruma(value)]
    #[fluent(value(|x: &T| x.to_string()))]
    actual: T,
}

impl<T: Copy + std::fmt::Display + std::ops::Rem<Output = T> + From<u8> + PartialEq> Validate<T>
    for IsEvenNumberValidation<T>
{
    fn validate(&self, value: &T) -> bool {
        *value % T::from(2u8) == T::from(0u8)
    }
}

#[validator]
#[derive(Clone, Debug, EsFluent)]
pub struct NonEmptyStringValidation {
    #[koruma(value)]
    input: String,
}

impl Validate<String> for NonEmptyStringValidation {
    fn validate(&self, value: &String) -> bool {
        !value.is_empty()
    }
}

#[derive(Koruma, KorumaAllFluent)]
pub struct User {
    #[koruma(IsEvenNumberValidation<_>)]
    pub id: i32,

    #[koruma(NonEmptyStringValidation)]
    pub username: String,
}

let user = User { id: 3, username: "".to_string() };
if let Err(errors) = user.validate() {
    if let Some(id_err) = errors.id().is_even_number_validation() {
        println!("{}", id_err.to_fluent_string());
    }

    if let Some(username_err) = errors.username().non_empty_string_validation() {
        println!("{}", username_err.to_fluent_string());
    }

    for failed in errors.id().all() {
        println!("{}", failed.to_fluent_string());
    }

    for failed in errors.username().all() {
        println!("{}", failed.to_fluent_string());
    }
}
```

`KorumaAllFluent` gives you an `all()` iterator whose elements can be converted with
`ToFluentString`. Initialise your i18n manager and select a locale before rendering localised
messages.
