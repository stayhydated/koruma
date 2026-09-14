use koruma_derive::{Koruma, KorumaAllDisplay, validator};
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

#[derive(Koruma, KorumaAllDisplay)]
struct Item {
    #[koruma(each(PositiveValidation))]
    values: Vec<i32>,
}

fn main() {}
