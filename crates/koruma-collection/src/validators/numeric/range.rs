use koruma::{Validate, validator};

/// Range validation for koruma.
///
///
/// # Example
/// ```rust
/// use koruma::Koruma;
/// use koruma_collection::numeric::RangeValidation;
///
/// #[derive(Koruma)]
/// struct Score {
///     #[koruma(RangeValidation::<_>::min(0).max(100))]
///     value: u32,
/// }
/// ```
///
/// Validates that a numeric value is within specified bounds.
#[validator]
#[cfg_attr(feature = "internal-showcase", showcase(
    name = "Range [0, 100)",
    description = "Validates that the input is a number between 0 and 100",
    input_type = Numeric,
    module = "numeric",
    create = |input: &str| -> anyhow::Result<_> {
        let num = input.parse::<f64>()?;
        Ok(RangeValidation::min(0_f64)
            .max(100_f64)
            .exclusive_max(true)
            .with_value(num)
            .build())
    }
))]
#[derive(Clone, Debug)]
#[cfg_attr(feature = "fluent", derive(es_fluent::EsFluent))]
#[cfg_attr(feature = "fluent", fluent(namespace = "numeric"))]
pub struct RangeValidation<T: PartialOrd + Copy + std::fmt::Display> {
    /// Minimum allowed value (inclusive)
    #[cfg_attr(feature = "fluent", fluent(value(|x: &T| x.to_string())))]
    pub min: T,
    /// Whether the minimum value is exclusive
    #[cfg_attr(
        feature = "fluent",
        fluent(
            arg_name = "left_delimiter",
            value(|x: &bool| if *x { "(" } else { "[" })
        )
    )]
    #[builder(default = false)]
    pub exclusive_min: bool,
    /// Maximum allowed value (inclusive)
    #[cfg_attr(feature = "fluent", fluent(value(|x: &T| x.to_string())))]
    pub max: T,
    /// Whether the maximum value is exclusive
    #[cfg_attr(
        feature = "fluent",
        fluent(
            arg_name = "right_delimiter",
            value(|x: &bool| if *x { ")" } else { "]" })
        )
    )]
    #[builder(default = false)]
    pub exclusive_max: bool,
    /// The value being validated (stored for error context)
    #[koruma(value)]
    #[cfg_attr(feature = "fluent", fluent(skip))]
    actual: T,
}

impl<T: PartialOrd + Copy + std::fmt::Display> Validate<T> for RangeValidation<T> {
    fn validate(&self, value: &T) -> bool {
        let lower_ok = if self.exclusive_min {
            *value > self.min
        } else {
            *value >= self.min
        };

        let upper_ok = if self.exclusive_max {
            *value < self.max
        } else {
            *value <= self.max
        };

        lower_ok && upper_ok
    }
}

#[cfg(feature = "fmt")]
impl<T: PartialOrd + Copy + std::fmt::Display> std::fmt::Display for RangeValidation<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let left_delimiter = if self.exclusive_min { "(" } else { "[" };
        let right_delimiter = if self.exclusive_max { ")" } else { "]" };

        write!(
            f,
            "Must be in the range {}{}, {}{}.",
            left_delimiter, self.min, self.max, right_delimiter
        )
    }
}

#[cfg(test)]
mod tests {
    use koruma::Validate as _;

    use super::RangeValidation;

    #[test]
    fn accepts_values_within_inclusive_bounds() {
        let validator = RangeValidation {
            min: 1_i32,
            exclusive_min: false,
            max: 3_i32,
            exclusive_max: false,
            actual: 0_i32,
        };

        assert!(validator.validate(&1));
        assert!(validator.validate(&2));
        assert!(validator.validate(&3));
    }

    #[test]
    fn rejects_values_outside_bounds() {
        let validator = RangeValidation {
            min: 1_i32,
            exclusive_min: false,
            max: 3_i32,
            exclusive_max: false,
            actual: 0_i32,
        };

        assert!(!validator.validate(&0));
        assert!(!validator.validate(&4));
    }

    #[test]
    fn supports_exclusive_bounds() {
        let validator = RangeValidation {
            min: 1_i32,
            exclusive_min: true,
            max: 3_i32,
            exclusive_max: true,
            actual: 0_i32,
        };

        assert!(!validator.validate(&1));
        assert!(validator.validate(&2));
        assert!(!validator.validate(&3));
    }

    #[cfg(feature = "fmt")]
    #[test]
    fn display_uses_interval_notation_for_exclusive_bounds() {
        let validator = RangeValidation {
            min: 1_i32,
            exclusive_min: true,
            max: 3_i32,
            exclusive_max: false,
            actual: 0_i32,
        };

        assert_eq!(validator.to_string(), "Must be in the range (1, 3].");
    }

    #[cfg(feature = "fmt")]
    #[test]
    fn display_uses_the_current_exclusivity_flags() {
        let mut validator = RangeValidation::min(1_i32)
            .max(3_i32)
            .with_value(0_i32)
            .build();

        assert_eq!(validator.to_string(), "Must be in the range [1, 3].");

        validator.exclusive_min = true;
        validator.exclusive_max = true;

        assert_eq!(validator.to_string(), "Must be in the range (1, 3).");
    }
}
