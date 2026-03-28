# koruma

> koruma is a per-field validation framework for Rust focused on type safety, ergonomics, and developer experience. It generates strongly typed validation error structs at compile time using derive macros.

Key features:

- Define reusable validator structs with `#[validator]`
- Attach validators to fields with `#[koruma(...)]`
- Derive `Koruma` on data types for typed error accessors
- Optional i18n support via Project Fluent and `es-fluent`
- Nested validation and newtype pattern support
- Built-in validator collection via `koruma-collection` crate
