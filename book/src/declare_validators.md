# Declare custom validators

Define a custom validator when a rule is specific to your domain or needs its own error data or
rendering. Annotate a named-field struct with `#[validator]`, store the input that failed, and
implement `Validate<T>` for the input type.

## Define a generic validator

This validator captures the invalid value in `actual` and exposes it through the generated
`actual()` getter:

```rust,ignore
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

## Define a type-specific validator

A validator can target one concrete type:

```rust,ignore
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

The core pattern stays the same: define the configuration fields, keep one inferred or explicit
value field, implement `Validate<T>`, and implement `Display` when callers need display messages.

## Select the captured value field

`#[validator]` first looks for an unmarked private field named `actual`, `input`, or `value`. If no
conventional name exists, it can infer the captured value when exactly one field remains unmarked.
Use `#[koruma(value)]` when multiple unmarked fields remain:

```rust,ignore
#[validator]
pub struct ThresholdValidation {
    minimum: i32,
    #[koruma(value)]
    observed: i32,
}
```

Keep the captured field private. External code can read it through the generated getter, such as
`validator.observed()`.

## Configure generated setters

Unannotated configuration fields generate direct setters. Use bare `#[koruma(setter)]`
when a configuration field is named `actual`, `input`, or `value`, or when marking configuration
fields lets Koruma infer the only remaining unmarked value field. Use `#[koruma(setter(...))]`
when a setter needs options.

| Option | Effect |
| --- | --- |
| `into` | Accepts a value that implements `Into<FieldType>` |
| `required` | Requires an explicit setter call, including an explicit `Some(...)` or `None` for `Option<T>` |
| `name = custom_name` | Generates `custom_name(...)` instead of a setter named after the field |
| `default` | Uses `Default::default()` when the setter is omitted |
| `default = expression` | Uses the expression when the setter is omitted |

```rust,ignore
use koruma::validator;

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

## Skip input capture

For validators that do not need to retain the failing input, use
`#[koruma(skip_capture)]` on an `Option<T>` field. During derived validation, koruma leaves
that field at `None` instead of cloning the input into the error value. If the validator still
needs `Clone` or `Debug`, implement those manually so the skipped field does not reintroduce type
bounds.

## Expose metadata to tooling

`#[validator]` also implements `ValidatorMetadata<T>`. Generic form builders and other tooling can
use the descriptor to list configurable setters and `validator_params()` to inspect configured
runtime values.

Booleans, integers through 64 bits, `isize`/`usize`, floats, `String`/`&str`, and one `Option`
layer around those types use concrete `ValidatorParamValue` variants. Other parameter types are
reported as opaque, so enabling metadata does not add trait bounds to the validator.
