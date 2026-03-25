# Introduction

`koruma` is a Rust validation framework built around derive macros and explicit validator types.
Instead of hiding validation logic behind opaque attributes, it lets you define reusable validator
structs, attach them to fields, and derive strongly typed error accessors.

This book is a practical guide to the workflow shown in the project README:

- define validators with `#[validator]`,
- attach them with `#[koruma(...)]`,
- derive `Koruma` on your data type,
- and inspect failures through generated typed error accessors.

The next chapter walks through the basic setup and the overall flow before going deeper into
custom validators, error handling, localisation, and newtypes.
