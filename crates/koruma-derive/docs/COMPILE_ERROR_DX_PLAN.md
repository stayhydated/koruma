# Compile Error DX Plan

This plan tracks follow-up improvements for derive and validator combinations
that can be reported earlier or with clearer compiler diagnostics.

## Current Repo State

As of 2026-06-04, `koruma` already has a strong parser and planning layer for
many user mistakes:

- `koruma-derive-core` rejects invalid attribute contexts, duplicate markers,
  unsupported `each(...)` collection shapes, invalid labels, invalid
  `setter(...)` options, and invalid `nested` / `newtype` combinations before
  token generation.
- `crates/koruma-derive/tests/ui.rs` already runs compile-fail and pass
  trybuild fixtures.
- `crates/koruma-derive/tests/ui-pass/renamed_koruma.rs` already verifies a
  renamed `koruma` facade for normal `Koruma` derive output.
- `koruma-core::__private` already has hidden generated-code traits:
  `BuildValidator` and `CaptureValueRef`.
- Current generated-code diagnostics still leak through for some semantic
  contracts. Notable examples are `missing_required_builder_setter.stderr`,
  which reports a missing generated `build` method, and
  `non_clone_capture.stderr`, which reports raw `Clone` / `CaptureValueRef`
  trait errors.

## Guiding Rule

Prefer direct parse or planning validation when the derive macro can see the
complete invalid state. Use generated assertion helpers for trait contracts that
depend on Rust type checking, such as `Validate<T>`, `Display`,
`FluentMessage`, `ValidateExt`, or builder readiness.

Use hidden marker-supertrait checks only for cross-derive contracts where two
independent derive macros need to prove that they were both expanded on the
same Rust type.

Generated assertions and markers should:

- stay behind `koruma::__private` or another hidden generated module,
- preserve generic parameters and where clauses,
- use the facade path discovered by `koruma_crate_path()`,
- use `quote_spanned!` or equivalent span placement so diagnostics point at the
  user-authored derive, field attribute, validator path, or marker that caused
  the issue,
- keep support for a renamed `koruma` facade.

## 1. Require `Koruma` For `KorumaAllDisplay` And `KorumaAllFluent`

Status: Complete. `KorumaAllDisplay` and `KorumaAllFluent` build a
`ValidationPlan` and generate impls for borrowed validator ref enums, but those
enums are emitted by `#[derive(Koruma)]`. There is no hidden marker proving
that `Koruma` also ran on the type.

Problem: deriving `KorumaAllDisplay` or `KorumaAllFluent` without `Koruma`
can produce missing generated-type errors or silently emit no useful output for
some shapes. The public docs already describe these derives as companions to
`Koruma`.

Implementation:

- Add a hidden marker trait in `koruma-core`, for example
  `koruma::__private::KorumaWasDerived`.
- Have `#[derive(Koruma)]` emit an impl of that marker for the source type when
  expansion succeeds.
- Have `KorumaAllDisplay` emit a requirement impl that requires
  `KorumaWasDerived`.
- Have `KorumaAllFluent` emit the same requirement when the `fluent` feature is
  enabled.
- Preserve source generics and where clauses with `split_for_impl()`.
- Keep the check independent of derive order.
- Prefer a diagnostic message that says `KorumaAllDisplay` /
  `KorumaAllFluent` must be derived together with `Koruma`.

Validation:

- Add trybuild failures for `#[derive(KorumaAllDisplay)]` without `Koruma`.
- Add trybuild failures for `#[derive(KorumaAllFluent)]` without `Koruma` under
  the `fluent` feature.
- Add passing coverage for `#[derive(Koruma, KorumaAllDisplay)]` and
  `#[derive(Koruma, KorumaAllFluent)]` on generic structs.
- Extend or mirror the renamed-facade fixture so the marker paths do not assume
  the dependency name is `koruma`.
- Update codegen snapshots for the marker and requirement impls.

Doc impact:

- Public docs already describe these derives as companion derives. Add a short
  note only if the diagnostic wording introduces a new user-visible rule.

## 2. Add Validator Readiness Assertions Near `#[koruma(...)]` Validator Uses

