# koruma

[![crates.io: koruma][crate-badge]][crate]

`koruma` is the application-facing facade for strongly typed field validation in the
[Koruma project][project]. It re-exports the core validation traits and, by default, the derives
and `#[validator]` attribute.

## Overview

The default `derive` feature enables `Koruma`, `KorumaAllDisplay`, and `#[validator]`. The
`fluent` feature adds `KorumaAllFluent` when used with `derive` and
[es-fluent][es-fluent]. Add [`koruma-collection`][koruma-collection] when its built-in validators
fit your rules.

[crate-badge]: https://img.shields.io/crates/v/koruma.svg?label=koruma
[crate]: https://crates.io/crates/koruma
[project]: https://github.com/stayhydated/koruma
[es-fluent]: https://github.com/stayhydated/es-fluent
[koruma-collection]: https://crates.io/crates/koruma-collection
