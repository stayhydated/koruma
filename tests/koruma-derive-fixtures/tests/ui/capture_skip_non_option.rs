use koruma_derive::validator;

#[derive(Debug)]
struct NonClone;

#[validator]
#[derive(Debug)]
pub struct SkipCaptureValidation {
    #[koruma(skip_capture)]
    actual: NonClone,
}

fn main() {}
