use koruma::{Koruma, Validate};

use super::validators::{
    EvenNumberValidation, GenericRangeValidation, NumberRangeValidation, StringLengthValidation,
    VecLenValidation,
};

/// Example struct demonstrating validation with non-generic validators.
#[derive(Koruma)]
pub struct Item {
    #[koruma(NumberRangeValidation(min = 0, max = 100))]
    pub age: i32,

    #[koruma(StringLengthValidation(min = 1, max = 67))]
    pub name: String,

    // This field is not validated
    #[allow(dead_code)]
    pub internal_id: u64,
}

/// Example struct demonstrating validation with generic validators.
/// The type parameter is inferred from the field type using `<_>` syntax!
#[derive(Koruma)]
pub struct GenericItem {
    #[koruma(GenericRangeValidation<_>(min = -10.0, max = 100.0))]
    pub score: f64,

    #[koruma(GenericRangeValidation<_>(min = 0, max = 1000))]
    pub points: u32,
}

/// Example struct demonstrating multiple validators per field.
#[derive(Koruma)]
pub struct MultiValidatorItem {
    // This field must be in range 0-100 AND be even
    #[koruma(NumberRangeValidation(min = 0, max = 100), EvenNumberValidation)]
    pub value: i32,
}

/// Example struct demonstrating collection validation with `each`.
#[derive(Koruma)]
pub struct Order {
    // Each score in the list must be in range 0-100
    #[koruma(each(GenericRangeValidation<_>(min = 0.0, max = 100.0)))]
    pub scores: Vec<f64>,
}

/// Example struct demonstrating optional field validation.
/// Optional fields skip validation when None.
#[derive(Koruma)]
pub struct UserProfile {
    #[koruma(StringLengthValidation(min = 1, max = 50))]
    pub username: String,

    // Optional field - only validated when Some
    #[koruma(StringLengthValidation(min = 1, max = 200))]
    pub bio: Option<String>,

    // Optional field with range validation
    #[koruma(NumberRangeValidation(min = 0, max = 150))]
    pub age: Option<i32>,
}

/// Example struct demonstrating COMBINED collection-level AND per-element validation.
/// The Vec length is validated, AND each element is also validated.
#[derive(Koruma)]
pub struct OrderWithLenCheck {
    // Vec must have 1-5 elements, AND each score must be in range 0-100
    // Note: VecLenValidation requires explicit type parameter - type inference uses the full field type
    #[koruma(VecLenValidation<f64>(min = 1, max = 5), each(GenericRangeValidation<_>(min = 0.0, max = 100.0)))]
    pub scores: Vec<f64>,
}
