use koruma_derive::{validator, Koruma};
use renamed_koruma::Validate;

#[validator]
#[derive(Debug)]
pub struct RequiredValidation<T> {
    actual: T,
}

impl<T> Validate<Option<T>> for RequiredValidation<Option<T>> {
    fn validate(&self, value: &Option<T>) -> bool {
        value.is_some()
    }
}

#[validator]
#[derive(Debug)]
pub struct PositiveValidation<T> {
    actual: T,
}

impl Validate<i32> for PositiveValidation<i32> {
    fn validate(&self, value: &i32) -> bool {
        *value > 0
    }
}

#[derive(Koruma)]
struct Item {
    #[koruma(RequiredValidation::<Option<_>>, unwrapped(PositiveValidation::<_>))]
    value: Option<i32>,
}

fn main() {
    let _ = Item { value: Some(1) }.validate();
}
