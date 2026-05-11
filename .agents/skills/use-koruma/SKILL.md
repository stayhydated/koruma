---
name: use-koruma
description: "Use when Codex needs to add, review, or update Rust validation built with koruma or koruma-collection, including custom validators, field #[koruma(...)] attributes, derived Koruma error accessors, KorumaAllDisplay or KorumaAllFluent rendering, nested validation, newtype validation, try_new or TryFrom checked constructors, per-element each(...) validation, koruma-collection validator selection, feature flags, validator messages, i18n, docs, and examples in this workspace."
---

# Use Koruma

## Overview

Use this skill for strongly typed per-field validation with `koruma`. Treat `koruma` and
`koruma-collection` as one workflow: `koruma` provides the traits, derive macros, validation
flow, and typed error accessors; `koruma-collection` provides common validators implemented on top
of those APIs.

## Choose the Surface

- Use `crates/koruma` as the normal user-facing entry point for application code.
- Use `koruma-collection` when a common string, format, numeric, collection, or general validator
  already fits the rule.
- Define a custom `#[validator]` only when the rule is domain-specific, needs custom stored error
  data, or needs custom `Display` or Fluent message behavior.
- Use `koruma-core`, `koruma-derive`, or `koruma-derive-core` directly only for integration,
  tooling, or macro internals.

## Inspect First

1. Read `examples/readme` for canonical executable examples.
2. Read the matching user-facing docs before changing public behavior:
   `README.md`, `crates/koruma/README.md`, `crates/koruma-collection/README.md`, and the relevant
   `book/src/*.md` page.
3. For built-in validators, read `references/validator-catalog.md` and verify feature flags against
   `crates/koruma-collection/Cargo.toml` when editing dependencies.
4. Keep implementation details in `docs/ARCHITECTURE.md`; keep READMEs and book pages
   example-first.

## Dependencies

Inside this workspace, follow the root `Cargo.toml` dependency model: define versions and path
dependencies at the workspace root, then use `workspace = true` in member crates. Non-example
crates should not add explicit path dependencies.

For external application examples:

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

## Synchronize Public Surfaces

When changing a public workflow, feature-flag story, validator inventory, validator message shape,
or user-visible API shape, update the same change across:

- `examples/readme` when relevant.
- The affected user-facing `README.md` files.
- The matching `book/src/*.md` pages.
- `crates/koruma-collection/README.md` and `book/src/koruma_collection.md` for collection
  inventory, feature flags, or usage guidance.
- `crates/koruma-collection/i18n/` and `Display` implementations when validator messages change.

For collection message changes, run `cargo run -p xtask -- sync-display-ftl --check` or
`cargo run -p xtask -- sync-display-ftl` as appropriate.

## Validation

Run the narrowest useful checks first, then broaden based on the touched surface:

- `cargo test -p koruma`
- `cargo test -p koruma-collection --features full-fluent`
- `cargo test -p readme`
- `cargo run -p xtask -- build-book` for book changes
- `cargo run -p xtask -- build-llms-txt` when generated public text surfaces must be refreshed
