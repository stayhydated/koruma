use koruma_derive::{Koruma, validator};
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
struct Item {
    #[koruma(PositiveValidation)]
    value: i32,
}

fn main() {
    let err = Item { value: -1 }.validate().unwrap_err();
    assert!(err.value().positive_validation().is_some());
}
