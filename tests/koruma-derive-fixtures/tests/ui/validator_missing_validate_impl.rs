use koruma_derive::{validator, Koruma};

#[validator]
#[derive(Debug)]
pub struct NoValidateValidation {
    #[koruma(value)]
    actual: i32,
}

#[derive(Koruma)]
struct Item {
    #[koruma(NoValidateValidation)]
    value: i32,
}

fn main() {}
