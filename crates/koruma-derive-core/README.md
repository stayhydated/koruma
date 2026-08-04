# koruma-derive-core

[![API docs](https://docs.rs/koruma-derive-core/badge.svg)](https://docs.rs/koruma-derive-core/)
[![Crates.io](https://img.shields.io/crates/v/koruma-derive-core.svg)](https://crates.io/crates/koruma-derive-core)

`koruma-derive-core` exposes typed parsers for tooling and procedural macros that consume
`#[koruma(...)]` syntax. Its public model covers data-field attributes, struct options,
validator-struct fields, labeled validator chains, target selectors, and setter metadata.

Each supported target accepts one `#[koruma(...)]` attribute; multiple validators and modifiers
are comma-separated inside that attribute.

Application code should depend on [`koruma`](https://crates.io/crates/koruma). Macro and tooling
authors can use the [API reference](https://docs.rs/koruma-derive-core/) for parser contracts.
