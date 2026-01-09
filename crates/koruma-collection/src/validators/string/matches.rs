use koruma::{Validate, validator};

/// Field matching validation for koruma.
///
///
/// # Example
/// ```rust
/// use koruma::Koruma;
/// use koruma_collection::string::MatchesValidation;
///
/// #[derive(Koruma)]
/// struct User {
///     password: String,
///     #[koruma(MatchesValidation::<_>(other = password))]
///     confirm_password: String,
/// }
/// ```
///
/// Validates that a value matches another value.
#[validator]
#[cfg_attr(feature = "showcase", showcase(
    name = "Matches Value",
    description = "Validates that the input matches 'expected'",
    create = |input: &str| {
        MatchesValidation::builder()
            .with_value(input.to_string())
            .other("expected".to_string())
            .build()
    }
))]
#[derive(Clone, Debug)]
#[cfg_attr(feature = "fluent", derive(es_fluent::EsFluent))]
pub struct MatchesValidation<T: PartialEq + std::fmt::Display + Clone> {
    /// The value to match against
    #[cfg_attr(feature = "fluent", fluent(value(|x: &T| x.to_string())))]
    pub other: T,
    /// The value being validated (stored for error context)
    #[koruma(value)]
    #[cfg_attr(feature = "fluent", fluent(value(|x: &T| x.to_string())))]
    pub actual: T,
}

impl<T: PartialEq + std::fmt::Display + Clone> Validate<T> for MatchesValidation<T> {
    fn validate(&self, value: &T) -> bool {
        value == &self.other
    }
}

#[cfg(feature = "fmt")]
impl<T: PartialEq + std::fmt::Debug + std::fmt::Display + Clone> std::fmt::Display
    for MatchesValidation<T>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "value does not match expected value")
    }
}
