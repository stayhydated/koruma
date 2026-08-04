# Localize errors with es-fluent

Derive `KorumaAllFluent` to make generated validation failures implement `FluentMessage`, then
render them with the es-fluent localizer owned by your application. Koruma does not select or store
the active locale.

This guide assumes the application already has an
[es-fluent](https://github.com/stayhydated/es-fluent) localizer such as `EmbeddedI18n`.

## Enable Fluent support

```toml
[dependencies]
koruma = { version = "0.10", features = ["derive", "fluent"] }
es-fluent = "0.18"
```

For localized built-in validators, also add `koruma-collection` with `fluent`, or use
`full-fluent` when the optional validators are needed.

## Define a localized validator

`EsFluent` derives a snake-case message ID from the validator type. This validator uses the
message ID `even_validation`:

```fluent
even_validation = The value { $actual } must be even.
```

Add the message to each locale in the application's es-fluent asset layout, then derive
`EsFluent` on the validator and `KorumaAllFluent` on the validated type:

```rust,ignore
use es_fluent::EsFluent;
use koruma::{Koruma, KorumaAllFluent, Validate, validator};

#[validator]
#[derive(Clone, Debug, EsFluent)]
pub struct EvenValidation {
    #[fluent(value = |value: &i32| value.to_string())]
    actual: i32,
}

impl Validate<i32> for EvenValidation {
    fn validate(&self, value: &i32) -> bool {
        *value % 2 == 0
    }
}

#[derive(Koruma, KorumaAllFluent)]
pub struct User {
    #[koruma(EvenValidation)]
    pub id: i32,
}
```

Use the application's selected localizer to render either a typed validator failure or the
aggregate generated error:

```rust,ignore
let user = User { id: 3 };

if let Err(errors) = user.validate() {
    println!("{}", i18n::localize(&errors));

    if let Some(error) = errors.id().even_validation() {
        println!("{}", i18n::localize(error));
    }

    for error in errors.id().all() {
        println!("{}", i18n::localize(&error));
    }
}
```

Here, `i18n::localize(...)` is an application helper around the selected `EmbeddedI18n`.
Passing the localizer through application state works the same way.

## Render element failures

Aggregate Fluent rendering joins direct, nested, and newtype messages with newlines. It does not
include failures produced by `each(...)`, because those messages need their element indices.
Enumerate and localize them directly:

```rust,ignore
for (index, element_errors) in errors.quantities().element_errors() {
    for error in element_errors.all() {
        println!("quantities[{index}]: {}", i18n::localize(&error));
    }
}
```

When a field has only `each(...)` rules, localize its element error values directly rather than
deriving aggregate `KorumaAllFluent` for the containing type. If aggregate rendering is required,
add a meaningful direct field rule as well.
