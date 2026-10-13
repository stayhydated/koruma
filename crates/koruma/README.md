# koruma

[![crates.io: koruma][crate-badge]][crate]

`koruma` is the application-facing facade for strongly typed field validation in the
[Koruma project][project]. It re-exports the core validation traits and, by default, the derives
and `#[validator]` attribute.

Requires Rust 1.99 or newer.

Add [`koruma-collection`][koruma-collection] when its built-in validators fit your rules.
The [API reference][api] documents the re-exported runtime contracts and macros.

## Features

The default feature set is `derive`. Runtime traits remain available without it.

| Feature | Default | Purpose |
| --- | --- | --- |
| `default` | Yes | Enables `derive`. |
| `derive` | Yes | Re-exports `Koruma`, `KorumaAllDisplay`, and `#[validator]` from `koruma-derive`. |
| `fluent` | No | Enables `koruma-core/fluent` and forwards `fluent` to an enabled derive dependency; `KorumaAllFluent` also requires `derive`. |
| `internal-showcase` | No | Internal site support: enables `inventory`, `koruma-core/internal-showcase`, and showcase support in an enabled derive dependency. |

## Example

With the default `derive` feature, declare a public custom validator and attach it to a
field. This complete example accepts ages of at least 13 and inspects an invalid
age through the generated accessor.

```rust
use koruma::*;

#[validator]
#[derive(Debug)]
pub struct MinimumAge {
    min: u8,
    actual: u8,
}

impl Validate<u8> for MinimumAge {
    fn validate(&self, value: &u8) -> bool {
        *value >= self.min
    }
}

#[derive(Koruma)]
struct Signup {
    #[koruma(MinimumAge.min(13))]
    age: u8,
}

fn main() {
    match (Signup { age: 8 }).validate() {
        Ok(()) => println!("Signup is valid"),
        Err(errors) => {
            if let Some(failure) = errors.age().minimum_age() {
                assert_eq!(*failure.actual(), 8);
            }
        }
    }
}
```

[crate-badge]: https://img.shields.io/crates/v/koruma.svg?label=koruma
[crate]: https://crates.io/crates/koruma
[project]: https://github.com/stayhydated/koruma
[koruma-collection]: https://crates.io/crates/koruma-collection
[api]: https://docs.rs/koruma/latest/koruma/