Status: Complete. Validation code currently checks builder readiness, capture
policy, and `Validate<T>` through generated expressions inside
`render_validation_check`. Missing required setters can surface as a missing
`build` method on the builder, and invalid validator target types can surface
as generated-code trait errors.

Problem: users should get diagnostics near the specific validator occurrence in
`#[koruma(...)]`, not in generated validation body internals.

Implementation:

- Carry `ParsedValidatorUse::source_span()` into `PlannedValidator` so each
  planned validator keeps the original validator path span.
- In or near `render_validation_check`, generate a small assertion helper for
  each validator occurrence.
- Assert that the builder expression type implements
  `CaptureValueRef<validation_target_ty>`.
- Assert that the captured output implements `BuildValidator`.
- Assert that the final validator type implements `Validate<validation_target_ty>`.
- Do not use derive markers for validators. Manual `Validate<T>`
  implementations and handwritten validators must remain valid.
- Use `quote_spanned!` at the validator occurrence span so errors point at the
  attribute item that needs fixing.
- Keep the runtime validation path unchanged for valid validators.

Validation:

- Convert or supplement `missing_required_builder_setter.rs` so the expected
  failure points at the validator use or omitted required setter context rather
  than a generated `build` method.
- Add trybuild failures for a validator type that does not implement
  `Validate<T>`.
- Add trybuild failures for a validator that implements `Validate<U>` but is
  used against target type `T`.
- Add passing cases for manual `Validate<T>` implementations, generic
  validators using `::<_>`, explicit `full(...)`, and `unwrapped(...)`.
- Update snapshots for any changed generated assertions.

Doc impact:

- No public docs needed unless wording clarifies required setters or
  `Validate<T>` target selection.

## 3. Improve Non-`Clone` Capture Diagnostics

Status: Complete. `#[koruma(value)]` capture currently adds a generated
`T: Clone` bound for `CaptureValueRef`. When the value type is not `Clone`, the
current trybuild fixture reports raw `Clone` and `CaptureValueRef` trait errors.
`#[koruma(skip_capture)]` already has direct validation that it must be used on
an `Option<T>` field.

Problem: the compiler should explain that default value capture stores a clone
of the validated input, and that users can either make the value cloneable or
use `#[koruma(skip_capture)]` with `Option<T>` when they do not need to store
the value.

Implementation:

- Keep the existing `Clone` bound for valid generated code.
- Add a generated assertion on the validator value field span when
  `CapturePolicy::CloneInput` is used.
- Phrase the error around Koruma's capture behavior rather than exposing only
  `CaptureValueRef` internals.
- Consider a hidden helper trait or assertion function whose name appears in
  diagnostics as a clearer requirement than the current builder impl.
- Preserve generic validator support.

Validation:

- Update `non_clone_capture.rs` / `.stderr` to expect the focused diagnostic.
- Keep `ui-pass/capture_skip_non_clone.rs` passing.
- Add a generic non-`Clone` fixture if the assertion has different generic
  behavior.

Doc impact:

- Public docs already mention `skip_capture` for non-`Clone` payloads. Update
  only if the diagnostic introduces new wording or examples worth mirroring.

## 4. Assert `NewtypeValidation` For Field-Level `#[koruma(newtype)]`

Status: Complete. `NewtypeValidation` already exists in `koruma-core` and is
implemented by `#[derive(Koruma)]` when the struct-level `#[koruma(newtype)]`
mode is used. Field-level `#[koruma(newtype)]` currently relies mainly on
`ValidateExt`, associated error types, and generated `.validate()` calls.

Problem: `#[koruma(newtype)]` on a field is a stronger user promise than
`#[koruma(nested)]`: it expects transparent newtype error access. If the field
type only implements ordinary nested validation, or does not derive `Koruma`,
the error should point at the field marker.

Implementation:

- Generate a span-focused assertion for each field marked `#[koruma(newtype)]`
  requiring the unwrapped field type to implement `koruma::NewtypeValidation`.
- Use the `newtype` marker span when available.
- Keep `#[koruma(nested)]` separate: it should require `ValidateExt`, not
  `NewtypeValidation`.
- Preserve optional newtype fields by asserting the unwrapped inner type.

Validation:

