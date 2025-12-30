#[allow(unused_imports)]
use es_fluent::{EsFluent, ToFluentString};
use koruma::{Koruma, Validate};

/// A validation rule that checks if a number is within a specified range.
#[koruma::validator]
#[derive(Debug, Clone, PartialEq, Eq, Hash, EsFluent)]
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

/// A validation rule that checks string length.
#[koruma::validator]
#[derive(Debug, Clone, PartialEq, Eq, Hash, EsFluent)]
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
#[derive(Koruma)]
pub struct Item {
    #[koruma(NumberRangeValidation(min = 0, max = 100))]
    pub age: i32,

    #[koruma(StringLengthValidation(min = 1, max = 50))]
    pub name: String,

    // This field is not validated
    pub internal_id: u64,
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
}
