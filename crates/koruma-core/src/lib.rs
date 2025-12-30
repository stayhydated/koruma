/// Result type for validation operations.
///
/// - `Ok(())` indicates validation passed
/// - `Err(())` indicates validation failed (details are in the validator struct)
pub type KorumaResult = Result<(), ()>;

/// Trait for types that can validate a value of type `T`.
///
/// Implementors should return `Ok(())` if validation passes,
/// or `Err(())` if validation fails. The error details are
/// captured in the validation struct itself (via `ToFluentString`).
pub trait Validate<T> {
    fn validate(&self, value: &T) -> KorumaResult;
}

/// Trait for validation error structs that have no errors.
///
/// This is auto-implemented by the derive macro for generated
/// error structs, allowing easy checking if any validation failed.
pub trait ValidationError {
    /// Returns `true` if there are no validation errors.
    fn is_empty(&self) -> bool;

    /// Returns `true` if there are any validation errors.
    fn has_errors(&self) -> bool {
        !self.is_empty()
    }
}

/// Trait for validator builders that can receive the value being validated.
///
/// This is auto-implemented by `#[koruma::validator]` to delegate to the
/// field marked with `#[koruma(value)]`.
pub trait BuilderWithValue<T> {
    fn with_value(self, value: T) -> Self;
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================================
    // Validate Trait Tests
    // ============================================================================

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

    // ============================================================================
    // ValidationError Trait Tests
    // ============================================================================

    struct TestError {
        has_age_error: bool,
        has_name_error: bool,
    }

    impl ValidationError for TestError {
        fn is_empty(&self) -> bool {
            !self.has_age_error && !self.has_name_error
        }
    }

    #[test]
    fn test_validation_error_is_empty() {
        let empty_error = TestError {
            has_age_error: false,
            has_name_error: false,
        };
        assert!(empty_error.is_empty());
        assert!(!empty_error.has_errors());
    }

    #[test]
    fn test_validation_error_has_errors() {
        let error_with_age = TestError {
            has_age_error: true,
            has_name_error: false,
        };
        assert!(!error_with_age.is_empty());
        assert!(error_with_age.has_errors());

        let error_with_both = TestError {
            has_age_error: true,
            has_name_error: true,
        };
        assert!(!error_with_both.is_empty());
        assert!(error_with_both.has_errors());
    }

    // ============================================================================
    // BuilderWithValue Trait Tests
    // ============================================================================

    struct TestBuilder {
        min: i32,
        max: i32,
        value: Option<i32>,
    }

    impl BuilderWithValue<i32> for TestBuilder {
        fn with_value(mut self, value: i32) -> Self {
            self.value = Some(value);
            self
        }
    }

    #[test]
    fn test_builder_with_value() {
        let builder = TestBuilder {
            min: 0,
            max: 100,
            value: None,
        };

        let builder_with_value = builder.with_value(42);
        assert_eq!(builder_with_value.value, Some(42));
    }

    #[test]
    fn test_builder_with_value_chaining() {
        let builder = TestBuilder {
            min: 0,
            max: 100,
            value: None,
        }
        .with_value(99);

        assert_eq!(builder.value, Some(99));
        assert_eq!(builder.min, 0);
        assert_eq!(builder.max, 100);
    }
}
