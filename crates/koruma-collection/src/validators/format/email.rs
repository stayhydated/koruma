use koruma::{Validate, validator};

/// Email validation for koruma.
///
///
/// # Example
/// ```rust
/// use koruma::Koruma;
/// use koruma_collection::format::EmailValidation;
///
/// #[derive(Koruma)]
/// struct User {
///     #[koruma(EmailValidation::<_>)]
///     email: String,
/// }
/// ```
///
/// Validates that a string is a valid email address.
#[validator]
#[cfg_attr(feature = "internal-showcase", showcase(
    name = "Email",
    description = "Validates that the input is a valid email address",
    input_type = Text,
    module = "format",
    create = |input: &str| -> anyhow::Result<_> {
        Ok(EmailValidation::with_value(input.to_string())
            .build())
    }
))]
#[derive(Clone, Debug)]
#[cfg_attr(feature = "fluent", derive(es_fluent::EsFluent))]
#[cfg_attr(feature = "fluent", fluent(namespace = "format"))]
pub struct EmailValidation<T: AsRef<str>> {
    /// The string being validated (stored for error context)
    #[koruma(value)]
    #[cfg_attr(feature = "fluent", fluent(skip))]
    actual: T,
}

impl<T: AsRef<str>> Validate<T> for EmailValidation<T> {
    fn validate(&self, value: &T) -> bool {
        email_address::EmailAddress::is_valid(value.as_ref())
    }
}

#[cfg(feature = "fmt")]
impl<T: AsRef<str>> std::fmt::Display for EmailValidation<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Not a valid email address.")
    }
}

#[cfg(test)]
mod tests {
    use koruma::Validate as _;

    use super::EmailValidation;

    #[test]
    fn accepts_valid_email() {
        let validator = EmailValidation {
            actual: String::new(),
        };

        assert!(validator.validate(&"user@example.com".to_string()));
    }

    #[test]
    fn rejects_invalid_email() {
        let validator = EmailValidation {
            actual: String::new(),
        };

        assert!(!validator.validate(&"invalid@@example.com".to_string()));
        assert!(!validator.validate(&"missing-domain@".to_string()));
        assert!(!validator.validate(&"".to_string()));
    }
}
