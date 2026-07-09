use koruma_derive::{validator, Koruma, KorumaAllDisplay};
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
    #[koruma(PositiveValidation)]
    value: i32,
}

fn main() {}
