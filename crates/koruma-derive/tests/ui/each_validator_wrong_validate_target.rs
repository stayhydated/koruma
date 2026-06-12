use koruma_derive::{validator, Koruma};
use renamed_koruma::Validate;

#[validator]
#[derive(Debug)]
pub struct TextValidation {
    #[koruma(value)]
    actual: i32,
}

impl Validate<String> for TextValidation {
    fn validate(&self, _value: &String) -> bool {
        true
    }
}

#[derive(Koruma)]
struct Item {
    #[koruma(each(TextValidation))]
    values: Vec<i32>,
}

fn main() {}
