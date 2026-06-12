use koruma_derive::{Koruma, validator};
use renamed_koruma::Validate;

const MIN_LEN: usize = 3;

#[validator]
#[derive(Debug)]
pub struct MatchesValidation {
    expected: String,
    actual: Option<String>,
}

impl Validate<String> for MatchesValidation {
    fn validate(&self, value: &String) -> bool {
        value == &self.expected
    }
}

#[validator]
#[derive(Debug)]
pub struct MinLenValidation {
    min: usize,
    actual: Option<String>,
}

impl Validate<String> for MinLenValidation {
    fn validate(&self, value: &String) -> bool {
        value.len() >= self.min
    }
}

#[derive(Koruma)]
struct Demo<const EXTRA_MIN: usize> {
    other: String,
    #[koruma(
        MatchesValidation::expected(self.other.clone()),
        min_const = MinLenValidation::min(MIN_LEN),
        min_const_generic = MinLenValidation::min(EXTRA_MIN),
    )]
    value: String,
}

fn main() {
    let item = Demo::<2> {
        other: "value".to_string(),
        value: "value".to_string(),
    };
    item.validate().unwrap();
}
