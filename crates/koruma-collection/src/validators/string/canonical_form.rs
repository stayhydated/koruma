use koruma::{Validate, validator};

/// Predicate-based validation for strings that must already be canonical.
///
/// This validator does not normalize input. Use it at storage or API
/// boundaries after constructors or parsers have already applied the
/// application-owned normalization policy.
///
/// # Example
/// ```rust
/// use koruma::Koruma;
/// use koruma_collection::string::CanonicalFormValidation;
///
/// fn is_lowercase_token(value: &str) -> bool {
///     !value.is_empty()
///         && value
///             .bytes()
///             .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
/// }
///
/// #[derive(Koruma)]
/// struct Provider {
///     #[koruma(CanonicalFormValidation::<_>.predicate(is_lowercase_token))]
///     provider_id: String,
/// }
/// ```
#[validator]
#[cfg_attr(feature = "internal-showcase", showcase(
    name = "Canonical Form",
    description = "Validates that the input is already in canonical lowercase token form",
    input_type = Text,
    module = "string",
    create = |input: &str| -> anyhow::Result<_> {
        Ok(CanonicalFormValidation::<String>::predicate(showcase_canonical_ascii_token)
            .with_value(input.to_string())
            .build())
    }
))]
#[derive(Clone, Debug)]
#[cfg_attr(feature = "fluent", derive(es_fluent::EsFluent))]
#[cfg_attr(feature = "fluent", fluent(namespace = "string"))]
pub struct CanonicalFormValidation<T: AsRef<str>> {
    /// Returns true only when the input is already in canonical form.
    #[koruma(setter(required))]
    #[cfg_attr(feature = "fluent", fluent(skip))]
    predicate: fn(&str) -> bool,
    /// The string being validated.
    #[koruma(skip_capture)]
    #[cfg_attr(feature = "fluent", fluent(skip))]
    actual: Option<T>,
}

impl<T: AsRef<str>> Validate<T> for CanonicalFormValidation<T> {
    fn validate(&self, value: &T) -> bool {
        (self.predicate)(value.as_ref())
    }
}

#[cfg(feature = "fmt")]
impl<T: AsRef<str>> std::fmt::Display for CanonicalFormValidation<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Must already be in canonical form.")
    }
}

#[cfg(feature = "internal-showcase")]
fn showcase_canonical_ascii_token(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

#[cfg(test)]
mod tests {
    use koruma::Validate as _;

    use super::CanonicalFormValidation;

    fn lowercase_token(value: &str) -> bool {
        !value.is_empty()
            && value
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
    }

    #[test]
    fn accepts_values_that_already_match_the_canonical_policy() {
        let validator: CanonicalFormValidation<String> = CanonicalFormValidation {
            predicate: lowercase_token,
            actual: None,
        };

        assert!(validator.validate(&"provider-1".to_string()));
    }

    #[test]
    fn rejects_values_that_require_normalization() {
        let validator: CanonicalFormValidation<String> = CanonicalFormValidation {
            predicate: lowercase_token,
            actual: None,
        };

        assert!(!validator.validate(&"Provider-1".to_string()));
        assert!(!validator.validate(&" provider-1 ".to_string()));
    }

    #[test]
    fn builder_accepts_a_required_canonical_predicate() {
        let validator = CanonicalFormValidation::predicate(lowercase_token)
            .with_value("provider-1".to_string())
            .build();

        assert!(validator.validate(&"provider-1".to_string()));
        assert!(!validator.validate(&"PROVIDER-1".to_string()));
    }

    #[cfg(feature = "fmt")]
    #[test]
    fn display_communicates_noncanonical_input_without_echoing_it() {
        let validator: CanonicalFormValidation<String> = CanonicalFormValidation {
            predicate: lowercase_token,
            actual: None,
        };

        assert_eq!(validator.to_string(), "Must already be in canonical form.");
        assert!(!validator.to_string().contains("Provider-1"));
    }
}
