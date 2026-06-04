use koruma_derive::{validator, Koruma};
use renamed_koruma::Validate;

#[validator]
#[derive(Debug)]
pub struct RequiredSetterValidation {
    #[koruma(setter(required))]
    min: usize,
    #[koruma(value)]
    actual: usize,
}

impl Validate<usize> for RequiredSetterValidation {
    fn validate(&self, value: &usize) -> bool {
        *value >= self.min
    }
}

#[derive(Koruma)]
struct Item {
    #[koruma(RequiredSetterValidation)]
    value: usize,
}

fn main() {
    let _ = Item { value: 10 }.validate();
}
