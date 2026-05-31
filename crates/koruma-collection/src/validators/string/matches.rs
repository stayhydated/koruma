use koruma::{Validate, validator};

/// Field matching validation for koruma.
///
///
/// # Example
/// ```rust
/// use koruma::Koruma;
/// use koruma_collection::string::MatchesValidation;
///
/// #[derive(Koruma)]
/// struct User {
///     password: String,
///     #[koruma(MatchesValidation::<_>::other(self.password.clone()))]
///     confirm_password: String,
/// }
/// ```
///
/// Validates that a value matches another value.
#[validator]
#[cfg_attr(feature = "internal-showcase", showcase(
    name = "Matches Value",
    description = "Validates that the input matches 'expected'",
    input_type = Text,
    module = "string",
    create = |input: &str| -> anyhow::Result<_> {
        Ok(MatchesValidation::with_value(input.to_string())
            .other("expected".to_string())
            .build())
    }
))]
#[derive(Clone, Debug)]
#[cfg_attr(feature = "fluent", derive(es_fluent::EsFluent))]
#[cfg_attr(feature = "fluent", fluent(namespace = "string"))]
pub struct MatchesValidation<T: PartialEq> {
    /// The value to match against
    #[cfg_attr(feature = "fluent", fluent(skip))]
    pub other: T,
    /// The value being validated.
    #[koruma(value, skip_capture)]
    #[cfg_attr(feature = "fluent", fluent(skip))]
    actual: Option<T>,
}

impl<T: PartialEq> Validate<T> for MatchesValidation<T> {
    fn validate(&self, value: &T) -> bool {
        value == &self.other
    }
}

#[cfg(feature = "fmt")]
impl<T: PartialEq> std::fmt::Display for MatchesValidation<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Does not match the expected value.")
    }
}

#[cfg(test)]
mod tests {
    use koruma::Validate as _;

    use super::MatchesValidation;

    #[test]
    fn accepts_when_values_match() {
        let validator = MatchesValidation {
            other: "secret".to_string(),
            actual: None,
        };
        assert!(validator.validate(&"secret".to_string()));
    }

    #[test]
    fn rejects_when_values_do_not_match() {
        let validator = MatchesValidation {
            other: "secret".to_string(),
            actual: None,
        };
        assert!(!validator.validate(&"SECRET".to_string()));
    }

    #[cfg(feature = "fmt")]
    #[test]
    fn display_does_not_echo_the_other_value() {
        let validator = MatchesValidation {
            other: "super-secret".to_string(),
            actual: None,
        };

        assert_eq!(validator.to_string(), "Does not match the expected value.");
        assert!(!validator.to_string().contains("super-secret"));
    }
}
