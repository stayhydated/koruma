# koruma

[![CI](https://github.com/stayhydated/koruma/actions/workflows/ci.yml/badge.svg?branch=master)](https://github.com/stayhydated/koruma/actions/workflows/ci.yml)
[![Codecov](https://codecov.io/gh/stayhydated/koruma/branch/master/graph/badge.svg)](https://codecov.io/gh/stayhydated/koruma)
[![Book](https://img.shields.io/badge/book-online-blue)](https://stayhydated.github.io/koruma/book/)
[![Crates.io](https://img.shields.io/crates/v/koruma.svg)](https://crates.io/crates/koruma)

`koruma` is the application-facing facade for strongly typed field validation. It re-exports the
core validation traits and, by default, the derives and `#[validator]` attribute.

## Features

- `derive` (default): enables `Koruma`, `KorumaAllDisplay`, and `#[validator]`.
- `fluent`: enables `KorumaAllFluent` when used with `derive` and
  [es-fluent](https://github.com/stayhydated/es-fluent).

Add [`koruma-collection`](https://crates.io/crates/koruma-collection) when its built-in validators
fit your rules.
