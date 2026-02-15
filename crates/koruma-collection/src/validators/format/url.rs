use koruma::{Validate, validator};

/// URL validation for koruma.
///
///
/// # Example
/// ```rust
/// use koruma::Koruma;
/// use koruma_collection::format::UrlValidation;
///
/// #[derive(Koruma)]
/// struct Resource {
///     #[koruma(UrlValidation::<_>)]
///     link: String,
/// }
/// ```
///
/// Validates that a string is a valid URL.
#[validator]
#[cfg_attr(feature = "internal-showcase", showcase(
    name = "URL",
    description = "Validates that the input is a valid URL",
    module = "format",
    create = |input: &str| -> anyhow::Result<_> {
        Ok(UrlValidation::builder()
            .with_value(input.to_string())
            .build())
    }
))]
#[derive(Clone, Debug)]
#[cfg_attr(feature = "fluent", derive(es_fluent::EsFluent))]
#[cfg_attr(feature = "fluent", fluent(namespace = "format"))]
pub struct UrlValidation<T: AsRef<str>> {
    /// The string being validated (stored for error context)
    #[koruma(value)]
    #[cfg_attr(feature = "fluent", fluent(skip))]
    pub actual: T,
}

impl<T: AsRef<str>> Validate<T> for UrlValidation<T> {
    fn validate(&self, value: &T) -> bool {
        let s = value.as_ref();
        url::Url::parse(s).is_ok()
    }
}

#[cfg(feature = "fmt")]
impl<T: AsRef<str>> std::fmt::Display for UrlValidation<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Not a valid URL.")
    }
}

#[cfg(test)]
mod tests {
    use koruma::Validate as _;

    use super::UrlValidation;

    #[test]
    fn accepts_valid_url() {
        let validator = UrlValidation {
            actual: String::new(),
        };
        assert!(validator.validate(&"https://example.com".to_string()));
    }

    #[test]
    fn rejects_invalid_url() {
        let validator = UrlValidation {
            actual: String::new(),
        };
        assert!(!validator.validate(&"not a url".to_string()));
    }
}
