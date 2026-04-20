# koruma-derive-core

[![Docs](https://docs.rs/koruma-derive-core/badge.svg)](https://docs.rs/koruma-derive-core/)
[![Crates.io](https://img.shields.io/crates/v/koruma-derive-core.svg)](https://crates.io/crates/koruma-derive-core)

Parsing utilities for `koruma` derive macros. This crate exposes the data model for
`#[koruma(...)]` attributes, including both shorthand validator args and Rust-native builder-chain
forms, and is intended for tooling or proc-macro internals.

Most users should depend on `koruma` (or `koruma-derive`) instead of this crate directly.
