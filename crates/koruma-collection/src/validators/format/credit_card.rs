use koruma::{Validate, validator};

/// Credit card validation for koruma.
///
///
/// # Example
/// ```rust
/// use koruma::Koruma;
/// use koruma_collection::format::CreditCardValidation;
///
/// #[derive(Koruma)]
/// struct Payment {
///     #[koruma(CreditCardValidation::<_>)]
///     card_number: String,
/// }
/// ```
///
/// Validates that a string is a valid credit card number.
#[validator]
#[cfg_attr(feature = "internal-showcase", showcase(
    name = "Credit Card",
    description = "Validates that the input is a valid credit card number",
    input_type = Text,
    module = "format",
    create = |input: &str| -> anyhow::Result<_> {
        Ok(CreditCardValidation::with_value(input.to_string())
            .build())
    }
))]
#[derive(Clone, Debug)]
#[cfg_attr(feature = "fluent", derive(es_fluent::EsFluent))]
#[cfg_attr(feature = "fluent", fluent(namespace = "format"))]
pub struct CreditCardValidation<T: AsRef<str>> {
    /// The string being validated.
    #[koruma(value(capture = skip))]
    #[cfg_attr(feature = "fluent", fluent(skip))]
    actual: Option<T>,
}

impl<T: AsRef<str>> Validate<T> for CreditCardValidation<T> {
    fn validate(&self, value: &T) -> bool {
        let s = value.as_ref();
        card_validate::Validate::from(s).is_ok()
    }
}

#[cfg(feature = "fmt")]
impl<T: AsRef<str>> std::fmt::Display for CreditCardValidation<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Not a valid credit card number.")
    }
}

#[cfg(test)]
mod tests {
    use koruma::Validate as _;

    use super::CreditCardValidation;

    #[test]
    fn accepts_valid_credit_card_number() {
        let validator = CreditCardValidation { actual: None };
        assert!(validator.validate(&"4111111111111111".to_string()));
    }

    #[test]
    fn rejects_invalid_credit_card_number() {
        let validator = CreditCardValidation { actual: None };
        assert!(!validator.validate(&"4111111111111112".to_string()));
    }
}
