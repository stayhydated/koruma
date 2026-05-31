use std::fmt;

use koruma::{Validate, validator};

/// A validation rule that checks if a number is within a specified range.
/// Uses `Display` for simple string error messages.
#[validator]
#[derive(Clone, Debug)]
pub struct NumberRangeValidation<T: PartialOrd + std::fmt::Display + Clone> {
    min: T,
    max: T,
    #[koruma(value)]
    actual: T,
}

impl<T: PartialOrd + std::fmt::Display + Clone> Validate<T> for NumberRangeValidation<T> {
    fn validate(&self, value: &T) -> bool {
        value >= &self.min && value <= &self.max
    }
}

impl<T: PartialOrd + std::fmt::Display + Clone> std::fmt::Display for NumberRangeValidation<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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

/// A validation rule that checks if a string matches an expected value.
/// Uses `Display` for simple string error messages.
#[validator]
#[derive(Clone, Debug)]
pub struct ZipCodeValidation {
    #[koruma(value)]
    input: String,
}

impl Validate<String> for ZipCodeValidation {
    fn validate(&self, value: &String) -> bool {
        // Simple validation: 5 digits
        value.len() == 5 && value.chars().all(|c| c.is_ascii_digit())
    }
}

impl fmt::Display for ZipCodeValidation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Zip code '{}' must be exactly 5 digits", self.input)
    }
}

/// A validation rule that checks whether a string starts with a prefix.
#[validator]
#[derive(Clone, Debug)]
pub struct StartsWithValidation<T: AsRef<str>> {
    prefix: &'static str,
    #[koruma(value)]
    actual: T,
}

impl<T: AsRef<str>> Validate<T> for StartsWithValidation<T> {
    fn validate(&self, value: &T) -> bool {
        value.as_ref().starts_with(self.prefix)
    }
}

impl<T: AsRef<str>> fmt::Display for StartsWithValidation<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Must start with '{}'", self.prefix)
    }
}
