use koruma_derive::{Koruma, validator};
use renamed_koruma::Validate;

#[validator]
#[derive(Debug)]
pub struct NumberRangeValidation<T: Copy + PartialOrd> {
    min: T,
    max: T,
    actual: T,
}

impl<T: Copy + PartialOrd> Validate<T> for NumberRangeValidation<T> {
    fn validate(&self, value: &T) -> bool {
        value >= &self.min && value <= &self.max
    }
}

#[derive(Koruma)]
struct FieldProbe {
    #[koruma(NumberRangeValidation::<_>.min(1).max(5).)]
    score: i32,
}

#[derive(Koruma)]
struct ElementProbe {
    #[koruma(each(NumberRangeValidation::<_>.min(1).max(5).))]
    scores: Vec<i32>,
}

fn main() {
    let _ = FieldProbe { score: 3 }.validate();
    let _ = ElementProbe {
        scores: vec![1, 3, 5],
    }
    .validate();
}
