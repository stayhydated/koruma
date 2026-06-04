use koruma_derive::{validator, Koruma};
use renamed_koruma::Validate;

#[validator]
#[derive(Debug)]
pub struct PositiveValidation {
    #[koruma(value)]
    actual: i32,
}

impl Validate<i32> for PositiveValidation {
    fn validate(&self, value: &i32) -> bool {
        *value > 0
    }
}

#[derive(Koruma)]
struct Child {
    #[koruma(PositiveValidation)]
    value: i32,
}

#[derive(Koruma)]
struct Parent {
    #[koruma(newtype)]
    child: Child,
}

fn main() {}
