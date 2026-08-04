# Validate newtypes and construct checked values

Use struct-level `#[koruma(newtype)]` for an exactly-one-field wrapper whose validation errors
should be exposed through the wrapped field. Add a checked-construction option that matches the API
your callers need.

## Choose the construction API

| Option or trait | Result |
| --- | --- |
| `#[koruma(newtype)]` | Transparent inner validation errors plus the newtype traits |
| `#[koruma(try_new)]` | An inherent `try_new(...)` that validates all struct fields |
| `#[koruma(try_from)]` | A standard `TryFrom<Inner>` implementation for a one-field struct |
| `NewtypeValue` | `as_inner`, `into_inner`, and `validate_inner` |
| `NewtypeTryFromInner` | Generic checked reconstruction with `try_from_inner` |

Struct-level `newtype` implements `NewtypeValue` and `NewtypeTryFromInner`, including for
private fields.

## Define a checked newtype

Give the single field a validator, `nested`, or `newtype` rule:

```rust,ignore
use koruma::{
    Koruma, KorumaAllDisplay, NewtypeTryFromInner as _, NewtypeValue as _,
};
use koruma_collection::collection;

#[derive(Debug, Koruma, KorumaAllDisplay)]
#[koruma(newtype, try_from)]
pub struct Username(
    #[koruma(collection::NonEmptyValidation::<_>)]
    String,
);

let errors = Username::try_from(String::new())
    .expect_err("empty username should fail");
assert!(errors.non_empty_validation().is_some());

let username = Username::try_from_inner("alice".to_string())?;
assert_eq!(username.as_inner(), "alice");

let raw = username.into_inner();
assert!(Username::validate_inner(&raw).is_ok());
```

Import `NewtypeValue` and `NewtypeTryFromInner` before calling their trait methods. Use
`try_new` instead when an inherent constructor is preferable; for a one-field wrapper it accepts
the inner value and returns `Result<Self, Error>`.

An unannotated field on a struct-level newtype delegates validation to its inner type, so that type
must implement `NewtypeValidation`.

## Use the newtype in another struct

Field-level `#[koruma(newtype)]` validates the wrapper and exposes its inner errors directly:

```rust,ignore
#[derive(Koruma)]
pub struct Login {
    #[koruma(newtype)]
    pub username: Username,
}

let errors = Login {
    username: Username(String::new()),
}
.validate()
.expect_err("username should fail");

assert!(errors.username().non_empty_validation().is_some());
```

For `Option<Newtype>`, `None` is skipped and the generated accessor returns
`Option<&InnerError>`. A newtype field can also have direct validators:

```rust,ignore
#[koruma(
    newtype,
    koruma_collection::general::RequiredValidation::<Option<_>>,
)]
pub email: Option<Email>;
```

With direct validators, the field error container exposes delegated errors through `inner()`;
`all()` includes a non-empty delegated error as `Inner`.

## Keep normalization outside validation

Normalize or parse external text before checked construction. Use
`string::CanonicalFormValidation::<_>.predicate(...)` when a storage or API boundary must reject
a value that is not already canonical. Keep every ingress path, including `FromStr`, `TryFrom`,
serde conversions, and application constructors, aligned on the same representation.

Named-field and tuple wrappers are both supported, but a struct-level newtype must contain exactly
one field.
