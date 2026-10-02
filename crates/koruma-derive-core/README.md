# koruma-derive-core

[![Codecov: koruma-derive-core][codecov-badge]][codecov]
[![crates.io: koruma-derive-core][crate-badge]][crate]

`koruma-derive-core` exposes typed parsers for tooling and procedural macros that consume
`#[koruma(...)]` syntax in the [Koruma project][project]. Application code normally uses
[`koruma`][koruma] instead.

## Overview

Its public model covers data-field attributes, struct options, validator-struct fields, labeled
validator chains, target selectors, and setter metadata. Each supported target accepts one
`#[koruma(...)]` attribute; multiple validators and modifiers are comma-separated inside it.

Validator field models preserve reference types so `koruma-derive` can report borrowed scalar
metadata values, including nested references and optional borrowed scalars.

[codecov-badge]: https://codecov.io/gh/stayhydated/koruma/branch/master/graph/badge.svg?component=koruma-derive-core
[codecov]: https://codecov.io/gh/stayhydated/koruma
[crate-badge]: https://img.shields.io/crates/v/koruma-derive-core.svg?label=koruma-derive-core
[crate]: https://crates.io/crates/koruma-derive-core
[project]: https://github.com/stayhydated/koruma
[koruma]: https://crates.io/crates/koruma
