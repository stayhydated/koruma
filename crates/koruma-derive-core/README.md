# koruma-derive-core

[![Docs](https://docs.rs/koruma-derive-core/badge.svg)](https://docs.rs/koruma-derive-core/)
[![Crates.io](https://img.shields.io/crates/v/koruma-derive-core.svg)](https://crates.io/crates/koruma-derive-core)

Parsing utilities for `koruma` derive macros. This crate exposes typed parser data for
`#[koruma(...)]` attributes, including Rust-native direct validator chains and read-only setter
access for downstream tooling and proc-macro internals.

The parser API is split by macro context: data-field attributes, struct-level options,
validator-struct fields, direct validator chains, and showcase metadata each have their own
typed parser surface.
Each target accepts one `#[koruma(...)]` attribute; multiple items are expressed inside that
attribute with comma-separated syntax.

Data-field validators may be labeled with lower-snake identifiers, such as
`#[koruma(username_prefix = string::PrefixValidation::<_>::prefix("user:"))]` or
`#[koruma(each(tag_prefix = string::PrefixValidation::<_>::prefix("tag:")))]`.
Labels are carried on `ParsedValidatorUse` for downstream name generation. Validator paths are
stored as non-empty `ValidatorPath` values, so helpers such as `ValidatorAttr::name()` do not
depend on caller-constructed raw paths.

`parse_field` returns `Result<ParsedDataField, syn::Error>`, preserving the difference between
unannotated fields, explicit `#[koruma(skip)]` fields, and fields that participate in validation.
Participating fields expose `FieldInfo` with a `ParsedFieldSpec` shape, which separates regular,
nested, and newtype validation so invalid combinations cannot be represented after parsing.
Raw data-field attribute items are exposed through read-only accessors such as
`DataFieldKorumaAttr::items()` instead of public mutable fields.

`parse_validator_struct` returns opaque `ValidatorStructSpec` metadata for `#[koruma::validator]`
structs. It keeps `value`, `value(capture = skip)`, and `setter(...)` metadata typed behind
accessors in this crate before the proc-macro renderer builds validator builders. Setter metadata
uses `SetterInputPolicy` and `SetterPresence` so tooling sees exact, into, required, optional, and
defaulted intent without combining raw boolean flags.

Most users should depend on `koruma` (or `koruma-derive`) instead of this crate directly.
