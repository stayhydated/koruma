use koruma::{Validate, validator};

/// Suffix validation for koruma.
///
///
/// # Example
/// ```rust
/// use koruma::Koruma;
/// use koruma_collection::string::SuffixValidation;
///
/// #[derive(Koruma)]
/// struct File {
///     #[koruma(SuffixValidation::<_>::suffix(".txt"))]
///     name: String,
/// }
/// ```
///
/// Validates that a string ends with a specified suffix.
#[validator]
#[cfg_attr(feature = "internal-showcase", showcase(
    name = "Suffix '.rs'",
    description = "Validates that the input ends with '.rs'",
    input_type = Text,
    module = "string",
    create = |input: &str| -> anyhow::Result<_> {
        Ok(SuffixValidation::suffix(".rs")
            .with_value(input.to_string())
            .build())
    }
))]
#[derive(Clone, Debug)]
#[cfg_attr(feature = "fluent", derive(es_fluent::EsFluent))]
#[cfg_attr(feature = "fluent", fluent(namespace = "string"))]
pub struct SuffixValidation<T: AsRef<str>> {
    /// The suffix to check for
    #[koruma(setter(into))]
    pub suffix: String,
    /// The string being validated.
    #[koruma(value(capture = skip))]
    #[cfg_attr(feature = "fluent", fluent(skip))]
    actual: Option<T>,
}

impl<T: AsRef<str>> Validate<T> for SuffixValidation<T> {
    fn validate(&self, value: &T) -> bool {
        let s = value.as_ref();
        s.ends_with(&self.suffix)
    }
}

#[cfg(feature = "fmt")]
impl<T: AsRef<str>> std::fmt::Display for SuffixValidation<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Must end with '{}'.", self.suffix)
    }
}

#[cfg(test)]
mod tests {
    use koruma::Validate as _;

    use super::SuffixValidation;

    #[test]
    fn accepts_when_suffix_matches() {
        let validator = SuffixValidation {
            suffix: ".rs".to_string(),
            actual: None,
        };
        assert!(validator.validate(&"lib.rs".to_string()));
    }

    #[test]
    fn rejects_when_suffix_does_not_match() {
        let validator = SuffixValidation {
            suffix: ".rs".to_string(),
            actual: None,
        };
        assert!(!validator.validate(&"lib.ts".to_string()));
    }
}
