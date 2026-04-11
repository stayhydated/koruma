use koruma::{Validate, validator};

/// Prefix validation for koruma.
///
///
/// # Example
/// ```rust
/// use koruma::Koruma;
/// use koruma_collection::string::PrefixValidation;
///
/// #[derive(Koruma)]
/// struct Config {
///     #[koruma(PrefixValidation<_>(prefix = "config_"))]
///     key: String,
/// }
/// ```
///
/// Validates that a string starts with a specified prefix.
#[validator]
#[cfg_attr(feature = "internal-showcase", showcase(
    name = "Prefix 'hello'",
    description = "Validates that the input starts with 'hello'",
    module = "string",
    create = |input: &str| -> anyhow::Result<_> {
        Ok(PrefixValidation::builder()
            .prefix("hello")
            .with_value(input.to_string())
            .build())
    }
))]
#[derive(Clone, Debug)]
#[cfg_attr(feature = "fluent", derive(es_fluent::EsFluent))]
#[cfg_attr(feature = "fluent", fluent(namespace = "string"))]
pub struct PrefixValidation<T: AsRef<str>> {
    /// The prefix to check for
    #[builder(into)]
    pub prefix: String,
    /// The string being validated (stored for error context)
    #[koruma(value)]
    #[cfg_attr(feature = "fluent", fluent(skip))]
    pub actual: T,
}

impl<T: AsRef<str>> Validate<T> for PrefixValidation<T> {
    fn validate(&self, value: &T) -> bool {
        let s = value.as_ref();
        s.starts_with(&self.prefix)
    }
}

#[cfg(feature = "fmt")]
impl<T: AsRef<str>> std::fmt::Display for PrefixValidation<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Must start with '{}'.", self.prefix)
    }
}

#[cfg(test)]
mod tests {
    use koruma::Validate as _;

    use super::PrefixValidation;

    #[test]
    fn accepts_when_prefix_matches() {
        let validator = PrefixValidation {
            prefix: "pre".to_string(),
            actual: String::new(),
        };
        assert!(validator.validate(&"prefix".to_string()));
    }

    #[test]
    fn rejects_when_prefix_does_not_match() {
        let validator = PrefixValidation {
            prefix: "pre".to_string(),
            actual: String::new(),
        };
        assert!(!validator.validate(&"xprefix".to_string()));
    }
}
