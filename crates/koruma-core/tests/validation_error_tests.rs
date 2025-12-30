//! Tests for the ValidationError trait.

use koruma_core::ValidationError;

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
