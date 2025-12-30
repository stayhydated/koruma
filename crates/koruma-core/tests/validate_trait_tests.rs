//! Tests for the Validate trait.

use koruma_core::{KorumaResult, Validate};

struct RangeValidator {
    min: i32,
    max: i32,
}

impl Validate<i32> for RangeValidator {
    fn validate(&self, value: &i32) -> KorumaResult {
        if *value >= self.min && *value <= self.max {
            Ok(())
        } else {
            Err(())
        }
    }
}

#[test]
fn test_validate_passes_for_valid_value() {
    let validator = RangeValidator { min: 0, max: 100 };
    assert!(validator.validate(&50).is_ok());
    assert!(validator.validate(&0).is_ok());
    assert!(validator.validate(&100).is_ok());
}

#[test]
fn test_validate_fails_for_invalid_value() {
    let validator = RangeValidator { min: 0, max: 100 };
    assert!(validator.validate(&-1).is_err());
    assert!(validator.validate(&101).is_err());
}

// Generic validator test
struct GenericLengthValidator<T> {
    min_len: usize,
    _marker: std::marker::PhantomData<T>,
}

impl<T: AsRef<str>> Validate<T> for GenericLengthValidator<T> {
    fn validate(&self, value: &T) -> KorumaResult {
        if value.as_ref().len() >= self.min_len {
            Ok(())
        } else {
            Err(())
        }
    }
}

#[test]
fn test_generic_validate_trait() {
    let validator = GenericLengthValidator::<String> {
        min_len: 3,
        _marker: std::marker::PhantomData,
    };

    assert!(validator.validate(&"hello".to_string()).is_ok());
    assert!(validator.validate(&"hi".to_string()).is_err());
}
