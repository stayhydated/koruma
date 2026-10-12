# koruma-derive

[![Codecov: koruma-derive][codecov-badge]][codecov]
[![crates.io: koruma-derive][crate-badge]][crate]

`koruma-derive` provides the procedural macros that generate typed validation APIs for the
[Koruma project][project]. Application code normally enables the default `derive` feature on
[`koruma`][koruma] instead of depending on this package directly.

Requires Rust 1.99 or newer. These macros generate code using Koruma's runtime
contracts; a direct macro dependency alone does not provide those contracts.

## Overview

The package provides:

- `#[validator]` for declaring configurable validators;
- `#[derive(Koruma)]` for validation and typed error generation;
- `#[derive(KorumaAllDisplay)]` for displayable failed-validator views; and
- `#[derive(KorumaAllFluent)]` with the `fluent` feature for localized views.

The [macro API reference][api] owns the attribute syntax and generated API
contracts. Use the facade's [complete custom-validator example][facade-example]
for application usage.

## Features

No features are enabled by default.

| Feature | Default | Purpose |
| --- | --- | --- |
| `fluent` | No | Provides `KorumaAllFluent` and emits localized failure views; applications normally select `koruma/fluent` together with `koruma/derive`. |
| `internal-showcase` | No | Internal site support: enables showcase generation and `koruma-derive-core/internal-showcase` parsers. |

[codecov-badge]: https://codecov.io/gh/stayhydated/koruma/branch/master/graph/badge.svg?component=koruma-derive
[codecov]: https://codecov.io/gh/stayhydated/koruma
[crate-badge]: https://img.shields.io/crates/v/koruma-derive.svg?label=koruma-derive
[crate]: https://crates.io/crates/koruma-derive
[project]: https://github.com/stayhydated/koruma
[koruma]: https://crates.io/crates/koruma
[api]: https://docs.rs/koruma-derive/latest/koruma_derive/
[facade-example]: https://github.com/stayhydated/koruma/blob/master/crates/koruma/README.md#example
