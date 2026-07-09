use koruma_derive::{validator, Koruma};

#[validator]
#[derive(Debug)]
pub struct NoValidateValidation {
    #[koruma(value)]
    actual: i32,
}

#[derive(Koruma)]
struct Item {
    #[koruma(each(NoValidateValidation))]
    values: Vec<i32>,
}

fn main() {}
