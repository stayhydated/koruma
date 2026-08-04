# Koruma public feature map

Use this reference for application-facing Koruma behavior that is too detailed for the main skill.
Use [the validator catalog](validator-catalog.md) for built-in rules.

## Contents

- [Crates and features](#crates-and-features)
- [Custom validators](#custom-validators)
- [Field rules and targets](#field-rules-and-targets)
- [Generated errors](#generated-errors)
- [Rendering and localization](#rendering-and-localization)
- [Nested values](#nested-values)
- [Validated newtypes and constructors](#validated-newtypes-and-constructors)
- [Generic integration traits](#generic-integration-traits)

## Crates and features

Use `koruma` as the application facade. It re-exports core traits and, with its default `derive`
feature, `Koruma`, `KorumaAllDisplay`, and `#[validator]`. Enable `fluent` with `derive` for
`KorumaAllFluent`.

Use `koruma-collection` for built-in validators:

- `fmt` (default): `Display` messages.
- `full`: optional validators plus `SmallVec` and `rust_decimal::Decimal` support.
- `fluent`: es-fluent messages and `koruma/fluent`.
- `full-fluent`: `full` and `fluent`.
- Individual capabilities: `credit-card`, `email`, `phone-number`, `regex`, `url`,
  `smallvec`, and `rust_decimal`.

`koruma-core`, `koruma-derive`, and `koruma-derive-core` are public integration crates for
handwritten adapters, macros, and tooling. Application code normally uses the facade.

## Custom validators

Annotate a named-field struct with `#[validator]`, then implement `Validate<T>`.

- Koruma infers a private captured field named `actual`, `input`, or `value`.
- If no conventional name exists, exactly one unmarked field can be inferred. Use
  `#[koruma(value)]` when the choice is ambiguous.
- Keep the captured field private and use its generated getter outside the validator.
- Use `#[koruma(skip_capture)]` on an `Option<T>` captured field when the error does not need the
  failed input. This avoids cloning the input during derived validation.
- Unannotated configuration fields generate direct setters.
- Setter options are `into`, `required`, `name = ...`, `default`, and
  `default = expression`.
- A non-required `Option<T>` configuration setter accepts `T`; its `maybe_*` form accepts
  `Option<T>`. Use `required` when callers must explicitly choose `Some` or `None`.
- `#[validator]` also implements `ValidatorMetadata<T>` for static descriptors and runtime
  parameter values. Unsupported parameter representations remain opaque instead of adding bounds.

## Field rules and targets

Put one `#[koruma(...)]` attribute on each participating field and comma-separate its items.

- `Validator::<_>`: infer a generic validator's type from the selected target.
- `Validator::<_>.setter(value)`: configure a validator. Put generics on the validator path and
  pass one expression per setter.
- `label = Validator::<_>`: select stable getter and `all()` variant names. Labels must be
  lower-snake and unique within the generated surface.
- `skip`: explicitly ignore a field. An unannotated field on a regular struct is already ignored.
- `nested`: delegate to a child `ValidateExt` value.
- `newtype`: delegate transparently to a `NewtypeValidation` wrapper.
- `each(...)`: validate elements of a syntactic `Vec<T>`, slice, array, or optional form.
- `full(...)`: validate the complete optional field or optional element.
- `unwrapped(...)`: explicitly select the default inner target.

An optional target unwraps by default: `None` is skipped and `Some(T)` is validated as `T`.
Use `Validator::<Option<_>>` or `full(Validator::<_>)` when the validator implements
`Validate<Option<T>>`.

Collection recognition is syntactic. Aliases and custom collection types are not resolved, and a
recognized `Vec<T>` path must resolve to `std::vec::Vec<T>`.

## Generated errors

`#[derive(Koruma)]` implements `ValidateExt` and generates typed aggregate and field errors.

- `errors.field()` returns a direct field error container, a nested error option, or transparent
  newtype errors according to the field shape.
- `errors.field().validator_name()` returns `Option<&Validator>` for a direct failure.
- `errors.field().all()` borrows direct-validator failures in source order.
- `element_errors()` yields each failing element index and its typed error container.
- Koruma evaluates every validator for a present target instead of stopping at the first failure.
- Failed-validator inspection borrows stored values and does not require validator types to
  implement `Clone`.

Generated aggregate errors implement `ValidationIssues`. Direct field and element failures become
individual issues with field names, validator names, labels, indices, and fallback messages.
Nested and newtype failures become parent-level summary issues; recurse through the typed delegated
error for individual child issues.

Use the typed failed-validator getter with `ValidatorMetadata::validator_params()` when generic
tooling also needs configured parameter values.

## Rendering and localization

Derive `KorumaAllDisplay` when direct and element `all()` values must implement `Display`.
Every stored validator on that surface must implement `Display`.

Derive `KorumaAllFluent` when those values must implement `es_fluent::FluentMessage`. Enable
`koruma/fluent`, derive or implement `FluentMessage` for custom validators, and render through
an application-owned localizer.

Aggregate Fluent strings include direct, nested, and newtype messages, joined with newlines. They
omit `each(...)` failures so callers can preserve element indices. Localize the values returned by
each element error's `all()` or typed accessors. When a type has an `each(...)`-only field, use
that element-level path instead of deriving aggregate `KorumaAllFluent`; add a meaningful direct
field rule when aggregate rendering is required.

For built-in localized messages, enable `koruma-collection/fluent` or `full-fluent`.

## Nested values

Use `#[koruma(nested)]` for a child that implements `ValidateExt`.

In a regular parent struct, the generated nested accessor returns `Option<&ChildError>` for both
required and optional child fields. It is `Some` only when the child produced errors. An optional
child also skips validation for `None`.

A handwritten `ValidateExt` implementation must use
`type Error: ValidationError + Default`.

A struct-level newtype whose single field uses nested delegation stores that inner error directly;
this is part of the wrapper's transparent error surface.

## Validated newtypes and constructors

Apply struct-level `#[koruma(newtype)]` to an exactly-one-field named or tuple struct.

- Give the field a validator, `nested`, or `newtype` rule.
- An unannotated field delegates to an inner `NewtypeValidation` value.
- Struct-level `newtype` implements `NewtypeValidation`, `NewtypeValue`, and
  `NewtypeTryFromInner`.
- `NewtypeValue` provides `as_inner`, `into_inner`, and `validate_inner`.
- `NewtypeTryFromInner::try_from_inner` provides generic checked reconstruction.
- `#[koruma(try_new)]` adds an inherent constructor that accepts every struct field and validates
  the result.
- `#[koruma(try_from)]` adds `TryFrom<Inner>` for an exactly-one-field struct.
- Use `try_from` without `newtype` when a one-field struct should keep the regular error shape.

Field-level `#[koruma(newtype)]` exposes a required wrapper's inner error directly and an optional
wrapper's error as `Option<&InnerError>`. Direct validators can be added to the same field; then
the generated field container exposes delegated errors through `inner()` and yields a non-empty
delegated error as `Inner` from `all()`.

## Generic integration traits

- `Validate<T>`: validator runtime contract.
- `ValidatorMetadata<T>`: validator descriptors and configured parameter values.
- `ValidationError`: `is_empty()` and `has_errors()` for error values.
- `ValidationIssues`: generic issue enumeration.
- `ValidateExt`: validation contract implemented by `Koruma`.
- `NewtypeValidation`: marker for transparent validated wrappers.
- `NewtypeValue`: generic inner-value access and validation.
- `NewtypeTryFromInner`: generic checked reconstruction.
