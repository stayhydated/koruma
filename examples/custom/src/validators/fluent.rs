use es_fluent::EsFluent;
use koruma::{KorumaResult, Validate, validator};

/// A validation rule that checks if a number is positive.
/// Uses `EsFluent` for internationalized error messages.
#[validator]
#[derive(Clone, Debug, EsFluent)]
pub struct PositiveNumberValidation {
    #[koruma(value)]
    pub actual: i32,
}

impl Validate<i32> for PositiveNumberValidation {
    fn validate(&self, value: &i32) -> KorumaResult {
        if *value <= 0 { Err(()) } else { Ok(()) }
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
    fn validate(&self, value: &String) -> KorumaResult {
        if value.is_empty() { Err(()) } else { Ok(()) }
    }
}
