# Troubleshooting

Koruma reports most configuration problems as compile errors at the validator attribute or derive.
Match the symptom below, change the attribute or trait implementation, and rerun `cargo check`.

| Symptom | Likely cause | Action |
| --- | --- | --- |
| A validator does not implement `Validate<T>` | The validator targets the wrong field type, often `T` versus `Option<T>` | Use the default unwrapped target for present optional values, or use `Validator::<Option<_>>`/`full(...)` when the validator must receive the whole option |
| A generated builder does not implement `BuildValidator` | A required validator setter was omitted | Call every non-defaulted setter, including fields marked `#[koruma(setter(required))]` |
| A setter expects exactly one argument | The setter call has zero or multiple arguments | Pass one expression to each setter and chain another setter for the next value |
| A setter does not accept generic arguments | Type arguments were placed on the method | Write `Validator::<_>.setter(value)`, not `Validator.setter::<_>(value)` |
| `each(...)` rejects the field type | The field is not a syntactic `Vec<T>`, slice, array, or optional variant | Use a supported collection spelling or validate the custom collection separately |
| A duplicate getter or enum variant is generated | Two validators have the same final type name | Add lower-snake labels such as `minimum = RangeValidation::<_>.min(0)` |
| `KorumaAllDisplay` reports a missing `Display` implementation | A stored validator cannot render through `Display` | Implement `Display` for every custom validator on that type, or remove `KorumaAllDisplay` if `all()` values do not need display rendering |
| `KorumaAllFluent` reports a missing `FluentMessage` implementation | A stored validator is not configured for Fluent | Derive or implement `FluentMessage` for every validator and enable `koruma`'s `fluent` feature |
| `KorumaAllFluent` reports that an `each(...)` field error lacks `FluentMessage` | The field has element validators but no direct field validator | Omit `KorumaAllFluent` and localize the concrete validator failures returned by `element_errors()`, or add a meaningful direct field rule when aggregate Fluent rendering is also needed |
| A struct-level newtype's field does not implement `NewtypeValidation` | The newtype field is unannotated, so Koruma delegates validation to the inner type | Attach a validator or `nested`/`newtype` marker to the field, or wrap a type that implements `NewtypeValidation` |
| `#[koruma(skip_capture)]` rejects a field | The captured field is not `Option<T>` | Change the captured field to `Option<T>`, or retain normal value capture |

## An optional field unexpectedly accepts `None`

Optional fields unwrap by default: Koruma skips `None` and validates only `Some(T)`. To require a
value, validate the full option:

```rust,ignore
#[koruma(koruma_collection::general::RequiredValidation::<Option<_>>)]
display_name: Option<String>,
```

Use `full(Validator::<_>)` for another validator that implements `Validate<Option<T>>`.

## Per-element errors do not appear in the field validator accessors

`each(...)` failures are stored separately from validators on the collection itself. Inspect
`element_errors()` and then use the generated validator accessor on each element error:

```rust,ignore
for (index, element_error) in errors.quantities().element_errors() {
    if let Some(error) = element_error.range_validation() {
        println!("quantities[{index}]: {error}");
    }
}
```

If the field also has a direct collection validator, inspect it through
its generated accessor, such as `len_validation()`, or through `errors.quantities().all()`.
