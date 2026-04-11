use koruma::{Validate, validator};

/// Phone number validation for koruma.
///
///
/// # Example
/// ```rust
/// use koruma::Koruma;
/// use koruma_collection::format::PhoneNumberValidation;
///
/// #[derive(Koruma)]
/// struct Contact {
///     #[koruma(PhoneNumberValidation<_>)]
///     phone: String,
/// }
/// ```
///
/// Validates that a string is a valid phone number.
#[validator]
#[cfg_attr(feature = "internal-showcase", showcase(
    name = "Phone Number",
    description = "Validates that the input is a valid phone number",
    module = "format",
    create = |input: &str| -> anyhow::Result<_> {
        Ok(PhoneNumberValidation::builder()
            .with_value(input.to_string())
            .build())
    }
))]
#[derive(Clone, Debug)]
#[cfg_attr(feature = "fluent", derive(es_fluent::EsFluent))]
#[cfg_attr(feature = "fluent", fluent(namespace = "format"))]
pub struct PhoneNumberValidation<T: AsRef<str>> {
    /// The string being validated (stored for error context)
    #[koruma(value)]
    #[cfg_attr(feature = "fluent", fluent(skip))]
    actual: T,
}

impl<T: AsRef<str>> Validate<T> for PhoneNumberValidation<T> {
    fn validate(&self, value: &T) -> bool {
        use std::str::FromStr as _;

        let s = value.as_ref();
        match phonenumber::PhoneNumber::from_str(s) {
            Ok(number) => number.is_valid(),
            Err(_) => false,
        }
    }
}

#[cfg(feature = "fmt")]
impl<T: AsRef<str>> std::fmt::Display for PhoneNumberValidation<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Not a valid phone number.")
    }
}

#[cfg(test)]
mod tests {
    use koruma::Validate as _;

    use super::PhoneNumberValidation;

    #[test]
    fn accepts_valid_phone_number() {
        let validator = PhoneNumberValidation {
            actual: String::new(),
        };
        assert!(validator.validate(&"+14155552671".to_string()));
    }

    #[test]
    fn rejects_invalid_phone_number() {
        let validator = PhoneNumberValidation {
            actual: String::new(),
        };
        assert!(!validator.validate(&"123".to_string()));
    }
}
