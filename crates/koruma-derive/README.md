# koruma-derive

[![CI](https://github.com/stayhydated/koruma/actions/workflows/ci.yml/badge.svg?branch=master)](https://github.com/stayhydated/koruma/actions/workflows/ci.yml)
[![Codecov](https://codecov.io/gh/stayhydated/koruma/branch/master/graph/badge.svg)](https://codecov.io/gh/stayhydated/koruma)
[![Book](https://img.shields.io/badge/book-online-blue)](https://stayhydated.github.io/koruma/book/)
[![Crates.io](https://img.shields.io/crates/v/koruma-derive.svg)](https://crates.io/crates/koruma-derive)

`koruma-derive` provides Koruma's procedural macros. Application code should normally enable the
default `derive` feature on [`koruma`](https://crates.io/crates/koruma) instead of depending on
this crate directly.

The crate provides:

- `#[validator]` for declaring configurable validators;
- `#[derive(Koruma)]` for validation and typed error generation;
- `#[derive(KorumaAllDisplay)]` for displayable failed-validator views; and
- `#[derive(KorumaAllFluent)]` with the `fluent` feature for localized views.
