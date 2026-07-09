use koruma_derive::{Koruma, validator};
use renamed_koruma::Validate;

#[derive(Debug)]
struct NonClone;

#[validator]
#[derive(Debug)]
pub struct SkipCaptureValidation {
    #[koruma(skip_capture)]
    actual: Option<NonClone>,
}

impl Validate<NonClone> for SkipCaptureValidation {
    fn validate(&self, _value: &NonClone) -> bool {
        true
    }
}

#[derive(Koruma)]
struct Demo {
    #[koruma(SkipCaptureValidation)]
    value: NonClone,
}

fn main() {
    let _ = Demo { value: NonClone }.validate();
}
