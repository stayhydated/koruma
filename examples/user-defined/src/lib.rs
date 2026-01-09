pub mod i18n;
pub mod validators;

use crate::{
    validators::fluent::{
        IsEvenNumberValidation, NonEmptyStringValidation, PositiveNumberValidation,
    },
    validators::normal::{NumberRangeValidation, StringLengthValidation, ZipCodeValidation},
};
use koruma::{Koruma, Validate};

// #[derive(Koruma)]
// struct Order {
//     #[koruma(LenValidation::<_>(min = 1, max = 5))]
//     items: Vec<String>,
// }

#[derive(Koruma)]
pub struct Item {
    #[koruma(NumberRangeValidation::<_>(min = 0, max = 100))]
    pub age: i32,

    #[koruma(StringLengthValidation(min = 1, max = 67))]
    pub name: String,

    // This field is not validated
    pub internal_id: u64,
}

/// Example struct using EsFluent-based validators.
#[derive(Koruma)]
pub struct User {
    #[koruma(IsEvenNumberValidation::<_>)]
    pub id: i32,

    #[koruma(NonEmptyStringValidation)]
    pub username: String,
}

// =============================================================================
// Nested Validation Examples (Display-based)
// =============================================================================

/// A nested struct representing a physical address.
/// Uses Display-based validators.
#[derive(Clone, Koruma)]
pub struct Address {
    #[koruma(StringLengthValidation(min = 1, max = 100))]
    pub street: String,

    #[koruma(StringLengthValidation(min = 1, max = 50))]
    pub city: String,

    #[koruma(ZipCodeValidation)]
    pub zip_code: String,
}

/// A struct with a nested Address field.
/// Demonstrates `#[koruma(nested)]` for Display-based error messages.
#[derive(Koruma)]
pub struct Customer {
    #[koruma(StringLengthValidation(min = 1, max = 100))]
    pub name: String,

    #[koruma(NumberRangeValidation::<_>(min = 18, max = 120))]
    pub age: i32,

    /// Nested struct - validation cascades automatically
    #[koruma(nested)]
    pub address: Address,
}

// =============================================================================
// Nested Validation Examples (EsFluent-based)
// =============================================================================

/// A nested struct representing account settings.
/// Uses EsFluent-based validators for i18n support.
#[derive(Clone, Koruma)]
pub struct AccountSettings {
    #[koruma(PositiveNumberValidation::<_>)]
    pub max_login_attempts: i32,

    #[koruma(NonEmptyStringValidation)]
    pub default_language: String,
}

/// A struct with a nested AccountSettings field.
/// Demonstrates `#[koruma(nested)]` for EsFluent-based error messages.
#[derive(Koruma)]
pub struct Account {
    #[koruma(IsEvenNumberValidation::<_>)]
    pub id: i32,

    #[koruma(NonEmptyStringValidation)]
    pub email: String,

    /// Nested struct - validation cascades automatically
    #[koruma(nested)]
    pub settings: AccountSettings,
}
