use koruma::{Validate, validator};

/// Required validation for koruma.
///
///
/// # Example
/// ```rust
/// use koruma::Koruma;
/// use koruma_collection::general::RequiredValidation;
///
/// #[derive(Koruma)]
/// struct User {
///     // <Option<_>> substitutes `_` with the inner type (String), giving Option<String>
///     #[koruma(RequiredValidation<Option<_>>)]
///     name: Option<String>,
/// }
/// ```
///
/// Validates that a value is present (not None for Option types).
#[validator]
#[cfg_attr(feature = "fluent", derive(es_fluent::EsFluent))]
#[cfg_attr(feature = "fluent", fluent(namespace = "general"))]
pub struct RequiredValidation<T> {
    /// Presence-only validation does not need to retain the input in errors.
    #[koruma(value, skip_capture)]
    #[cfg_attr(feature = "fluent", fluent(skip))]
    actual: Option<T>,
}

impl<T> Clone for RequiredValidation<T> {
    fn clone(&self) -> Self {
        Self { actual: None }
    }
}

impl<T> std::fmt::Debug for RequiredValidation<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RequiredValidation")
            .field("actual", &"<skipped>")
            .finish()
    }
}

impl<T> Validate<Option<T>> for RequiredValidation<Option<T>> {
    fn validate(&self, value: &Option<T>) -> bool {
        value.is_some()
    }
}

#[cfg(feature = "fmt")]
impl<T> std::fmt::Display for RequiredValidation<Option<T>> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "This field is required.")
    }
}

#[cfg(test)]
mod tests {
    use koruma::Validate as _;

    use super::RequiredValidation;

    #[test]
    fn accepts_some_values() {
        let validator = RequiredValidation::<Option<String>> { actual: None };
        assert!(validator.validate(&Some("value".to_string())));
    }

    #[test]
    fn rejects_none_values() {
        let validator = RequiredValidation::<Option<String>> { actual: None };
        assert!(!validator.validate(&None));
    }

    #[cfg(feature = "fmt")]
    #[test]
    fn display_mentions_presence_not_emptiness() {
        let validator = RequiredValidation::<Option<String>> { actual: None };
        assert_eq!(validator.to_string(), "This field is required.");
    }
}
