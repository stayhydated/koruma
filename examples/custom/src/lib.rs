pub mod i18n;
pub mod validators;

use crate::{
    validators::fluent::{IsEvenNumberValidation, NonEmptyStringValidation},
    validators::normal::{NumberRangeValidation, StringLengthValidation},
};
use koruma::{Koruma, Validate};

#[derive(Koruma)]
pub struct Item {
    #[koruma(NumberRangeValidation<_>(min = 0, max = 100))]
    pub age: i32,

    #[koruma(StringLengthValidation(min = 1, max = 67))]
    pub name: String,

    // This field is not validated
    pub internal_id: u64,
}

/// Example struct using EsFluent-based validators.
#[derive(Koruma)]
pub struct User {
    #[koruma(IsEvenNumberValidation<_>)]
    pub id: i32,

    #[koruma(NonEmptyStringValidation)]
    pub username: String,
}
