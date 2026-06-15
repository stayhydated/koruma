---
name: use-koruma
description: "Use only for user-facing guidance on applying koruma or koruma-collection in application Rust code, including feature flags, core traits, custom #[validator] validators and setter metadata, field #[koruma(...)] validators/modifiers/labels/target selectors, optional and per-element each(...) validation, generated error accessors, KorumaAllDisplay or KorumaAllFluent rendering, nested validation, newtype validation, try_new or TryFrom checked constructors, built-in validator selection, validator messages, and i18n. Do not use for generic Rust build/test tasks."
---

# Use Koruma

## Scope Boundary

Treat this skill as public, reusable guidance for `koruma` consumers. Use it
only for user-facing application workflows: choosing `koruma` or
`koruma-collection`, applying built-in validators, defining custom validators,
deriving typed error accessors, rendering validation failures, and connecting
validator messages to Fluent localization.

Do not use this skill for crate implementation design, maintenance automation,
release work, or contributor workflows. It describes consumer application usage
only.

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
   Each setter call takes exactly one argument. Put generic arguments on the
   validator path, not on the setter method.
   Put all validators and field modifiers for a field in one `#[koruma(...)]`
   attribute, separated by commas.
   Optional fields and optional `each(...)` elements unwrap by default. Use
   `Validator::<Option<_>>` or `full(Validator::<_>)` when the validator should
   receive the whole optional value, and `unwrapped(Validator::<_>)` only when
   you want to make the default target explicit.
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

- `references/koruma-feature-map.md`: complete public feature map for crates, feature flags,
  macros, attributes, target selection, errors, constructors, nested/newtype, and i18n.
- `references/validator-catalog.md`: built-in validator inventory, module names, feature flags, and usage notes.

Prefer current public docs and published examples over memory when details matter.
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
es-fluent = "0.16"
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

    #[koruma(general::RequiredValidation::<Option<_>>)]
    display_name: Option<String>,
}
```

Use `#[validator]`, keep one private captured input field named `actual`,
`input`, or `value`, implement `Validate<T>`, and derive or implement the error
rendering needed by the caller. Use `#[koruma(value)]` only when the captured
field needs another name:

```rust
use koruma::{Validate, validator};
use std::fmt;

#[validator]
#[derive(Clone, Debug)]
pub struct StringLengthValidation {
    min: usize,
    max: usize,
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

Keep captured value fields private and use the generated getter when external
code needs the captured input. For validators that do not need to store the
failing input, use `#[koruma(skip_capture)]` on an `Option<T>` value field so
derived validation does not clone the original value. Built-in
collection, string, format, and numeric validators that do not render the
failing input use this pattern internally.

`#[validator]` also emits `ValidatorMetadata<T>` for tooling. Use it when a
generic integration needs validator descriptors or runtime parameter values;
generic values may be reported as opaque to avoid adding trait bounds.

Unannotated configuration fields on custom validators generate direct setters.
Koruma infers an unmarked value field named `actual`, `input`, or `value`, and
can infer any field name when exactly one field is unmarked. Use bare
`#[koruma(setter)]` when a configuration field is named `actual`, `input`, or
`value`, or when marking configuration fields lets Koruma infer the only
remaining unmarked value field. Use `#[koruma(setter(...))]` when a setter
needs options. Supported setter options are `into`, `required`, `name`, and
`default`. Setter names that collide with generated builder APIs are rejected,
including `new`, `build`, `with_value`, `builder`, `__koruma_builder`,
`build_validator`, `capture_value_ref`, and the generated `maybe_` prefix.
Optional non-required `Option<T>` setter fields generate a direct setter that
takes `T` and a `maybe_*` setter that takes `Option<T>`. Use
`setter(required)` on `Option<T>` when callers must pass `Some(...)` or `None`
explicitly.

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

Generated aggregate error structs implement `ValidationIssues` for generic
field and element issue enumeration. Prefer the strongly typed accessors for
normal app code, and use `ValidationIssues` when building tooling that needs
field names, validator type names, labels, element indices, and messages.

Common patterns:

- Use `#[koruma(each(Validator::<_>))]` or `#[koruma(each(Validator::<_>::first_setter(...)))]` for per-element validation of `Vec<T>`, slices, arrays, and optional variants of those. Type aliases and custom collections are not resolved; recognized `Vec<T>` paths must resolve to `std::vec::Vec<T>`.
- Use `#[koruma(label_name = Validator::<_>)]` or `#[koruma(each(label_name = Validator::<_>))]` to select generated getter and `all()` variant names explicitly.
- Use `#[koruma(skip)]` to explicitly exclude a field from validation when a `#[koruma(...)]` marker would otherwise be expected nearby; fields with no `#[koruma(...)]` attribute are ignored by default.
- Use `#[koruma(full(Validator::<_>))]` or `Validator::<Option<_>>` for full optional-field or optional-element validation; use `#[koruma(unwrapped(Validator::<_>))]` to document the default unwrapped target.
- Use `#[koruma(nested)]` when a field is another `Koruma` type and the parent should expose the nested error tree. Handwritten `ValidateExt` integrations must use an associated error type implementing `ValidationError + Default`.
- Use `#[koruma(newtype)]` for transparent error access through newtype wrappers.
  A `newtype` field may also include ordinary field validators, such as
  `#[koruma(newtype, general::RequiredValidation::<Option<_>>)]`; those
  validators use the normal optional, `full(...)`, and `unwrapped(...)` target
  rules. Struct-level newtypes also implement `NewtypeValue` and
  `NewtypeTryFromInner` so integration code can borrow, consume, validate, and
  checked-reconstruct inner values even when wrapper fields are private.
- Add `#[koruma(try_new)]` to generate a checked constructor that takes the struct fields,
  validates the instance, and returns `Result<Self, Error>`.
- Use `NewtypeTryFromInner::try_from_inner` for checked reconstruction of
  struct-level newtypes, and add `#[koruma(try_from)]` only when you also need a
  standard-library `TryFrom<Inner>` impl.
- Use tuple structs when they fit the domain; `Koruma`, `try_new`, `try_from`, and struct-level
  `newtype` all support exactly-one-field tuple wrappers.
