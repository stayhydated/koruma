use koruma::{Validate, validator};

/// URL validation for koruma.
///
///
/// # Example
/// ```rust
/// use koruma::Koruma;
/// use koruma_collection::format::UrlValidation;
///
/// #[derive(Koruma)]
/// struct Resource {
///     #[koruma(UrlValidation::<_>)]
///     link: String,
/// }
/// ```
///
/// Validates that a string is a valid URL.
#[validator]
#[cfg_attr(feature = "showcase", showcase(
    name = "URL",
    description = "Validates that the input is a valid URL",
    create = |input: &str| {
        UrlValidation::builder()
            .with_value(input.to_string())
            .build()
    }
))]
#[derive(Clone, Debug)]
#[cfg_attr(feature = "fluent", derive(es_fluent::EsFluent))]
pub struct UrlValidation<T: AsRef<str>> {
    /// The string being validated (stored for error context)
    #[koruma(value)]
    #[cfg_attr(feature = "fluent", fluent(value(|x: &T| x.as_ref().to_string())))]
    pub actual: T,
}

impl<T: AsRef<str>> Validate<T> for UrlValidation<T> {
    fn validate(&self, value: &T) -> bool {
        let s = value.as_ref();
        url::Url::parse(s).is_ok()
    }
}

#[cfg(feature = "fmt")]
impl<T: AsRef<str>> std::fmt::Display for UrlValidation<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "not a valid URL")
    }
}
