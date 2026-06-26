# Multiple Validators & Error Handling

A field can have more than one validator. After deriving `Koruma`, calling `validate()` runs all
configured validators and returns `Result<(), Errors>`. On failure, the generated `Errors` type
provides strongly typed field accessors, and each field error group exposes accessors for the
individual validators that failed.

```rust
if let Err(errors) = item.validate() {
    for failed in errors.age().all() {
        println!("age validator: {}", failed);
    }

    for failed in errors.name().all() {
        println!("name validator: {}", failed);
    }
}
```

This pattern is useful when a field has multiple rules and you want to show every failure instead of
only the first one. The order of execution follows the order in which validators are listed in the
`#[koruma(...)]` attribute, and all configured validators are evaluated. `all()` returns borrowed
failed-validator values, so inspecting every failure does not require the validator types to
implement `Clone`.

For tooling that wants a generic, field-scoped view instead of strongly typed
accessors, generated aggregate error structs implement `ValidationIssues`.
`errors.issues()` returns structured field and element issues with field names,
available as typed `ValidationFieldName` values, validator type names, labels,
element indices, and human-readable messages.

You can customise error rendering by implementing `Display` for validators, or localise errors with
[es-fluent](https://github.com/stayhydated/es-fluent) and `KorumaAllFluent`, which is covered in the next chapter.
