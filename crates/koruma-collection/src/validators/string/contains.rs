use koruma::{Validate, validator};

/// Contains validation for koruma.
///
///
/// # Example
/// ```rust
/// use koruma::Koruma;
/// use koruma_collection::string::ContainsValidation;
///
/// #[derive(Koruma)]
/// struct User {
///     #[koruma(ContainsValidation::<_>::substring("test"))]
///     email: String,
/// }
/// ```
///
/// Validates that a string contains a specified substring.
#[validator]
#[cfg_attr(feature = "internal-showcase", showcase(
    name = "Contains 'test'",
    description = "Validates that the input contains the substring 'test'",
    input_type = Text,
    module = "string",
    create = |input: &str| -> anyhow::Result<_> {
        Ok(ContainsValidation::substring("test")
            .with_value(input.to_string())
            .build())
    }
))]
#[derive(Clone, Debug)]
#[cfg_attr(feature = "fluent", derive(es_fluent::EsFluent))]
#[cfg_attr(feature = "fluent", fluent(namespace = "string"))]
pub struct ContainsValidation<T: AsRef<str>> {
    /// The substring to search for
    #[builder(into)]
    pub substring: String,
    /// The string being validated (stored for error context)
    #[koruma(value)]
    #[cfg_attr(feature = "fluent", fluent(skip))]
    actual: T,
}

impl<T: AsRef<str>> Validate<T> for ContainsValidation<T> {
    fn validate(&self, value: &T) -> bool {
        let s = value.as_ref();
        s.contains(&self.substring)
    }
}

#[cfg(feature = "fmt")]
impl<T: AsRef<str>> std::fmt::Display for ContainsValidation<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Must contain the substring '{}'.", self.substring)
    }
}

#[cfg(test)]
mod tests {
    use koruma::Validate as _;

    use super::ContainsValidation;

    #[test]
    fn accepts_when_substring_is_present() {
        let validator = ContainsValidation {
            substring: "ell".to_string(),
            actual: String::new(),
        };
        assert!(validator.validate(&"hello".to_string()));
    }

    #[test]
    fn rejects_when_substring_is_missing() {
        let validator = ContainsValidation {
            substring: "ell".to_string(),
            actual: String::new(),
        };
        assert!(!validator.validate(&"world".to_string()));
    }
}
