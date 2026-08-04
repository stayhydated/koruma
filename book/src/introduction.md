# Introduction

Koruma lets Rust applications attach reusable validators to struct fields and inspect failures
through generated, strongly typed error accessors. Use `koruma` for the derive macros and core
traits, and add `koruma-collection` when its built-in validators fit your rules.

This book is for application developers who are comfortable defining Rust structs and adding Cargo
dependencies. It covers the complete consumer workflow:

- get a working validation result with built-in validators;
- define domain-specific validators with `#[validator]`;
- attach field, optional-value, and per-element rules with `#[koruma(...)]`;
- render or enumerate generated errors;
- validate nested structs and newtype wrappers; and
- localize messages with an application-owned es-fluent localizer.

Start with [Get started](getting_started.md) for a runnable path. Use the later chapters as focused
guides and references, and see [Troubleshooting](troubleshooting.md) when a derive or validator
configuration does not compile.
