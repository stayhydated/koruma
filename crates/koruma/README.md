# koruma

[![API docs](https://docs.rs/koruma/badge.svg)](https://docs.rs/koruma/)
[![Crates.io](https://img.shields.io/crates/v/koruma.svg)](https://crates.io/crates/koruma)

`koruma` is the application-facing facade for strongly typed field validation. It re-exports the
core validation traits and, by default, the derives and `#[validator]` attribute.

```toml
[dependencies]
koruma = "0.10"
```

## Features

- `derive` (default): enables `Koruma`, `KorumaAllDisplay`, and `#[validator]`.
- `fluent`: enables `KorumaAllFluent` when used with `derive` and
  [es-fluent](https://github.com/stayhydated/es-fluent).

Add [`koruma-collection`](https://crates.io/crates/koruma-collection) when its built-in validators
fit your rules.

See the [getting-started guide](https://stayhydated.github.io/koruma/book/getting_started.html) for
a runnable example, or use the [API reference](https://docs.rs/koruma/) for trait and macro details.
