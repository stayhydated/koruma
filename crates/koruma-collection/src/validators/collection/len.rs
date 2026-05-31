use koruma::{Validate, validator};

use super::HasLen;

/// Length validation for collections.
///
///
/// # Example
/// ```rust
/// use koruma::Koruma;
/// use koruma_collection::collection::LenValidation;
///
/// #[derive(Koruma)]
/// struct Order {
///     #[koruma(LenValidation::<_>::min(1).max(5))]
///     items: Vec<String>,
/// }
/// ```
///
/// Validates that a collection's length is within the specified bounds.
///
/// For `String` and `str`, length is measured in Unicode scalar values (`char`s).
///
/// Works with any type that implements `HasLen`.
#[validator]
#[cfg_attr(feature = "internal-showcase", showcase(
    name = "Length",
    description = "Validates string length is between 1 and 10",
    input_type = Text,
    module = "collection",
    create = |input: &str| -> anyhow::Result<_> {
        Ok(LenValidation::min(1)
            .max(10)
            .with_value(input.to_string())
            .build())
    }
))]
#[derive(Clone, Debug)]
#[cfg_attr(feature = "fluent", derive(es_fluent::EsFluent))]
#[cfg_attr(feature = "fluent", fluent(namespace = "collection"))]
pub struct LenValidation<T: HasLen> {
    /// Minimum allowed length (inclusive)
    min: usize,
    /// Maximum allowed length (inclusive)
    max: usize,
    /// The collection being validated.
    #[koruma(value(capture = skip))]
    #[cfg_attr(feature = "fluent", fluent(skip))]
    actual: Option<T>,
}

impl<T: HasLen> Validate<T> for LenValidation<T> {
    fn validate(&self, value: &T) -> bool {
        let len = value.len();
        !(len < self.min || len > self.max)
    }
}

#[cfg(feature = "fmt")]
impl<T: HasLen> std::fmt::Display for LenValidation<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "The length must be between {} and {}.",
            self.min, self.max
        )
    }
}

#[cfg(test)]
mod tests {
    use koruma::Validate as _;

    use crate::validators::collection::HasLen;

    use super::LenValidation;

    struct NonCloneCollection(usize);

    impl HasLen for NonCloneCollection {
        fn len(&self) -> usize {
            self.0
        }
    }

    #[test]
    fn accepts_values_within_bounds() {
        let validator = LenValidation {
            min: 1,
            max: 3,
            actual: None,
        };
        assert!(validator.validate(&"ab".to_string()));
    }

    #[test]
    fn rejects_values_outside_bounds() {
        let validator = LenValidation {
            min: 1,
            max: 3,
            actual: None,
        };
        assert!(!validator.validate(&"".to_string()));
        assert!(!validator.validate(&"abcd".to_string()));
    }

    #[test]
    fn counts_unicode_scalar_values_for_strings() {
        let validator = LenValidation {
            min: 3,
            max: 3,
            actual: None,
        };
        assert!(validator.validate(&"a💀é".to_string()));
        assert!(!validator.validate(&"💀💀💀💀".to_string()));
    }

    #[test]
    fn validates_non_clone_lengths_by_reference() {
        let validator = LenValidation {
            min: 1,
            max: 3,
            actual: None,
        };

        assert!(validator.validate(&NonCloneCollection(2)));
        assert!(!validator.validate(&NonCloneCollection(4)));
    }
}
