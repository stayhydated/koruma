use koruma_derive::{Koruma, validator};
use renamed_koruma::Validate;

#[derive(Debug)]
struct NonClone;

#[validator]
#[derive(Debug)]
pub struct NonCloneValidation {
    #[koruma(value)]
    actual: NonClone,
}

impl Validate<NonClone> for NonCloneValidation {
    fn validate(&self, _value: &NonClone) -> bool {
        true
    }
}

#[derive(Koruma)]
struct Demo {
    #[koruma(NonCloneValidation)]
    value: NonClone,
}

fn main() {
    let _ = Demo { value: NonClone }.validate();
}
