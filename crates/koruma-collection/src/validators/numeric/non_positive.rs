use koruma::{Validate, validator};

use super::Numeric;

/// Non-positive number validation for koruma.
///
///
/// # Example
/// ```rust
/// use koruma::Koruma;
/// use koruma_collection::numeric::NonPositiveValidation;
///
/// #[derive(Koruma)]
/// struct Debit {
///     #[koruma(NonPositiveValidation::<_>::builder())]
///     amount: f64,
/// }
/// ```
///
/// Validates that a numeric value is non-positive (<= 0).
#[validator]
#[cfg_attr(feature = "internal-showcase", showcase(
    name = "Non-Positive Number",
    description = "Validates that the input is a non-positive number (<= 0)",
    input_type = Numeric,
    module = "numeric",
    create = |input: &str| -> anyhow::Result<_> {
        let num = input.parse::<f64>()?;
        Ok(NonPositiveValidation::builder()
            .with_value(num)
            .build())
    }
))]
#[derive(Clone, Debug)]
#[cfg_attr(feature = "fluent", derive(es_fluent::EsFluent))]
#[cfg_attr(feature = "fluent", fluent(namespace = "numeric"))]
pub struct NonPositiveValidation<T: Numeric> {
    /// The value being validated (stored for error context)
    #[koruma(value)]
    #[cfg_attr(feature = "fluent", fluent(skip))]
    actual: T,
}

impl<T: Numeric> Validate<T> for NonPositiveValidation<T> {
    fn validate(&self, value: &T) -> bool {
        *value <= T::zero()
    }
}

#[cfg(feature = "fmt")]
impl<T: Numeric> std::fmt::Display for NonPositiveValidation<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Must be zero or a negative number.")
    }
}

#[cfg(test)]
mod tests {
    use koruma::Validate as _;

    use super::NonPositiveValidation;

    #[test]
    fn accepts_zero_and_negative_values() {
        let validator = NonPositiveValidation { actual: 0_i32 };
        assert!(validator.validate(&0));
        assert!(validator.validate(&-1));
    }

    #[test]
    fn rejects_positive_values() {
        let validator = NonPositiveValidation { actual: 0_i32 };
        assert!(!validator.validate(&1));
    }
}
