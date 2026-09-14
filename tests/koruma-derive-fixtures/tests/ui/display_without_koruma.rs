use koruma_derive::{validator, KorumaAllDisplay};
use renamed_koruma::Validate;
use std::fmt;

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

impl fmt::Display for PositiveValidation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("must be positive")
    }
}

#[derive(KorumaAllDisplay)]
struct Item {
    #[koruma(PositiveValidation)]
    value: i32,
}

fn main() {}
