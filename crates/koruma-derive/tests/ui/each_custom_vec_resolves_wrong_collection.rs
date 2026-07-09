use koruma_derive::{Koruma, validator};
use renamed_koruma::Validate;

mod custom {
    pub struct Vec<T>(std::vec::Vec<T>);

    impl<T> Vec<T> {
        pub fn iter(&self) -> std::slice::Iter<'_, T> {
            self.0.iter()
        }
    }
}

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
    #[koruma(each(PositiveValidation))]
    values: custom::Vec<i32>,
}

fn main() {}
