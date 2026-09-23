# koruma-derive

[![Codecov: koruma-derive][codecov-badge]][codecov]
[![crates.io: koruma-derive][crate-badge]][crate]

`koruma-derive` provides the procedural macros that generate typed validation APIs for the
[Koruma project][project]. Application code normally enables the default `derive` feature on
[`koruma`][koruma] instead of depending on this package directly.

## Overview

The package provides:

- `#[validator]` for declaring configurable validators;
- `#[derive(Koruma)]` for validation and typed error generation;
- `#[derive(KorumaAllDisplay)]` for displayable failed-validator views; and
- `#[derive(KorumaAllFluent)]` with the `fluent` feature for localized views.

[codecov-badge]: https://codecov.io/gh/stayhydated/koruma/branch/master/graph/badge.svg?component=koruma-derive
[codecov]: https://codecov.io/gh/stayhydated/koruma
[crate-badge]: https://img.shields.io/crates/v/koruma-derive.svg?label=koruma-derive
[crate]: https://crates.io/crates/koruma-derive
[project]: https://github.com/stayhydated/koruma
[koruma]: https://crates.io/crates/koruma
