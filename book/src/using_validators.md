# Using Validators

Once validators are defined, attach them to fields with `#[koruma(...)]` and derive `Koruma` on
your data type. The derive macro generates a `validate()` method that runs the configured
validators and returns `Result<(), Errors>`.

```rust
use koruma::{Koruma, KorumaAllDisplay};

#[derive(Koruma, KorumaAllDisplay)]
pub struct Item {
    #[koruma(NumberRangeValidation<_>(min = 0, max = 100))]
    pub age: i32,

    #[koruma(StringLengthValidation(min = 1, max = 67))]
    pub name: String,

    // Fields without #[koruma(...)] are ignored by validation.
    pub internal_id: u64,
}

let item = Item {
    age: 150,
    name: "".to_string(),
    internal_id: 1,
};

match item.validate() {
    Ok(()) => println!("Item is valid"),
    Err(errors) => {
        if let Some(age_err) = errors.age().number_range_validation() {
            println!("age failed: {}", age_err);
        }
        if let Some(name_err) = errors.name().string_length_validation() {
            println!("name failed: {}", name_err);
        }
    }
}
```

Use `TypeName<_>(...)` when the validator is generic and Rust can infer the missing type
parameter. If a field has no `#[koruma(...)]` attribute, `koruma` does not validate it.

For per-element validation, `each(...)` supports `Vec<T>`, borrowed slices like
`&[T]`, arrays like `[T; N]`, and optional variants of those:

```rust
#[derive(Koruma)]
pub struct Order {
    #[koruma(each(NumberRangeValidation<_>(min = 1, max = 5)))]
    pub line_item_scores: Vec<i32>,
}
```

Borrowed fields work too when the validator accepts the borrowed item type:

```rust
use koruma::Koruma;
use koruma_collection::string::PatternValidation;

#[derive(Koruma)]
pub struct BorrowedUser<'a> {
    #[koruma(PatternValidation<_>(pattern = r"^user:[a-z]+$"))]
    pub username: &'a str,
}
```

For fields with more than one validator, `koruma` generates accessors for each validator as well as
an `all()` iterator when you derive `KorumaAllDisplay` or `KorumaAllFluent`. The next chapter
covers that multi-validator case in more detail.
