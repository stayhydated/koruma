//! Validator implementations for integration tests.

use koruma::{Validate, validator};
use std::fmt;

/// A validation rule that checks if a number is within a specified range.
#[validator]
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct NumberRangeValidation {
    min: i32,
    max: i32,
    #[koruma(value)]
    actual: i32,
}

impl Validate<i32> for NumberRangeValidation {
    fn validate(&self, value: &i32) -> bool {
        !(*value < self.min || *value > self.max)
    }
}

/// A generic validation rule that checks if a number is within a specified range.
/// Works with any type that implements `PartialOrd + Clone`.
#[validator]
#[derive(Clone, Debug)]
pub struct GenericRangeValidation<T> {
    pub min: T,
    pub max: T,
    #[koruma(value)]
    actual: T,
}

// Use a blanket impl with trait bounds instead of a macro
impl<T: PartialOrd + Clone> Validate<T> for GenericRangeValidation<T> {
    fn validate(&self, value: &T) -> bool {
        *value >= self.min && *value <= self.max
    }
}

/// A validation rule that checks whether a fixed-size byte array starts with a prefix.
/// This exercises validator builders that carry both lifetime and const generics.
#[validator]
#[derive(Clone, Debug)]
pub struct PrefixBytesValidation<'a, const N: usize> {
    pub prefix: &'a [u8],
    #[koruma(value)]
    actual: [u8; N],
}

impl<'a, const N: usize> Validate<[u8; N]> for PrefixBytesValidation<'a, N> {
    fn validate(&self, value: &[u8; N]) -> bool {
        value.starts_with(self.prefix)
    }
}

/// A validation rule that checks string length.
#[validator]
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct StringLengthValidation {
    min: usize,
    max: usize,
    #[koruma(value)]
    input: String,
}

impl Validate<String> for StringLengthValidation {
    fn validate(&self, value: &String) -> bool {
        let len = value.chars().count();
        !(len < self.min || len > self.max)
    }
}

/// A validation rule that checks if a number is even.
#[validator]
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct EvenNumberValidation {
    #[koruma(value)]
    actual: i32,
}

impl Validate<i32> for EvenNumberValidation {
    fn validate(&self, value: &i32) -> bool {
        value % 2 == 0
    }
}

/// A validation rule that intentionally does not implement Clone.
#[validator]
#[derive(Debug, Eq, Hash, PartialEq)]
pub struct NonCloneRangeValidation {
    min: i32,
    max: i32,
    #[koruma(value)]
    actual: i32,
}

impl Validate<i32> for NonCloneRangeValidation {
    fn validate(&self, value: &i32) -> bool {
        !(*value < self.min || *value > self.max)
    }
}

impl fmt::Display for NonCloneRangeValidation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "value must be between {} and {}, got {}",
            self.min, self.max, self.actual
        )
    }
}

/// A validation rule that checks if a Vec has length within a specified range.
/// This demonstrates collection-level validation (as opposed to per-element validation).
///
/// Note: Since the Koruma macro passes the full field value to `with_value`, the `actual`
/// field stores the complete Vec. The `actual_len()` method provides convenient access
/// to the length for error messages.
#[validator]
#[derive(Clone, Debug)]
pub struct VecLenValidation<T> {
    pub min: usize,
    pub max: usize,
    /// The Vec being validated
    #[koruma(value)]
    actual: Vec<T>,
}

impl<T> Validate<Vec<T>> for VecLenValidation<T> {
    fn validate(&self, value: &Vec<T>) -> bool {
        let len = value.len();
        !(len < self.min || len > self.max)
    }
}

impl<T> VecLenValidation<T> {
    /// Get the actual length of the Vec for error reporting.
    pub fn actual_len(&self) -> usize {
        self.actual.len()
    }
}

/// A validation rule that checks if a value is present (not None).
/// Works with Option<T> types.
#[validator]
pub struct RequiredValidation<T> {
    #[koruma(value, skip_capture)]
    #[allow(dead_code)]
    actual: Option<T>,
}

impl<T> Clone for RequiredValidation<T> {
    fn clone(&self) -> Self {
        Self { actual: None }
    }
}

impl<T> fmt::Debug for RequiredValidation<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RequiredValidation")
            .field("actual", &"<skipped>")
            .finish()
    }
}

impl<T> Validate<Option<T>> for RequiredValidation<Option<T>> {
    fn validate(&self, value: &Option<T>) -> bool {
        value.is_some()
    }
}

/// A validation rule that checks if a string matches an expected string.
#[validator]
#[derive(Clone, Debug)]
pub struct MatchesStringValidation {
    pub expected: String,
    #[koruma(value)]
    actual: String,
}

impl Validate<String> for MatchesStringValidation {
    fn validate(&self, value: &String) -> bool {
        value == &self.expected
    }
}

/// A validation rule that checks if a string matches a shared static secret.
#[validator]
#[derive(Clone, Debug)]
pub struct MatchesStaticStrValidation {
    pub expected: &'static str,
    #[koruma(value)]
    actual: String,
}

impl Validate<String> for MatchesStaticStrValidation {
    fn validate(&self, value: &String) -> bool {
        value == self.expected
    }
}

/// A validation rule that checks whether a string starts with a prefix.
#[validator]
#[derive(Clone, Debug)]
pub struct StartsWithValidation<T: AsRef<str>> {
    pub prefix: &'static str,
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
