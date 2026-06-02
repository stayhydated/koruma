// Coverage for external macro wrapping of a newtype field into Option<T>.

use koruma::Koruma;

use super::validators::{NumberRangeValidation, RequiredValidation};

/// A validated form newtype.
#[derive(Clone, Debug, Koruma)]
#[koruma(newtype)]
pub struct Age {
    #[koruma(NumberRangeValidation::min(0).max(150))]
    pub value: i32,
}

/// Expanded form shape with an explicitly optional newtype field.
#[derive(Koruma)]
pub struct ExplicitOptionForm {
    #[koruma(newtype, full(RequiredValidation::<_>))]
    pub age: Option<Age>,
}

/// Direct form shape with a required newtype field.
#[derive(Koruma)]
pub struct RequiredNewtypeForm {
    #[koruma(newtype)]
    pub age: Age,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_explicit_option_works() {
        // RequiredValidation validates the full Option<T>.
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
        // Non-optional newtype fields delegate to the wrapped type's validation.
        let item = RequiredNewtypeForm {
            age: Age { value: 25 },
        };
        assert!(item.validate().is_ok());

        let invalid_item = RequiredNewtypeForm {
            age: Age { value: -5 },
        };
        let err = invalid_item.validate().unwrap_err();
        // For simple newtypes without additional field validators, age() returns
        // &InnerError directly.
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
