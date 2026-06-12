use koruma_derive::{Koruma, validator};
use renamed_koruma::Validate;

#[validator]
#[derive(Debug)]
pub struct PositiveValidation {
    actual: i32,
}

impl Validate<i32> for PositiveValidation {
    fn validate(&self, value: &i32) -> bool {
        *value > 0
    }
}

#[validator]
#[derive(Debug)]
pub struct LenValidation {
    min: usize,
    input: String,
}

impl Validate<String> for LenValidation {
    fn validate(&self, value: &String) -> bool {
        value.len() >= self.min
    }
}

#[validator]
#[derive(Debug)]
pub struct CandidateValidation {
    candidate: i32,
}

impl Validate<i32> for CandidateValidation {
    fn validate(&self, value: &i32) -> bool {
        *value == self.candidate
    }
}

#[validator]
#[derive(Debug)]
pub struct MinScoreValidation {
    #[koruma(setter)]
    lower_bound: i32,
    candidate: i32,
}

impl Validate<i32> for MinScoreValidation {
    fn validate(&self, value: &i32) -> bool {
        *value >= self.lower_bound
    }
}

#[derive(Koruma)]
struct Item {
    #[koruma(PositiveValidation)]
    count: i32,
    #[koruma(LenValidation::min(3))]
    label: String,
    #[koruma(CandidateValidation)]
    lucky_number: i32,
    #[koruma(MinScoreValidation::lower_bound(10))]
    score: i32,
}

fn main() {
    let _ = Item {
        count: 1,
        label: "abc".to_string(),
        lucky_number: 7,
        score: 10,
    }
    .validate();
}
