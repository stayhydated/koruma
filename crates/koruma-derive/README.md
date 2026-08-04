# koruma-derive

[![API docs](https://docs.rs/koruma-derive/badge.svg)](https://docs.rs/koruma-derive/)
[![Crates.io](https://img.shields.io/crates/v/koruma-derive.svg)](https://crates.io/crates/koruma-derive)

`koruma-derive` provides Koruma's procedural macros. Application code should normally enable the
default `derive` feature on [`koruma`](https://crates.io/crates/koruma) instead of depending on
this crate directly.

The crate provides:

- `#[validator]` for declaring configurable validators;
- `#[derive(Koruma)]` for validation and typed error generation;
- `#[derive(KorumaAllDisplay)]` for displayable failed-validator views; and
- `#[derive(KorumaAllFluent)]` with the `fluent` feature for localized views.

See the [Koruma book](https://stayhydated.github.io/koruma/book/) for application workflows and the
[API reference](https://docs.rs/koruma-derive/) for macro syntax.
