use koruma::{Validate, validator};

use super::Numeric;

/// Negative number validation for koruma.
///
///
/// # Example
/// ```rust
/// use koruma::Koruma;
/// use koruma_collection::numeric::NegativeValidation;
///
/// #[derive(Koruma)]
/// struct Temperature {
///     #[koruma(NegativeValidation::<_>)]
///     celsius: f64,
/// }
/// ```
///
/// Validates that a numeric value is strictly negative (< 0).
#[validator]
#[cfg_attr(feature = "internal-showcase", showcase(
    name = "Negative Number",
    description = "Validates that the input is a negative number (< 0)",
    input_type = Numeric,
    module = "numeric",
    create = |input: &str| -> anyhow::Result<_> {
        let num = input.parse::<f64>()?;
        Ok(NegativeValidation::builder()
            .with_value(num)
            .build())
    }
))]
#[derive(Clone, Debug)]
#[cfg_attr(feature = "fluent", derive(es_fluent::EsFluent))]
#[cfg_attr(feature = "fluent", fluent(namespace = "numeric"))]
pub struct NegativeValidation<T: Numeric> {
    /// The value being validated (stored for error context)
    #[koruma(value)]
    #[cfg_attr(feature = "fluent", fluent(skip))]
    pub actual: T,
}

impl<T: Numeric> Validate<T> for NegativeValidation<T> {
    fn validate(&self, value: &T) -> bool {
        *value < T::default()
    }
}

#[cfg(feature = "fmt")]
impl<T: Numeric> std::fmt::Display for NegativeValidation<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "value {} must be negative (< 0)", self.actual)
    }
}
