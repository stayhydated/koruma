# koruma-derive-core

[![Docs](https://docs.rs/koruma-derive-core/badge.svg)](https://docs.rs/koruma-derive-core/)
[![Crates.io](https://img.shields.io/crates/v/koruma-derive-core.svg)](https://crates.io/crates/koruma-derive-core)

Parsing utilities for `koruma` derive macros. This crate exposes the data model for
`#[koruma(...)]` attributes and is intended for tooling or proc-macro internals.

## Installation

```toml
[dependencies]
koruma-derive-core = "0.3"
```

## Features

- `showcase`: enables parsing of `#[showcase(...)]` metadata.
