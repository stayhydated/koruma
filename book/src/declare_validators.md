# Declaring Validators

Validators are regular Rust types that describe a validation rule. To define one, annotate the
struct with `#[validator]` and implement `Validate<T>` for the input type you want to check. The
`#[validator]` macro infers an unmarked private field named `actual`, `input`, or `value` as
the captured input value used for error reporting, and generates a getter with the same name. If no
conventional name exists, it can infer the value field when exactly one field is unmarked. Use
`#[koruma(value)]` when multiple unmarked fields remain. Keep that field private and use the
generated getter for external reads.

For example, a generic range validator:

```rust
use koruma::{Validate, validator};
use std::fmt;

#[validator]
#[derive(Clone, Debug)]
pub struct NumberRangeValidation<T: PartialOrd + fmt::Display + Clone> {
    min: T,
    max: T,
    actual: T,
}

impl<T: PartialOrd + fmt::Display + Clone> Validate<T> for NumberRangeValidation<T> {
    fn validate(&self, value: &T) -> bool {
        value >= &self.min && value <= &self.max
    }
}

impl<T: PartialOrd + fmt::Display + Clone> fmt::Display for NumberRangeValidation<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Value {} must be between {} and {}",
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
            "String length {} must be between {} and {} characters",
            self.input.chars().count(),
            self.min,
            self.max
        )
    }
}
```

The core pattern stays the same: define any configuration fields you need, keep one inferred or
explicit value field, implement `Validate<T>`, and optionally implement `Display` for friendly
error messages. External callers can read the captured value through the generated getter
(`validator.actual()`, `validator.input()`, and so on).

`#[validator]` also emits `ValidatorMetadata<T>`. The descriptor reports the
validator type and configurable setter fields, while `validator_params()` reads
runtime parameter values from a validator instance. Bool, numeric, string, and
optional values are represented directly; generic or otherwise unconstrained
values are reported as opaque so metadata does not add trait bounds to the
validator.

Unannotated configuration fields generate direct setters. Use bare `#[koruma(setter)]`
when a configuration field is named `actual`, `input`, or `value`, or when marking configuration
fields lets Koruma infer the only remaining unmarked value field. Use `#[koruma(setter(...))]`
when a setter needs options.
Supported setter options are `into`, `required`, `name`, and `default`:

```rust
#[validator]
pub struct PrefixValidation<T> {
    #[koruma(setter(into))]
    prefix: String,
    #[koruma(skip_capture)]
    actual: Option<T>,
}
```

Setter names that collide with generated builder APIs are rejected, including `new`, `build`,
`with_value`, `builder`, `__koruma_builder`, `build_validator`, `capture_value_ref`, and the
generated `maybe_` prefix.

For optional non-required configuration fields, `Option<T>` setters take `T` directly and wrap it
in `Some(...)`. Use the generated `maybe_*` setter when you already have an `Option<T>`. Mark an
`Option<T>` setter as `required` when `None` is a meaningful explicit configuration value.

For validators that do not need to retain the failing input, use
`#[koruma(skip_capture)]` on an `Option<T>` field. During derived validation, koruma leaves
that field at `None` instead of cloning the input into the error value. If the validator still
needs `Clone` or `Debug`, implement those manually so the skipped field does not reintroduce type
bounds.
