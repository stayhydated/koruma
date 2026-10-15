# Build integrations and attribute tooling

Use the runtime contracts to integrate validation into another library, or use
the attribute parser to inspect Koruma syntax without expanding a procedural
macro. Ordinary applications can keep using `koruma` and its default `derive`
feature.

## Choose a package

All five public packages require Rust 1.99 or newer.

| Package | Direct-consumer use |
| --- | --- |
| `koruma` | Application validation, runtime traits, and optional macro re-exports. |
| `koruma-collection` | Built-in rules alongside the facade; see the [validator catalog](koruma_collection.md). |
| `koruma-core` | Runtime traits and error data for handwritten validators or adapters. |
| `koruma-derive` | Procedural macros; applications normally access them through `koruma/derive`. A direct macro dependency still needs the runtime contracts. |
| `koruma-derive-core` | Typed attribute parsers for procedural macros and source-analysis tools. |

## Implement a runtime rule

With a direct dependency on `koruma-core`, no optional features are needed for
`Validate<T>`. The rule borrows its input and returns whether it passes:

```rust
# extern crate koruma_core;
use koruma_core::Validate;

struct MinimumAge(u8);

impl Validate<u8> for MinimumAge {
    fn validate(&self, value: &u8) -> bool {
        *value >= self.0
    }
}

let rule = MinimumAge(13);
assert!(rule.validate(&18));
assert!(!rule.validate(&8));
```

This trait alone does not generate builders, captured failure values, or
metadata. Use [`#[validator]`](declare_validators.md) through the facade when
those generated APIs are needed. Handwritten metadata providers implement
`ValidatorMetadata<T>` separately; metadata does not prove that a type implements
`Validate<T>`.

For a complete value with typed errors, implement `ValidateExt`; its associated
error must implement `ValidationError` and `Default`.
[Nested validation](nested_validation.md) explains how generated parents use
that contract.
Use [structured issues](error_handling.md#enumerate-structured-issues) when an
adapter needs field names and element indices instead of concrete error types.

## Inspect field attributes

Use `koruma-derive-core` with `syn` 3 and its `full` feature when your tool starts
from Rust syntax nodes. No optional Koruma parser features are needed for this
example. It parses one annotated field and confirms that one direct validator
was recorded:

```rust
# extern crate koruma_derive_core;
# extern crate syn;
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

Pass the field's zero-based position as the second argument, including for tuple
fields. Match `ParsedDataField::Unannotated` and `ParsedDataField::Skipped` when
walking all fields; neither participates in validation. For a participating
field, `field_validators()` and `element_validators()` distinguish direct rules
from `each(...)` rules. Nested fields can participate without direct validators.

Parsing checks Koruma syntax and returns `syn::Error` for malformed or
conflicting attributes. It does not resolve names or verify that a validator
implements a Rust trait. Keep one `#[koruma(...)]` attribute per field and combine
its validators and modifiers inside that attribute. Propagate the parser error
so callers can retain its source span.

See the [parser API reference](https://docs.rs/koruma-derive-core/latest/koruma_derive_core/)
for `parse_struct_options`, `parse_validator_struct`, and the typed models used
to inspect other attribute contexts.
