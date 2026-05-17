use koruma::{Validate, validator};

use super::Numeric;

/// Positive number validation for koruma.
///
///
/// # Example
/// ```rust
/// use koruma::Koruma;
/// use koruma_collection::numeric::PositiveValidation;
///
/// #[derive(Koruma)]
/// struct Order {
///     #[koruma(PositiveValidation::<_>)]
///     quantity: i32,
/// }
/// ```
///
/// Validates that a numeric value is strictly positive (> 0).
#[validator]
#[cfg_attr(feature = "internal-showcase", showcase(
    name = "Positive Number",
    description = "Validates that the input is a positive number (> 0)",
    input_type = Numeric,
    module = "numeric",
    create = |input: &str| -> anyhow::Result<_> {
        let num = input.parse::<f64>()?;
        Ok(PositiveValidation::with_value(num)
            .build())
    }
))]
#[derive(Clone, Debug)]
#[cfg_attr(feature = "fluent", derive(es_fluent::EsFluent))]
#[cfg_attr(feature = "fluent", fluent(namespace = "numeric"))]
pub struct PositiveValidation<T: Numeric> {
    /// The value being validated (stored for error context)
    #[koruma(value)]
    #[cfg_attr(feature = "fluent", fluent(skip))]
    actual: T,
}

impl<T: Numeric> Validate<T> for PositiveValidation<T> {
    fn validate(&self, value: &T) -> bool {
        *value > T::zero()
    }
}

#[cfg(feature = "fmt")]
impl<T: Numeric> std::fmt::Display for PositiveValidation<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Must be positive.")
    }
}

#[cfg(test)]
mod tests {
    use koruma::Validate as _;

    use super::PositiveValidation;

    #[test]
    fn accepts_positive_values() {
        let validator = PositiveValidation { actual: 0_i32 };
        assert!(validator.validate(&1));
    }

    #[test]
    fn rejects_zero_and_negative_values() {
        let validator = PositiveValidation { actual: 0_i32 };
        assert!(!validator.validate(&0));
        assert!(!validator.validate(&-1));
    }
}
