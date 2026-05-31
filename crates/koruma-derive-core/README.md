# koruma-derive-core

[![Docs](https://docs.rs/koruma-derive-core/badge.svg)](https://docs.rs/koruma-derive-core/)
[![Crates.io](https://img.shields.io/crates/v/koruma-derive-core.svg)](https://crates.io/crates/koruma-derive-core)

Parsing utilities for `koruma` derive macros. This crate exposes the data model for
`#[koruma(...)]` attributes, including Rust-native direct validator chains and setter access for
downstream tooling and proc-macro internals.

Data-field validators may be labeled with lower-snake identifiers, such as
`#[koruma(username_prefix = string::PrefixValidation::<_>::prefix("user:"))]` or
`#[koruma(each(tag_prefix = string::PrefixValidation::<_>::prefix("tag:")))]`.
Labels are carried on `ParsedValidatorUse` for downstream name generation.

`parse_field` returns `Result<Option<FieldInfo>, syn::Error>`: `Some` for fields that participate
in validation, `None` for skipped or unannotated fields, and `Err` for invalid metadata.
Participating fields expose a `ParsedFieldSpec` shape, which separates regular, nested, and
newtype validation so invalid combinations cannot be represented after parsing.

Most users should depend on `koruma` (or `koruma-derive`) instead of this crate directly.
