# Architecture: koruma-derive-core

## Purpose

`koruma-derive-core` owns the parsing layer for `#[koruma(...)]` attributes. It provides a stable API for derive macros and external tooling to inspect koruma validation metadata without depending on the proc-macro crate.

## Modules

- `crates/koruma-derive-core/src/parse.rs`: public parser re-exports and `SpannedValue<T>`
  source provenance helpers.
- `crates/koruma-derive-core/src/parse/diagnostics.rs`: context-specific diagnostic builders
  shared by the parser modules, including accepted-item lists and validator-field option errors.
- `crates/koruma-derive-core/src/parse/keywords.rs`: centralized Koruma attribute keywords and
  reserved generated builder methods used by context diagnostics.
- `crates/koruma-derive-core/src/parse/validator_chain.rs`: direct validator-chain grammar,
  type-argument parsing, and builder setter call collection.
- `crates/koruma-derive-core/src/parse/data_field.rs`: derive data-field grammar for
  modifiers, labels, explicit target selectors, and `each(...)`.
- `crates/koruma-derive-core/src/parse/derive_struct.rs`: struct-level `try_new`,
  `newtype`, and `newtype(try_from)` grammar.
- `crates/koruma-derive-core/src/parse/validator_struct.rs`: validator-struct field grammar
  for `value`, `value(capture = skip)`, and `setter(...)`.
- `crates/koruma-derive-core/src/parse/showcase.rs`: showcase-only metadata grammar behind the
  `internal-showcase` feature.
- `crates/koruma-derive-core/src/utils.rs`: syntax-only type helpers (`KnownTypeShape`, Option/Vec inference, and placeholder substitution).
- `crates/koruma-derive-core/src/tests`: snapshot coverage for parsing behavior.

## Data model

- `ValidatorPath`: a non-empty validator path wrapper. It preserves the parsed `syn::Path` while making the terminal validator name available without relying on unchecked raw path construction.
- `ValidatorAttr`: a single direct validator chain (non-empty path, accessor-backed setter calls, type inference flags, and setter access via `setter_calls()`). Setter call arguments are represented as `ValidatorSetterArg::Expr` nodes that preserve normal Rust expressions.
- `SpannedValue<T>`: a small wrapper used when semantic nodes need to carry the source token
  span that introduced a marker, label, selector, or option.
- `ValidatorLabel`: a validated lower-snake label with its source span, used for label-aware
  generated names.
- `ParsedValidatorUse`: a single validator occurrence on a data field. It carries the parsed
  validator, an optional `ValidatorLabel`, target selection, and the source span used for
  diagnostics.
- `StructKorumaAttr` / `StructKorumaItem`: struct-level `#[koruma(...)]` grammar for `try_new`, `newtype`, and `newtype(try_from)`.
- `DataFieldKorumaAttr` / `DataFieldKorumaItem`: data-field `#[koruma(...)]` grammar for modifiers, direct field validators, optional `label = Validator` labels, explicit `full(...)`/`unwrapped(...)` target selectors, and `each(...)` element validators without a raw token bucket. Attribute items and field/element validation specs expose read-only accessors instead of public mutable vectors.
- `ValidatorStructSpec` / `ValidatorFieldSpec`: normalized validator-struct metadata for
  `#[koruma::validator]`. The spec proves that exactly one field is marked `#[koruma(value)]`,
  value fields do not also define setter metadata, and setter defaults are not combined with
  `required`. Field, value, and setter details are exposed through accessors rather than public
  fields.
- `ValidatorValueSpec` / `ValidatorSetterSpec` / `SetterInputPolicy` / `SetterPresence`:
  typed validator-field `#[koruma(...)]` grammar for `value`, `value(capture = skip)`, and
  `setter(...)`. Setter input conversion and required/optional/defaulted presence are normalized
  as enums rather than independent flags.
- `FieldSource`: source metadata derived from `syn::Field`, shared by unannotated, skipped, and participating data fields.
- `ParsedDataField`: explicit data-field participation after field-level `#[koruma(...)]` parsing. It preserves unannotated fields, explicit skip markers, and participating `FieldInfo` as separate states.
- `ParsedFieldSpec`: normalized field shape for participating fields. It is an enum with `Regular`, `Nested`, and `Newtype` variants so parser output cannot encode skipped-with-validator, nested-with-validator, or newtype-with-element-validator states.
- `FieldInfo`: participating per-field metadata derived from `syn::Field`.
- `StructOptions` / `StructMode`: normalized struct-level shape and constructor intent. Regular structs can request `try_new`; newtype structs carry their `newtype` marker and can request `try_new`, `try_from`, or both.
- `KnownTypeShape`: centralized syntactic type recognition for `Option`, `Vec`, slices, arrays, and references, including the recognized path segment or syntax span for diagnostics. It is deliberately not Rust type resolution and does not resolve aliases or custom collection types.
- `ValidatorStructSpec` / `CapturePolicy`: typed metadata for `#[koruma::validator]` structs,
  including the field marked `#[koruma(value)]` and whether capture clones the borrowed input or
  uses `capture = skip`.
- `ShowcaseAttr` (feature `internal-showcase`): parsed `#[showcase(...)]` metadata, including required explicit `input_type`.

## Parsing notes

- `parse_field` parses the field-level `#[koruma(...)]` attribute with `DataFieldKorumaAttr`, handles `skip`, `nested`, `newtype`, and `each(...)`, and returns unannotated fields, skipped fields, and participating fields as separate typed states. Duplicate generated-name diagnostics are handled by the derive planning layer where the full field scope is known.
- `parse_struct_options` parses struct-level attributes with `StructKorumaAttr` and normalizes `try_new`, `newtype`, and `newtype(try_from)` into `StructOptions`.
- Field modifiers, struct option markers, target selectors, labels, setter options, and
  `each(...)` markers carry source spans so duplicate and conflict diagnostics can point at the
  second actionable token instead of falling back to the whole field or attribute.
- Reused context and option diagnostics are built in `parse/diagnostics.rs`; parser modules keep
  local `syn::Error::new` calls only for shape-specific messages whose wording depends on local
  grammar state.
- `parse_field` respects `cfg_attr` via `syn-cfg-attr` helpers.
- Generic validator bindings use standard Rust direct validator chains (`Validator::<_>::min(...)`) for type inference and substitution.
- Data-field validators carry a `ValidatorTargetSelector`. Bare validators and
  `unwrapped(Validator::<_>)` use the default unwrapped optional target, while
  `full(Validator::<_>)` selects the full field or element target explicitly.
  Parser output keeps Rust type arguments intact; `koruma-derive` resolves
  inferred types after field and element types are known.
- `parse_validator_struct` parses validator-field attributes separately from data-field
  attributes, locates `#[koruma(value)]`, normalizes setter metadata, and validates validator-field
  grammar before `koruma-derive` code generation.
- `find_showcase_attr` (feature `internal-showcase`) parses showcase metadata on validators and rejects missing or invalid `input_type`.

## Feature flags

- `internal-showcase`: enables parsing of `#[showcase(...)]` metadata used for validator registries.

## Tests

- Snapshot tests under `crates/koruma-derive-core/src/tests` validate parser output.
