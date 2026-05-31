// Test case demonstrating the issue with external macro wrapping
// This simulates what happens when gpui_form wraps a non-optional field

use koruma::{Koruma, Validate};

use super::validators::{NumberRangeValidation, RequiredValidation};

/// A newtype that would be used in a form
#[derive(Clone, Debug, Koruma)]
#[koruma(newtype)]
pub struct Age {
    #[koruma(NumberRangeValidation::min(0).max(150))]
    pub value: i32,
}

/// This simulates what gpui_form produces - the field is explicitly Option<T>
/// This works correctly in Koruma today
#[derive(Koruma)]
pub struct ExplicitOptionForm {
    #[koruma(newtype, RequiredValidation::<_>)]
    pub age: Option<Age>,
}

/// This simulates the ORIGINAL struct before gpui_form transforms it
/// The field is non-optional `Age`, but gpui_form would wrap it to `Option<Age>`
/// When Koruma runs on the transformed struct, it sees `Option<Age>` and works correctly
///
/// BUT if Koruma runs BEFORE the transformation (on the original struct),
/// it sees `Age` (non-optional) and generates wrong code
#[derive(Koruma)]
pub struct OriginalStructBeforeTransformation {
    // NOTE: This is NOT marked as optional, but gpui_form would make it Option<Age>
    // If Koruma expands on this, it treats age as non-optional
    #[koruma(newtype)] // Missing RequiredValidation because gpui_form adds it
    pub age: Age,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_explicit_option_works() {
        // This should work - RequiredValidation validates the full Option<T>
        let form = ExplicitOptionForm {
            age: Some(Age { value: 25 }),
        };
        assert!(form.validate().is_ok());

        let empty_form = ExplicitOptionForm { age: None };
        let err = empty_form.validate().unwrap_err();
        assert!(err.age().required_validation().is_some());
    }

    #[test]
    fn test_non_optional_newtype_validates() {
        // Non-optional newtype field - should always validate
        let item = OriginalStructBeforeTransformation {
            age: Age { value: 25 },
        };
        assert!(item.validate().is_ok());

        let invalid_item = OriginalStructBeforeTransformation {
            age: Age { value: -5 },
        };
        let err = invalid_item.validate().unwrap_err();
        // Should have inner error from the Age newtype validation
        // For simple newtype without validators, age() returns &InnerError directly
        assert!(!err.age().is_empty());
    }

    #[test]
    fn test_explicit_option_all_method() {
        // Test that .all() method works on newtype with RequiredValidation
        let empty_form = ExplicitOptionForm { age: None };
        let err = empty_form.validate().unwrap_err();

        // all() returns all failed validators - requires the Validator enum to exist
        let all_failed = err.age().all();
        assert_eq!(all_failed.count(), 1);

        // Valid case should have no failed validators
        let valid_form = ExplicitOptionForm {
            age: Some(Age { value: 25 }),
        };
        assert!(valid_form.validate().is_ok());
    }
}
