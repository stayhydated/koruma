# koruma-derive-core

[![CI](https://github.com/stayhydated/koruma/actions/workflows/ci.yml/badge.svg?branch=master)](https://github.com/stayhydated/koruma/actions/workflows/ci.yml)
[![Codecov](https://codecov.io/gh/stayhydated/koruma/branch/master/graph/badge.svg)](https://codecov.io/gh/stayhydated/koruma)
[![Book](https://img.shields.io/badge/book-online-blue)](https://stayhydated.github.io/koruma/book/)
[![Crates.io](https://img.shields.io/crates/v/koruma-derive-core.svg)](https://crates.io/crates/koruma-derive-core)

`koruma-derive-core` exposes typed parsers for tooling and procedural macros that consume
`#[koruma(...)]` syntax. Its public model covers data-field attributes, struct options,
validator-struct fields, labeled validator chains, target selectors, and setter metadata.

Each supported target accepts one `#[koruma(...)]` attribute; multiple validators and modifiers
are comma-separated inside that attribute.

Application code should depend on [`koruma`](https://crates.io/crates/koruma).
