# koruma-derive-core

[![Codecov: koruma-derive-core][codecov-badge]][codecov]
[![crates.io: koruma-derive-core][crate-badge]][crate]

`koruma-derive-core` exposes typed parsers for tooling and procedural macros that consume
`#[koruma(...)]` syntax in the [Koruma project][project]. Application code normally uses
[`koruma`][koruma] instead.

Requires Rust 1.99 or newer. Direct consumers work with `syn` 3 syntax nodes.

## Overview

Its public model covers data-field attributes, struct options, validator-struct fields, labeled
validator chains, target selectors, and setter metadata. Each supported target accepts one
`#[koruma(...)]` attribute; multiple validators and modifiers are comma-separated inside it.

The [API reference][api] owns the parser contracts and typed models.

## Features

No features are enabled by default.

| Feature | Default | Purpose |
| --- | --- | --- |
| `internal-showcase` | No | Internal site support: exposes showcase attribute parsers, input kinds, and module metadata. |

## Example

With no optional features, inspect a `syn::Field` without invoking a procedural
macro. This complete example propagates parse errors and confirms that the
annotated field contains one validator. Parsing records syntax; it does not
resolve the validator name to a Rust type.

```rust
use koruma_derive_core::{ParsedDataField, parse_field};
use syn::{Field, parse_quote};

fn main() -> syn::Result<()> {
    let field: Field = parse_quote! {
        #[koruma(NonEmptyValidation)]
        username: String
    };

    assert!(matches!(
        parse_field(&field, 0)?,
        ParsedDataField::Participating(info) if info.field_validators().len() == 1
    ));
    Ok(())
}
```

[codecov-badge]: https://codecov.io/gh/stayhydated/koruma/branch/master/graph/badge.svg?component=koruma-derive-core
[codecov]: https://codecov.io/gh/stayhydated/koruma
[crate-badge]: https://img.shields.io/crates/v/koruma-derive-core.svg?label=koruma-derive-core
[crate]: https://crates.io/crates/koruma-derive-core
[project]: https://github.com/stayhydated/koruma
[koruma]: https://crates.io/crates/koruma
[api]: https://docs.rs/koruma-derive-core/latest/koruma_derive_core/
