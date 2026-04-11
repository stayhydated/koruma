use koruma::{Validate, validator};

/// ASCII validation for koruma.
///
///
/// # Example
/// ```rust
/// use koruma::Koruma;
/// use koruma_collection::string::AsciiValidation;
///
/// #[derive(Koruma)]
/// struct User {
///     #[koruma(AsciiValidation<_>)]
///     username: String,
/// }
/// ```
///
/// Validates that a string contains only ASCII characters.
#[validator]
#[cfg_attr(feature = "internal-showcase", showcase(
    name = "ASCII",
    description = "Validates that the input contains only ASCII characters",
    module = "string",
    create = |input: &str| -> anyhow::Result<_> {
        Ok(AsciiValidation::builder()
            .with_value(input.to_string())
            .build())
    }
))]
#[derive(Clone, Debug)]
#[cfg_attr(feature = "fluent", derive(es_fluent::EsFluent))]
#[cfg_attr(feature = "fluent", fluent(namespace = "string"))]
pub struct AsciiValidation<T: AsRef<str>> {
    /// The string being validated (stored for error context)
    #[koruma(value)]
    #[cfg_attr(feature = "fluent", fluent(skip))]
    actual: T,
}

impl<T: AsRef<str>> Validate<T> for AsciiValidation<T> {
    fn validate(&self, value: &T) -> bool {
        let s = value.as_ref();
        s.is_ascii()
    }
}

#[cfg(feature = "fmt")]
impl<T: AsRef<str>> std::fmt::Display for AsciiValidation<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Must contain only ASCII characters.")
    }
}

#[cfg(test)]
mod tests {
    use koruma::Validate as _;

    use super::AsciiValidation;

    #[test]
    fn accepts_ascii_input() {
        let validator = AsciiValidation {
            actual: String::new(),
        };
        assert!(validator.validate(&"hello123".to_string()));
    }

    #[test]
    fn rejects_non_ascii_input() {
        let validator = AsciiValidation {
            actual: String::new(),
        };
        assert!(!validator.validate(&"héllo".to_string()));
    }
}
