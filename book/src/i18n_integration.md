# i18n Integration with [es-fluent](https://github.com/stayhydated/es-fluent)

`koruma` can integrate with [es-fluent](https://github.com/stayhydated/es-fluent) to localise validation errors. Enable the `fluent` feature
and add the matching [es-fluent](https://github.com/stayhydated/es-fluent) dependency:

```toml
[dependencies]
koruma = { version = "*", features = ["derive", "fluent"] }
es-fluent = "0.16"
```

This setup assumes:

- `koruma` is built with `derive` + `fluent`.
- your application owns an `es-fluent` localizer, such as `EmbeddedI18n`.
- a locale is selected on that localizer before rendering messages.

Rendering is explicit: `KorumaAllFluent` produces `FluentMessage` values, and
your application chooses the localizer used to turn them into strings. The
examples expose a small `i18n::localize(...)` helper around an app-owned
`EmbeddedI18n`; an application can instead pass that localizer through its own
state.

Validators intended for localisation derive `EsFluent`. When the validated value needs custom
conversion, annotate it with `#[fluent(value(|x| ...))]`. Then derive `KorumaAllFluent` on the
consumer type.

```rust
use es_fluent::EsFluent;
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
    #[koruma(IsEvenNumberValidation::<_>)]
    pub id: i32,

    #[koruma(NonEmptyStringValidation)]
    pub username: String,
}

let user = User { id: 3, username: "".to_string() };
if let Err(errors) = user.validate() {
    if let Some(id_err) = errors.id().is_even_number_validation() {
        println!("{}", i18n::localize(id_err));
    }

    if let Some(username_err) = errors.username().non_empty_string_validation() {
        println!("{}", i18n::localize(username_err));
    }

    for failed in errors.id().all() {
        println!("{}", i18n::localize(failed));
    }

    for failed in errors.username().all() {
        println!("{}", i18n::localize(failed));
    }
}
```

`KorumaAllFluent` gives you an `all()` iterator whose elements can be converted with
`FluentMessage` + `FluentLocalizer`. Use the app-owned localizer directly, or
wrap it in a small helper function, after selecting the locale you want to render.
