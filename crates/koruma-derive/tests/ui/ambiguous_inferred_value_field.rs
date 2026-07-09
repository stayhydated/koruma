use koruma_derive::validator;
use renamed_koruma::Validate;

#[validator]
pub struct AmbiguousValidation {
    actual: i32,
    input: i32,
}

impl Validate<i32> for AmbiguousValidation {
    fn validate(&self, value: &i32) -> bool {
        *value == self.actual
    }
}

fn main() {}
