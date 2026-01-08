use es_fluent::EsFluent;
use koruma::{Validate, validator};

/// A validation rule that checks if a number is positive.
/// Uses `EsFluent` for internationalized error messages.
#[validator]
#[derive(Clone, Debug, EsFluent)]
pub struct IsEvenNumberValidation<
    T: Clone + Copy + std::fmt::Display + std::ops::Rem<Output = T> + From<u8> + PartialEq,
> {
    #[koruma(value)]
    #[fluent(value(|x: &T| x.to_string()))]
    pub actual: T,
}

impl<T: Copy + std::fmt::Display + std::ops::Rem<Output = T> + From<u8> + PartialEq> Validate<T>
    for IsEvenNumberValidation<T>
{
    fn validate(&self, value: &T) -> bool {
        *value % T::from(2u8) == T::from(0u8)
    }
}

/// A validation rule that checks if a string is non-empty.
/// Uses `EsFluent` for internationalized error messages.
#[validator]
#[derive(Clone, Debug, EsFluent)]
pub struct NonEmptyStringValidation {
    #[koruma(value)]
    pub input: String,
}

impl Validate<String> for NonEmptyStringValidation {
    fn validate(&self, value: &String) -> bool {
        !value.is_empty()
    }
}

/// A validation rule that checks if a number is positive.
/// Uses `EsFluent` for internationalized error messages.
#[validator]
#[derive(Clone, Debug, EsFluent)]
pub struct PositiveNumberValidation<T: Clone + Copy + std::fmt::Display + PartialOrd + Default> {
    #[koruma(value)]
    #[fluent(value(|x: &T| x.to_string()))]
    pub actual: T,
}

impl<T: Copy + std::fmt::Display + PartialOrd + Default> Validate<T>
    for PositiveNumberValidation<T>
{
    fn validate(&self, value: &T) -> bool {
        *value > T::default()
    }
}