- Add trybuild failures for `#[koruma(newtype)]` on a type that derives
  ordinary `Koruma` but is not a struct-level newtype.
- Add trybuild failures for `#[koruma(newtype)]` on a type that does not
  implement `ValidateExt`.
- Add passing coverage for required and optional validated newtypes.

Doc impact:

- Existing newtype docs likely stay valid. Add a short note only if the current
  wording does not distinguish `nested` from `newtype` strongly enough.

## 5. Assert `ValidateExt` For `#[koruma(nested)]`

Status: Complete. Nested validation currently calls `.validate()` on the nested
field value. If the nested type does not derive `Koruma` or implement
`ValidateExt`, diagnostics can appear as ordinary method or trait-resolution
errors in generated code.

Problem: users should see that `#[koruma(nested)]` requires the nested type to
derive `Koruma` or otherwise implement `ValidateExt`.

Implementation:

- Generate a span-focused assertion for each field marked `#[koruma(nested)]`
  requiring the unwrapped field type to implement `koruma::ValidateExt`.
- Use the `nested` marker span when available.
- Preserve optional nested fields by asserting the unwrapped inner type.
- Keep the existing validation body unchanged.

Validation:

- Add trybuild failures for required and optional nested fields whose inner
  type lacks `ValidateExt`.
- Add passing coverage for required and optional nested structs that derive
  `Koruma`.

Doc impact:

- No docs needed unless the nested validation chapter needs a stronger
  one-line prerequisite.

## 6. Assert `Display` And `FluentMessage` Requirements Near Companion Derives

Status: Complete. Public docs state that `KorumaAllDisplay` requires stored
validators to implement `Display`, and `KorumaAllFluent` requires stored
validators to implement `es_fluent::FluentMessage`. `KorumaAllFluent` adds
where-clause bounds; `KorumaAllDisplay` delegates directly to `Display::fmt`.
Neither path has targeted trybuild coverage for missing display or fluent
implementations.

Problem: users should get focused diagnostics near the companion derive or the
validator use, rather than opaque failures inside generated `fmt` or
`to_fluent_string_with` bodies.

Implementation:

- For `KorumaAllDisplay`, generate assertions that every stored field,
  element, and delegated newtype error value implements `Display`.
- For `KorumaAllFluent`, generate assertions that every stored field, element,
  delegated newtype error value, and main error storage value implements
  `FluentMessage`.
- Prefer spans on validator uses when checking validator types. Use the
  companion derive span for aggregate or delegated error-type checks.
- Keep existing where clauses for valid generated impls.

Validation:

- Add trybuild failures for `KorumaAllDisplay` with a validator lacking
  `Display`.
- Add trybuild failures for `KorumaAllFluent` with a validator lacking
  `FluentMessage`.
- Add passing coverage for manual `Display`, derived `EsFluent`, and generic
  validators with appropriate bounds.

Doc impact:

- Public docs already describe these requirements. No doc change unless the
  diagnostic wording changes recommended usage.

## Suggested Order

1. Implement item 1 first. It is the clearest cross-derive contract and can
   reuse the same marker-supertrait pattern used in `es-fluent`.
2. Implement item 2 next. It centralizes most validator-use trait diagnostics
   at the existing validation-check rendering point.
3. Implement item 3 after item 2, because non-`Clone` capture overlaps with
   `CaptureValueRef` readiness.
4. Implement items 4 and 5 together. They are field-marker assertions over
   `ValidateExt` and `NewtypeValidation`.
5. Implement item 6 last. It is mostly companion-derive assertion coverage and
   may be lower priority once item 1 prevents missing companion output.

## Handoff Checklist

- Add focused trybuild fixtures for every new compile-time failure.
- Add passing trybuild coverage for manual implementations, generics, optional
  fields, and renamed `koruma` facade paths where relevant.
- Update insta snapshots for generated marker or assertion tokens.
- Run `cargo fmt`.
- Run `cargo test -p koruma-derive-core -p koruma-derive` for derive changes.
- Run `cargo test -p koruma` when public facade or integration behavior is
  touched.
- Update README, book, examples, and `.agents/skills/use-koruma` guidance only
  for behavior that changes public application workflow or recommended usage.
