use koruma::{Validate, validator};

/// Alphanumeric validation for koruma.
///
///
/// # Example
/// ```rust
/// use koruma::Koruma;
/// use koruma_collection::string::AlphanumericValidation;
///
/// #[derive(Koruma)]
/// struct User {
///     #[koruma(AlphanumericValidation::<_>)]
///     username: String,
/// }
/// ```
///
/// Validates that a string contains only alphanumeric characters.
#[validator]
#[cfg_attr(feature = "internal-showcase", showcase(
    name = "Alphanumeric",
    description = "Validates that the input contains only alphanumeric characters",
    input_type = Text,
    module = "string",
    create = |input: &str| -> anyhow::Result<_> {
        Ok(AlphanumericValidation::with_value(input.to_string())
            .build())
    }
))]
#[derive(Clone, Debug)]
#[cfg_attr(feature = "fluent", derive(es_fluent::EsFluent))]
#[cfg_attr(feature = "fluent", fluent(namespace = "string"))]
pub struct AlphanumericValidation<T: AsRef<str>> {
    /// The string being validated.
    #[koruma(value, skip_capture)]
    #[cfg_attr(feature = "fluent", fluent(skip))]
    actual: Option<T>,
}

impl<T: AsRef<str>> Validate<T> for AlphanumericValidation<T> {
    fn validate(&self, value: &T) -> bool {
        let s = value.as_ref();
        s.chars().all(|c| c.is_alphanumeric())
    }
}

#[cfg(feature = "fmt")]
impl<T: AsRef<str>> std::fmt::Display for AlphanumericValidation<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Must contain only letters and numbers.")
    }
}

#[cfg(test)]
mod tests {
    use koruma::Validate as _;

    use super::AlphanumericValidation;

    #[test]
    fn accepts_alphanumeric_input() {
        let validator = AlphanumericValidation { actual: None };
        assert!(validator.validate(&"abc123".to_string()));
    }

    #[test]
    fn rejects_non_alphanumeric_input() {
        let validator = AlphanumericValidation { actual: None };
        assert!(!validator.validate(&"abc-123".to_string()));
    }
}
