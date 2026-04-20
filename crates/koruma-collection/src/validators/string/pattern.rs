use koruma::{Validate, validator};

/// Pattern validation for koruma.
///
///
/// # Example
/// ```rust
/// use koruma::Koruma;
/// use koruma_collection::string::PatternValidation;
/// use regex::Regex;
///
/// #[derive(Koruma)]
/// struct User {
///     #[koruma(PatternValidation<_>(pattern = Regex::new(r"^[a-zA-Z0-9_]+$").unwrap()))]
///     username: String,
/// }
/// ```
///
/// Validates that a string matches a compiled regular expression pattern.
#[validator]
#[cfg_attr(feature = "internal-showcase", showcase(
    name = "Regex Pattern",
    description = "Validates that the input matches a regex pattern (uses ^[a-zA-Z0-9_]+$)",
    input_type = Text,
    module = "string",
    create = |input: &str| -> anyhow::Result<_> {
        Ok(PatternValidation::builder()
            .with_value(input.to_string())
            .pattern(::regex::Regex::new(r"^[a-zA-Z0-9_]+$")?)
            .build())
    }
))]
#[derive(Clone, Debug)]
#[cfg_attr(feature = "fluent", derive(es_fluent::EsFluent))]
#[cfg_attr(feature = "fluent", fluent(namespace = "string"))]
pub struct PatternValidation<T: AsRef<str>> {
    /// The compiled regex pattern to match against
    #[cfg_attr(feature = "fluent", fluent(skip))]
    pub pattern: regex::Regex,
    /// The string being validated (stored for error context)
    #[koruma(value)]
    #[cfg_attr(feature = "fluent", fluent(skip))]
    actual: T,
}

impl<T: AsRef<str>> Validate<T> for PatternValidation<T> {
    fn validate(&self, value: &T) -> bool {
        self.pattern.is_match(value.as_ref())
    }
}

#[cfg(feature = "fmt")]
impl<T: AsRef<str>> std::fmt::Display for PatternValidation<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Does not match the required pattern.")
    }
}

#[cfg(test)]
mod tests {
    use koruma::Validate as _;

    use super::PatternValidation;

    #[test]
    fn accepts_when_pattern_matches() {
        let validator = PatternValidation::builder()
            .pattern(regex::Regex::new(r"^\d+$").unwrap())
            .with_value(String::new())
            .build();
        assert!(validator.validate(&"12345".to_string()));
    }

    #[test]
    fn rejects_when_pattern_does_not_match() {
        let validator = PatternValidation::builder()
            .pattern(regex::Regex::new(r"^\d+$").unwrap())
            .with_value(String::new())
            .build();
        assert!(!validator.validate(&"123a".to_string()));
    }

    #[test]
    fn invalid_pattern_fails_at_construction_time() {
        let pattern = "(".to_string();
        assert!(regex::Regex::new(&pattern).is_err());
    }

    #[test]
    fn validate_uses_the_current_pattern_value() {
        let mut validator = PatternValidation::builder()
            .pattern(regex::Regex::new(r"^a$").unwrap())
            .with_value(String::new())
            .build();

        assert!(validator.validate(&"a".to_string()));
        assert!(!validator.validate(&"b".to_string()));

        validator.pattern = regex::Regex::new(r"^b$").unwrap();

        assert!(!validator.validate(&"a".to_string()));
        assert!(validator.validate(&"b".to_string()));
    }

    #[test]
    fn builder_accepts_precompiled_regexes() {
        let validator = PatternValidation::builder()
            .pattern(regex::Regex::new(r"^cache-hit-\d+$").unwrap())
            .with_value(String::new())
            .build();

        assert!(validator.validate(&"cache-hit-42".to_string()));
    }

    #[cfg(feature = "fmt")]
    #[test]
    fn display_does_not_echo_the_pattern() {
        let validator = PatternValidation::builder()
            .pattern(regex::Regex::new(r"^\d+$").unwrap())
            .with_value(String::new())
            .build();

        assert_eq!(
            validator.to_string(),
            "Does not match the required pattern."
        );
        assert!(!validator.to_string().contains(r"^\d+$"));
    }
}
