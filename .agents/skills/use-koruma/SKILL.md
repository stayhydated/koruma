---
name: use-koruma
description: "Use only for user-facing guidance on applying koruma or koruma-collection in application Rust code, including built-in validator selection, custom validators, field #[koruma(...)] attributes, derived Koruma error accessors, KorumaAllDisplay or KorumaAllFluent rendering, nested validation, newtype validation, try_new or TryFrom checked constructors, per-element each(...) validation, feature flags, validator messages, and i18n. Do not use for generic Rust build/test tasks."
---

# Use Koruma

## Scope Boundary

Treat this skill as a hosted public-usage guide for `koruma` consumers. Use it
only for user-facing application workflows: choosing `koruma` or
`koruma-collection`, applying built-in validators, defining custom validators,
deriving typed error accessors, rendering validation failures, and connecting
validator messages to Fluent localization.

Do not use this skill as a contributor guide for `koruma` repository internals.
For build, test, format, lint, maintenance, release, or architecture work, read
the repository source, `AGENTS.md`, and the relevant crate documentation
directly.

## Core Workflow

Start from the user-facing facade. Most application code uses `koruma`, and
adds `koruma-collection` when built-in validators fit the rule:

1. Inspect the local application type, validation calls, and existing
   `Cargo.toml` feature setup before changing patterns.
2. Use `koruma` as the normal entry point for derives, core traits, and error
   rendering helpers.
3. Use `koruma-collection` when a common string, format, numeric, collection,
   or general validator already fits the rule.
4. Define a custom `#[validator]` only when the rule is domain-specific, needs
   custom stored error data, or needs custom `Display` or Fluent behavior.
5. Attach validators with field-level `#[koruma(...)]` attributes. Use
   `TypeName::<_>` for zero-configuration generic validators or
   `TypeName::<_>::first_setter(...)` when configuring generic validators.
   Optional fields and optional `each(...)` elements unwrap by default; wrap a
   validator in `full(...)` when it should receive the whole optional value.
   Add lower-snake labels with `label_name = Validator::<_>` when you need
   descriptive stable accessors or when validators would otherwise generate the
   same getter/variant name; labels work inside `each(...)` too.
6. Derive `Koruma` on the validated type. Add `KorumaAllDisplay` for `all()`
   iterators over borrowed `Display`-renderable validators, or `KorumaAllFluent`
   for localized borrowed `FluentMessage` values. Failed-validator inspection
   does not require validator types to implement `Clone`.
7. For Fluent, derive `EsFluent` on validators and render messages through an
   app-owned `es-fluent` localizer.

## Reference Selection

Load only the reference needed for the task:

- `references/validator-catalog.md`: built-in validator inventory, module names, feature flags, and usage notes.

Prefer current public docs or source examples over memory when details matter.
Prefer examples and concrete Rust snippets over prose-only guidance.

## Implementation Rules

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

Prefer collection modules when they match the rule:

```rust
use koruma::{Koruma, KorumaAllDisplay, Validate};
use koruma_collection::{collection, general, numeric, string};

#[derive(Koruma, KorumaAllDisplay)]
struct SignupInput {
    #[koruma(collection::NonEmptyValidation::<_>)]
    username: String,

    #[koruma(string::AsciiValidation::<_>, string::AlphanumericValidation::<_>)]
    handle: String,

    #[koruma(numeric::RangeValidation::<_>::min(13_u8).max(120_u8))]
    age: u8,

    #[koruma(full(general::RequiredValidation::<_>))]
    display_name: Option<String>,
}
```

Use `#[validator]`, mark the captured input with `#[koruma(value)]`, implement
`Validate<T>`, and derive or implement the error rendering needed by the caller:

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

Keep `#[koruma(value)]` fields private and use the generated getter when
external code needs the captured input. For validators that do not need to store
the failing input, use `#[koruma(value(capture = skip))]` on an `Option<T>` value
field so derived validation does not clone the original value. Built-in
collection, string, format, and numeric validators that do not render the
failing input use this pattern internally.

For direct setter generation on custom validators, use
`#[koruma(setter(...))]` on configuration fields. Supported setter options are
`into`, `required`, `name`, and `default`.

Read generated errors through field and validator accessors:

```rust
#[derive(Koruma, KorumaAllDisplay)]
pub struct Item {
    #[koruma(NumberRangeValidation::<_>::min(0).max(100))]
    pub age: i32,

    #[koruma(StringLengthValidation::min(1).max(67))]
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

Generated validator accessors are snake_case versions of validator type names
unless the validator is labeled with `label_name = Validator`, in which case
the generated accessor is `label_name()` and the `all()` enum variant uses
`LabelName`.
Fields without `#[koruma(...)]` are ignored. Multiple validators run in the
order listed, and all configured validators are evaluated.

Common patterns:

- Use `#[koruma(each(Validator::<_>))]` or `#[koruma(each(Validator::<_>::first_setter(...)))]` for per-element validation of `Vec<T>`, slices, arrays, and optional variants of those.
- Use `#[koruma(label_name = Validator::<_>)]` or `#[koruma(each(label_name = Validator::<_>))]` to select generated getter and `all()` variant names explicitly.
- Use `#[koruma(nested)]` when a field is another `Koruma` type and the parent should expose the nested error tree. Handwritten `ValidateExt` integrations must use an associated error type implementing `ValidationError + Default`.
- Use `#[koruma(newtype)]` for transparent error access through newtype wrappers.
- Add `#[koruma(try_new, newtype)]` to generate a checked `try_new` constructor.
- Add `#[koruma(newtype(try_from))]` to generate `TryFrom<Inner>` for checked conversions.
