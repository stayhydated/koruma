pub mod i18n;
pub mod validators;

use crate::{
    validators::fluent::{NonEmptyStringValidation, PositiveNumberValidation},
    validators::normal::{NumberRangeValidation, StringLengthValidation},
};
use es_fluent::EsFluent;
use es_fluent_lang::es_fluent_language;
use koruma::{Koruma, Validate};
use strum::EnumIter;

#[es_fluent_language]
#[derive(Clone, Copy, Debug, EnumIter, EsFluent, PartialEq)]
pub enum Languages {}

impl Languages {
    pub fn next(self) -> Self {
        use strum::IntoEnumIterator as _;
        let all = Self::iter().collect::<Vec<_>>();
        let current_index = all.iter().position(|&l| l == self).unwrap_or(0);
        all[(current_index + 1) % all.len()]
    }
}

#[derive(Koruma)]
pub struct Item {
    #[koruma(NumberRangeValidation(min = 0, max = 100))]
    pub age: i32,

    #[koruma(StringLengthValidation(min = 1, max = 67))]
    pub name: String,

    // This field is not validated
    pub internal_id: u64,
}

/// Example struct using EsFluent-based validators.
#[derive(Koruma)]
pub struct User {
    #[koruma(PositiveNumberValidation)]
    pub id: i32,

    #[koruma(NonEmptyStringValidation)]
    pub username: String,
}
