---
name: use-koruma
description: "Use only for user-facing guidance on applying koruma or koruma-collection in application Rust code, including built-in validator selection, custom validators, field #[koruma(...)] attributes, derived Koruma error accessors, KorumaAllDisplay or KorumaAllFluent rendering, nested validation, newtype validation, try_new or TryFrom checked constructors, per-element each(...) validation, feature flags, validator messages, and i18n. Do not use for generic Rust build/test tasks."
---

# Use Koruma

## Overview

This skill is user-facing only. Use it to help application developers apply strongly typed
per-field validation with `koruma` and `koruma-collection` in their own Rust code.

Do not add build, test, format, lint, or other verification steps here; those belong to the
normal Rust workflow.

## Choose the Surface

- Use `koruma` as the normal entry point for application code.
- Use `koruma-collection` when a common string, format, numeric, collection, or general validator
  already fits the rule.
- Define a custom `#[validator]` only when the rule is domain-specific, needs custom stored error
  data, or needs custom `Display` or Fluent message behavior.

## Inspect First

1. Read the local application code using `koruma` before changing patterns.
2. For built-in validators, read `references/validator-catalog.md` when you need inventory,
   module names, or feature flags.
3. Prefer examples and concrete Rust snippets over prose-only guidance.

## Dependencies

For application code:

```toml
[dependencies]
koruma = { version = "*" }
koruma-collection = { version = "*", features = ["full"] }
```

For localized messages:

```toml
[dependencies]
koruma = { version = "*", features = ["derive", "fluent"] }
koruma-collection = { version = "*", features = ["full-fluent"] }
es-fluent = { version = "*", features = ["derive"] }
```

## Use Built-In Validators

Prefer the collection modules when they match the rule:

```rust
use koruma::{Koruma, KorumaAllDisplay, Validate};
use koruma_collection::{collection, general, numeric, string};

#[derive(Koruma, KorumaAllDisplay)]
struct SignupInput {
    #[koruma(collection::NonEmptyValidation::<_>::builder())]
    username: String,

    #[koruma(string::AsciiValidation::<_>::builder(), string::AlphanumericValidation::<_>::builder())]
    handle: String,

    #[koruma(numeric::RangeValidation::<_>::builder().min(13_u8).max(120_u8))]
    age: u8,

    #[koruma(general::RequiredValidation::<Option<_>>::builder())]
    display_name: Option<String>,
}
```

Use `TypeName::<_>::builder()...` for generic validators. Builder syntax gives standard Rust
method completion for validator settings and computed values.

## Define Custom Validators

Use `#[validator]`, mark the captured input with `#[koruma(value)]`, implement `Validate<T>`, and
derive or implement the error rendering needed by the caller.

```rust
use koruma::{Validate, validator};
use std::fmt;

#[validator]
#[derive(Clone, Debug)]
pub struct StringLengthValidation {
    min: usize,
    max: usize,
    #[koruma(value)]
    input: String,
}

impl Validate<String> for StringLengthValidation {
    fn validate(&self, value: &String) -> bool {
        let len = value.chars().count();
        len >= self.min && len <= self.max
    }
}

impl fmt::Display for StringLengthValidation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "String length {} must be between {} and {} characters",
            self.input.chars().count(),
            self.min,
            self.max
        )
    }
}
```

Keep `#[koruma(value)]` fields private and use the generated getter when external code needs the
captured input. For presence-only validators, use `#[koruma(value, skip_capture)]` on an
`Option<T>` value field so missing-value errors do not require cloning the original value.

## Attach and Read Errors

Derive `Koruma` on the validated type. Add `KorumaAllDisplay` for `all()` iterators over
`Display`-renderable validators, or `KorumaAllFluent` for localized `FluentMessage` values.

```rust
#[derive(Koruma, KorumaAllDisplay)]
pub struct Item {
    #[koruma(NumberRangeValidation::<_>::builder().min(0).max(100))]
    pub age: i32,

    #[koruma(StringLengthValidation::builder().min(1).max(67))]
    pub name: String,
}

if let Err(errors) = item.validate() {
    if let Some(age_err) = errors.age().number_range_validation() {
        println!("age failed: {age_err}");
    }

    for failed in errors.name().all() {
        println!("name validator: {failed}");
    }
}
```

Generated validator accessors are snake_case versions of the validator type names. Fields without
`#[koruma(...)]` are ignored. Multiple validators run in the order listed, and all configured
validators are evaluated.

## Common Patterns

- Use `#[koruma(each(Validator::<_>::builder()...))]` for per-element validation of `Vec<T>`, slices, arrays,
  and optional variants of those.
- Use `#[koruma(nested)]` when a field is another `Koruma` type and the parent should expose the
  nested error tree.
- Use `#[koruma(newtype)]` for transparent error access through newtype wrappers.
- Add `#[koruma(try_new, newtype)]` to generate a checked `try_new` constructor.
- Add `#[koruma(newtype(try_from))]` to generate `TryFrom<Inner>` for checked conversions.
- For Fluent, derive `EsFluent` on validators, derive `KorumaAllFluent` on consumers, and render
  messages through an app-owned `es-fluent` localizer.
