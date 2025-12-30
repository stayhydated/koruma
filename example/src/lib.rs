#[allow(unused_imports)]
use es_fluent::{EsFluent, ToFluentString};
use koruma::{Koruma, Validate};

/// A validation rule that checks if a number is within a specified range.
#[koruma::validator]
#[derive(Clone, Debug, Eq, EsFluent, Hash, PartialEq)]
pub struct NumberRangeValidation {
    min: i32,
    max: i32,
    #[koruma(value)]
    pub actual: Option<i32>,
}

impl Validate<i32> for NumberRangeValidation {
    fn validate(&self, value: &i32) -> Result<(), ()> {
        if *value < self.min || *value > self.max {
            Err(())
        } else {
            Ok(())
        }
    }
}

// ============================================================================
// Generic Validator Example
// ============================================================================

/// A generic validation rule that checks if a number is within a specified range.
/// Works with any type that implements `PartialOrd + Clone`.
#[koruma::validator]
#[derive(Clone, Debug)]
pub struct GenericRangeValidation<T> {
    pub min: T,
    pub max: T,
    #[koruma(value)]
    pub actual: Option<T>,
}

// Use the auto-generated macro to implement Validate for multiple types at once!
impl_generic_range_validation!(
    i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize, f32, f64
);

/// A validation rule that checks string length.
#[koruma::validator]
#[derive(Clone, Debug, Eq, EsFluent, Hash, PartialEq)]
pub struct StringLengthValidation {
    min: usize,
    max: usize,
    #[koruma(value)]
    pub input: Option<String>,
}

impl Validate<String> for StringLengthValidation {
    fn validate(&self, value: &String) -> Result<(), ()> {
        let len = value.len();
        if len < self.min || len > self.max {
            Err(())
        } else {
            Ok(())
        }
    }
}

/// Example struct demonstrating validation.
/// Example struct demonstrating validation with non-generic validators.
#[derive(Koruma)]
pub struct Item {
    #[koruma(NumberRangeValidation(min = 0, max = 100))]
    pub age: i32,

    #[koruma(StringLengthValidation(min = 1, max = 67))]
    pub name: String,

    // This field is not validated
    pub internal_id: u64,
}

/// Example struct demonstrating validation with generic validators.
/// The type parameter is inferred from the field type using `<_>` syntax!
#[derive(Koruma)]
pub struct GenericItem {
    #[koruma(GenericRangeValidation<_>(min = -10.0, max = 100.0))]
    pub score: f64,

    #[koruma(GenericRangeValidation<_>(min = 0, max = 1000))]
    pub points: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use koruma::ValidationError;

    #[test]
    fn test_valid_item() {
        let item = Item {
            age: 25,
            name: "Alice".to_string(),
            internal_id: 123,
        };

        assert!(item.validate().is_ok());
    }

    #[test]
    fn test_invalid_age_with_value() {
        let item = Item {
            age: 150, // Out of range
            name: "Bob".to_string(),
            internal_id: 456,
        };

        let err = item.validate().unwrap_err();
        assert!(err.age().is_some());
        assert!(err.name().is_none());
        assert!(err.has_errors());

        // The error contains the actual value that failed
        let age_err = err.age().unwrap();
        assert_eq!(age_err.actual, Some(150));

        // Can get the fluent string from the error
        let _fluent_msg = age_err.to_fluent_string();
    }

    #[test]
    fn test_invalid_name_with_value() {
        let item = Item {
            age: 30,
            name: "".to_string(), // Too short
            internal_id: 789,
        };

        let err = item.validate().unwrap_err();
        assert!(err.age().is_none());
        assert!(err.name().is_some());

        // The error contains the actual value that failed
        let name_err = err.name().unwrap();
        assert_eq!(name_err.input, Some("".to_string()));
    }

    #[test]
    fn test_multiple_errors() {
        let item = Item {
            age: -5,              // Out of range
            name: "".to_string(), // Too short
            internal_id: 0,
        };

        let err = item.validate().unwrap_err();
        assert!(err.age().is_some());
        assert!(err.name().is_some());

        // Both errors contain their respective values
        assert_eq!(err.age().unwrap().actual, Some(-5));
        assert_eq!(err.name().unwrap().input, Some("".to_string()));

        // Both errors are collected, not just the first one
        assert!(!err.is_empty());
    }

    #[test]
    fn test_generic_validator_i32() {
        let validator = GenericRangeValidation::<i32>::builder()
            .min(0)
            .max(100)
            .with_value(50)
            .build();

        assert!(validator.validate(&50).is_ok());
        assert!(validator.validate(&150).is_err());
        assert_eq!(validator.actual, Some(50));
    }

    #[test]
    fn test_generic_validator_f64() {
        let validator = GenericRangeValidation::<f64>::builder()
            .min(0.0)
            .max(1.0)
            .with_value(0.5)
            .build();

        assert!(validator.validate(&0.5).is_ok());
        assert!(validator.validate(&1.5).is_err());
        assert_eq!(validator.actual, Some(0.5));
    }

    #[test]
    fn test_generic_item_valid() {
        let item = GenericItem {
            score: 50.0,
            points: 500,
        };

        assert!(item.validate().is_ok());
    }

    #[test]
    fn test_generic_item_invalid_score() {
        let item = GenericItem {
            score: 150.0, // Out of range (max 100.0)
            points: 500,
        };

        let err = item.validate().unwrap_err();
        assert!(err.score().is_some());
        assert!(err.points().is_none());

        // The error contains the actual value
        let score_err = err.score().unwrap();
        assert_eq!(score_err.actual, Some(150.0));
    }

    #[test]
    fn test_generic_item_invalid_points() {
        let item = GenericItem {
            score: 50.0,
            points: 2000, // Out of range (max 1000)
        };

        let err = item.validate().unwrap_err();
        assert!(err.score().is_none());
        assert!(err.points().is_some());

        // The error contains the actual value
        let points_err = err.points().unwrap();
        assert_eq!(points_err.actual, Some(2000));
    }
}
