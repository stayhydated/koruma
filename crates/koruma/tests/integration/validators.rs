//! Validator implementations for integration tests.

use koruma::{KorumaResult, Validate, validator};

/// A validation rule that checks if a number is within a specified range.
#[validator]
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
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

/// A generic validation rule that checks if a number is within a specified range.
/// Works with any type that implements `PartialOrd + Clone`.
#[validator]
#[derive(Clone, Debug)]
pub struct GenericRangeValidation<T> {
    pub min: T,
    pub max: T,
    #[koruma(value)]
    pub actual: T,
}

// Use the auto-generated macro to implement Validate for multiple types at once!
impl_generic_range_validation!(
    i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize, f32, f64
    => |this, value| *value >= this.min && *value <= this.max
);

/// A validation rule that checks string length.
#[validator]
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
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

/// A validation rule that checks if a number is even.
#[validator]
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct EvenNumberValidation {
    #[koruma(value)]
    pub actual: i32,
}

impl Validate<i32> for EvenNumberValidation {
    fn validate(&self, value: &i32) -> KorumaResult {
        if value % 2 != 0 { Err(()) } else { Ok(()) }
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
    pub actual: Vec<T>,
}

impl<T> Validate<Vec<T>> for VecLenValidation<T> {
    fn validate(&self, value: &Vec<T>) -> KorumaResult {
        let len = value.len();
        if len < self.min || len > self.max {
            Err(())
        } else {
            Ok(())
        }
    }
}

impl<T> VecLenValidation<T> {
    /// Get the actual length of the Vec for error reporting.
    pub fn actual_len(&self) -> usize {
        self.actual.len()
    }
}
