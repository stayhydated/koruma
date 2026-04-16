# Declaring Validators

Validators are regular Rust types that describe a validation rule. To define one, annotate the
struct with `#[validator]` and implement `Validate<T>` for the input type you want to check. The
`#[koruma(value)]` attribute marks the field that stores the actual input value used for error
reporting, and `#[validator]` generates a getter with the same name. Keep that field private and
use the generated getter for external reads.

For example, a generic range validator:

```rust
use koruma::{Validate, validator};
use std::fmt;

#[validator]
#[derive(Clone, Debug)]
pub struct NumberRangeValidation<T: PartialOrd + Copy + fmt::Display + Clone> {
    min: T,
    max: T,
    #[koruma(value)]
    actual: T,
}

impl<T: PartialOrd + Copy + fmt::Display> Validate<T> for NumberRangeValidation<T> {
    fn validate(&self, value: &T) -> bool {
        *value >= self.min && *value <= self.max
    }
}

impl<T: PartialOrd + Copy + fmt::Display + Clone> fmt::Display for NumberRangeValidation<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "value {} must be between {} and {}",
            self.actual, self.min, self.max
        )
    }
}
```

You can also write type-specific validators. For example, a validator for string length:

```rust
use koruma::{Validate, validator};
use std::fmt;

#[validator]
#[derive(Clone, Debug)]
pub struct StringLengthValidation {
    min: usize,
    max: usize,
    #[koruma(value)]
    input: String,
}

impl Validate<String> for StringLengthValidation {
    fn validate(&self, value: &String) -> bool {
        let len = value.chars().count();
        len >= self.min && len <= self.max
    }
}

impl fmt::Display for StringLengthValidation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "string length {} must be between {} and {} characters",
            self.input.chars().count(),
            self.min,
            self.max
        )
    }
}
```

The core pattern stays the same: define any configuration fields you need, mark the validated value
with `#[koruma(value)]`, implement `Validate<T>`, and optionally implement `Display` for friendly
error messages. External callers can read the captured value through the generated getter
(`validator.actual()`, `validator.input()`, and so on).

For presence-only validators that do not need to retain the failing input, use
`#[koruma(value, skip_capture)]` on an `Option<T>` field. During derived validation, koruma leaves
that field at `None` instead of cloning the input into the error value. If the validator still
needs `Clone` or `Debug`, implement those manually so the skipped field does not reintroduce type
bounds.
