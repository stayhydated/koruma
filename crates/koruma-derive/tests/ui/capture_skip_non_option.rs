use koruma_derive::validator;

#[derive(Debug)]
struct NonClone;

#[validator]
#[derive(Debug)]
pub struct SkipCaptureValidation {
    #[koruma(value(capture = skip))]
    actual: NonClone,
}

fn main() {}
