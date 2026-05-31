use koruma::{Validate, validator};

use super::Numeric;

/// Non-negative number validation for koruma.
///
///
/// # Example
/// ```rust
/// use koruma::Koruma;
/// use koruma_collection::numeric::NonNegativeValidation;
///
/// #[derive(Koruma)]
/// struct Account {
///     #[koruma(NonNegativeValidation::<_>)]
///     balance: f64,
/// }
/// ```
///
/// Validates that a numeric value is non-negative (>= 0).
#[validator]
#[cfg_attr(feature = "internal-showcase", showcase(
    name = "Non-Negative Number",
    description = "Validates that the input is a non-negative number (>= 0)",
    input_type = Numeric,
    module = "numeric",
    create = |input: &str| -> anyhow::Result<_> {
        let num = input.parse::<f64>()?;
        Ok(NonNegativeValidation::with_value(num)
            .build())
    }
))]
#[derive(Clone, Debug)]
#[cfg_attr(feature = "fluent", derive(es_fluent::EsFluent))]
#[cfg_attr(feature = "fluent", fluent(namespace = "numeric"))]
pub struct NonNegativeValidation<T: Numeric> {
    /// The value being validated.
    #[koruma(value(capture = skip))]
    #[cfg_attr(feature = "fluent", fluent(skip))]
    actual: Option<T>,
}

impl<T: Numeric> Validate<T> for NonNegativeValidation<T> {
    fn validate(&self, value: &T) -> bool {
        value >= &T::zero()
    }
}

#[cfg(feature = "fmt")]
impl<T: Numeric> std::fmt::Display for NonNegativeValidation<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Must be zero or a positive number.")
    }
}

#[cfg(test)]
mod tests {
    use koruma::Validate as _;

    use super::NonNegativeValidation;

    #[test]
    fn accepts_zero_and_positive_values() {
        let validator = NonNegativeValidation { actual: None };
        assert!(validator.validate(&0));
        assert!(validator.validate(&1));
    }

    #[test]
    fn rejects_negative_values() {
        let validator = NonNegativeValidation { actual: None };
        assert!(!validator.validate(&-1));
    }
}
