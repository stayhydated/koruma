pub mod i18n;
pub mod validators;

use crate::{
    validators::fluent::{
        IsEvenNumberValidation, NonEmptyStringValidation, Only67Validation,
        PositiveNumberValidation,
    },
    validators::normal::{NumberRangeValidation, StringLengthValidation, ZipCodeValidation},
};
use koruma::{Koruma, Validate};
use koruma_collection::{collection, general, numeric, string};

#[derive(Koruma)]
pub struct Order {
    #[koruma(each(validators::normal::NumberRangeValidation::<_>::builder().min(1).max(5)))]
    pub quantities: Vec<i32>,
}

#[derive(Koruma)]
pub struct BorrowedOrder<'a> {
    #[koruma(each(validators::normal::NumberRangeValidation::<_>::builder().min(1).max(5)))]
    pub quantities: &'a [i32],
}

#[derive(Koruma, koruma::KorumaAllDisplay)]
pub struct BorrowedUsername<'a> {
    #[koruma(validators::normal::StartsWithValidation::<_>::builder().prefix("user:"))]
    pub username: &'a str,
}

#[derive(Koruma, koruma::KorumaAllDisplay)]
pub struct Item {
    #[koruma(
        validators::normal::NumberRangeValidation::<_>::builder()
            .min(0)
            .max(100)
    )]
    pub age: i32,

    #[koruma(StringLengthValidation::builder().min(1).max(67))]
    pub name: String,

    // This field is not validated
    pub internal_id: u64,
}

/// Example struct using EsFluent-based validators.
#[derive(Koruma)]
pub struct User {
    #[koruma(IsEvenNumberValidation::<_>::builder())]
    pub id: i32,

    #[koruma(NonEmptyStringValidation::builder())]
    pub username: String,
}

// =============================================================================
// Nested Validation Examples (Display-based)
// =============================================================================

/// A nested struct representing a physical address.
/// Uses Display-based validators.
#[derive(Clone, Koruma)]
pub struct Address {
    #[koruma(StringLengthValidation::builder().min(1).max(100))]
    pub street: String,

    #[koruma(StringLengthValidation::builder().min(1).max(50))]
    pub city: String,

    #[koruma(ZipCodeValidation::builder())]
    pub zip_code: String,
}

/// A struct with a nested Address field.
/// Demonstrates `#[koruma(nested)]` for Display-based error messages.
#[derive(Koruma)]
pub struct Customer {
    #[koruma(StringLengthValidation::builder().min(1).max(100))]
    pub name: String,

    #[koruma(NumberRangeValidation::<_>::builder().min(18).max(120))]
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
    #[koruma(PositiveNumberValidation::<_>::builder())]
    pub max_login_attempts: i32,

    #[koruma(NonEmptyStringValidation::builder())]
    pub default_language: String,
}

/// A struct with a nested AccountSettings field.
/// Demonstrates `#[koruma(nested)]` for EsFluent-based error messages.
#[derive(Koruma)]
pub struct Account {
    #[koruma(IsEvenNumberValidation::<_>::builder())]
    pub id: i32,

    #[koruma(NonEmptyStringValidation::builder())]
    pub email: String,

    /// Nested struct - validation cascades automatically
    #[koruma(nested)]
    pub settings: AccountSettings,
}

// =============================================================================
// Newtype Validation Examples
// =============================================================================

/// A newtype wrapper around String, representing an email address.
/// Uses `#[koruma(try_new, newtype)]` to:
/// - validate at construction time (`Email::try_new`)
/// - delegate validation errors directly to the wrapper
#[derive(Clone, Koruma, koruma::KorumaAllFluent)]
#[koruma(try_new, newtype)]
pub struct Email {
    #[koruma(NonEmptyStringValidation::builder())]
    pub value: String,
}

/// A struct using the Email newtype.
/// Demonstrates `#[koruma(newtype)]` on a field to transparently access the inner validation errors.
#[derive(Koruma, koruma::KorumaAllFluent)]
pub struct SignupForm {
    #[koruma(NonEmptyStringValidation::builder())]
    pub username: String,

    /// Newtype field - validation cascades, and errors are treated as if they were on this field
    #[koruma(newtype)]
    pub email: Email,
}

/// An optional newtype field preserves the distinction between "missing" and "invalid".
/// When `email` is `None`, there is no synthetic inner error.
#[derive(Koruma, koruma::KorumaAllFluent)]
pub struct OptionalSignupForm {
    #[koruma(newtype)]
    pub email: Option<Email>,
}

/// An unnamed (tuple) newtype wrapper around String, representing a username.
/// Uses `#[koruma(try_new, newtype)]` with a tuple struct.
/// The field is accessed as `Username::try_new(value).unwrap().0` (tuple index 0).
#[derive(Clone, Koruma, koruma::KorumaAllFluent)]
#[koruma(try_new, newtype)]
pub struct Username(#[koruma(NonEmptyStringValidation::builder())] pub String);

/// A struct using the Username unnamed newtype.
/// Works identically to named newtypes - errors delegate transparently.
#[derive(Koruma, koruma::KorumaAllFluent)]
pub struct LoginForm {
    #[koruma(newtype)]
    pub username: Username,
}

#[derive(Clone, Koruma, koruma::KorumaAllFluent)]
#[koruma(newtype(try_from))]
pub struct Only67u8(#[koruma(Only67Validation::<_>::builder())] pub u8);

#[derive(Koruma, koruma::KorumaAllDisplay)]
pub struct SignupInput {
    #[koruma(collection::NonEmptyValidation::<_>::builder())]
    pub username: String,

    #[koruma(string::AsciiValidation::<_>::builder(), string::AlphanumericValidation::<_>::builder())]
    pub handle: String,

    #[koruma(numeric::RangeValidation::<_>::builder().min(13_u8).max(120_u8))]
    pub age: u8,

    #[koruma(general::RequiredValidation::<Option<_>>::builder())]
    pub display_name: Option<String>,
}
