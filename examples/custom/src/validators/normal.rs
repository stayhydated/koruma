use std::fmt;

use koruma::{KorumaResult, Validate, validator};

/// A validation rule that checks if a number is within a specified range.
/// Uses `Display` for simple string error messages.
#[validator]
#[derive(Clone, Debug)]
pub struct NumberRangeValidation {
    min: i32,
    max: i32,
    #[koruma(value)]
    pub actual: i32,
}

impl Validate<i32> for NumberRangeValidation {
    fn validate(&self, value: &i32) -> KorumaResult {
        if *value < self.min || *value > self.max {
            Err(())
        } else {
            Ok(())
        }
    }
}

impl fmt::Display for NumberRangeValidation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Value {} must be between {} and {}",
            self.actual, self.min, self.max
        )
    }
}

/// A validation rule that checks string length.
/// Uses `Display` for simple string error messages.
#[validator]
#[derive(Clone, Debug)]
pub struct StringLengthValidation {
    min: usize,
    max: usize,
    #[koruma(value)]
    pub input: String,
}

impl Validate<String> for StringLengthValidation {
    fn validate(&self, value: &String) -> KorumaResult {
        let len = value.len();
        if len < self.min || len > self.max {
            Err(())
        } else {
            Ok(())
        }
    }
}

impl fmt::Display for StringLengthValidation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "String length {} must be between {} and {} characters",
            self.input.len(),
            self.min,
            self.max
        )
    }
}
