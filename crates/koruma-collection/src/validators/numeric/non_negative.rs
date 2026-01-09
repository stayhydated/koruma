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
#[cfg_attr(feature = "showcase", showcase(
    name = "Non-Negative Number",
    description = "Validates that the input is a non-negative number (>= 0)",
    input_type = Numeric,
    create = |input: &str| {
        let num = input.parse::<f64>().unwrap_or(0.0);
        NonNegativeValidation::builder()
            .with_value(num)
            .build()
    }
))]
#[derive(Clone, Debug)]
#[cfg_attr(feature = "fluent", derive(es_fluent::EsFluent))]
pub struct NonNegativeValidation<T: Numeric> {
    /// The value being validated (stored for error context)
    #[koruma(value)]
    #[cfg_attr(feature = "fluent", fluent(value(|x: &T| x.to_string())))]
    pub actual: T,
}

impl<T: Numeric> Validate<T> for NonNegativeValidation<T> {
    fn validate(&self, value: &T) -> bool {
        *value >= T::default()
    }
}

#[cfg(feature = "fmt")]
impl<T: Numeric> std::fmt::Display for NonNegativeValidation<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "value {} must be non-negative (>= 0)", self.actual)
    }
}
