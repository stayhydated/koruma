//! Tests for the Validate trait.

use koruma_core::Validate;

struct RangeValidator {
    min: i32,
    max: i32,
}

impl Validate<i32> for RangeValidator {
    fn validate(&self, value: &i32) -> bool {
        if *value >= self.min && *value <= self.max {
            true
        } else {
            false
        }
    }
}

#[test]
fn test_validate_passes_for_valid_value() {
    let validator = RangeValidator { min: 0, max: 100 };
    assert!(validator.validate(&50));
    assert!(validator.validate(&0));
    assert!(validator.validate(&100));
}

#[test]
fn test_validate_fails_for_invalid_value() {
    let validator = RangeValidator { min: 0, max: 100 };
    assert!(!validator.validate(&-1));
    assert!(!validator.validate(&101));
}

// Generic validator test
struct GenericLengthValidator<T> {
    min_len: usize,
    _marker: std::marker::PhantomData<T>,
}

impl<T: AsRef<str>> Validate<T> for GenericLengthValidator<T> {
    fn validate(&self, value: &T) -> bool {
        if value.as_ref().len() >= self.min_len {
            true
        } else {
            false
        }
    }
}

#[test]
fn test_generic_validate_trait() {
    let validator = GenericLengthValidator::<String> {
        min_len: 3,
        _marker: std::marker::PhantomData,
    };

    assert!(validator.validate(&"hello".to_string()));
    assert!(!validator.validate(&"hi".to_string()));
}
