use koruma::{Validate, validator};

/// Suffix validation for koruma.
///
///
/// # Example
/// ```rust
/// use koruma::Koruma;
/// use koruma_collection::string::SuffixValidation;
///
/// #[derive(Koruma)]
/// struct File {
///     #[koruma(SuffixValidation::<_>(suffix = ".txt"))]
///     name: String,
/// }
/// ```
///
/// Validates that a string ends with a specified suffix.
#[validator]
#[cfg_attr(feature = "showcase", showcase(
    name = "Suffix '.rs'",
    description = "Validates that the input ends with '.rs'",
    create = |input: &str| {
        SuffixValidation::builder()
            .suffix(".rs")
            .with_value(input.to_string())
            .build()
    }
))]
#[derive(Clone, Debug)]
#[cfg_attr(feature = "fluent", derive(es_fluent::EsFluent))]
pub struct SuffixValidation<T: AsRef<str>> {
    /// The suffix to check for
    #[builder(into)]
    pub suffix: String,
    /// The string being validated (stored for error context)
    #[koruma(value)]
    #[cfg_attr(feature = "fluent", fluent(value(|x: &T| x.as_ref().to_string())))]
    pub actual: T,
}

impl<T: AsRef<str>> Validate<T> for SuffixValidation<T> {
    fn validate(&self, value: &T) -> bool {
        let s = value.as_ref();
        s.ends_with(&self.suffix)
    }
}

#[cfg(feature = "fmt")]
impl<T: AsRef<str>> std::fmt::Display for SuffixValidation<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "value does not end with \"{}\"", self.suffix)
    }
}
