use es_fluent::EsFluent;
use koruma_derive::{validator, Koruma, KorumaAllFluent};
use renamed_koruma::Validate;

#[validator]
#[derive(Debug, EsFluent)]
pub struct PositiveValidation {
    #[koruma(value)]
    actual: i32,
}

impl Validate<i32> for PositiveValidation {
    fn validate(&self, value: &i32) -> bool {
        *value > 0
    }
}

#[derive(Koruma, KorumaAllFluent)]
struct Item {
    #[koruma(PositiveValidation)]
    value: i32,
}

fn main() {
    let _ = Item { value: 1 }.validate();
}
