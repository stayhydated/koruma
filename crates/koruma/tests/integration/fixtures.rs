use koruma::Koruma;

use super::validators::{
    EvenNumberValidation, GenericRangeValidation, MatchesStaticStrValidation,
    MatchesStringValidation, NonCloneRangeValidation, NumberRangeValidation, RequiredValidation,
    StartsWithValidation, StringLengthValidation, VecLenValidation,
};

const STATIC_CONFIRM_SECRET: &str = "shared-secret";

/// Example struct demonstrating validation with non-generic validators.
#[derive(Koruma)]
pub struct Item {
    #[koruma(NumberRangeValidation::min(0).max(100))]
    pub age: i32,

    #[koruma(StringLengthValidation::min(1).max(67))]
    pub name: String,

    // This field is not validated
    #[allow(dead_code)]
    pub internal_id: u64,
}

/// Example struct demonstrating validation with generic validators.
/// The type parameter is inferred from the field type using `<_>` syntax!
#[derive(Koruma)]
pub struct GenericItem {
    #[koruma(GenericRangeValidation::<_>::min(-10.0).max(100.0))]
    pub score: f64,

    #[koruma(GenericRangeValidation::<_>::min(0).max(1000))]
    pub points: u32,
}

/// Example struct demonstrating multiple validators per field.
#[derive(Koruma)]
pub struct MultiValidatorItem {
    // This field must be in range 0-100 AND be even
    #[koruma(NumberRangeValidation::min(0).max(100), EvenNumberValidation)]
    pub value: i32,
}

/// Example struct demonstrating failed-validator inspection without Clone.
#[derive(Koruma, koruma::KorumaAllDisplay)]
pub struct NonCloneValidatorItem {
    #[koruma(NonCloneRangeValidation::min(0).max(100))]
    pub value: i32,
}

/// Example struct demonstrating collection validation with `each`.
#[derive(Koruma)]
pub struct Order {
    // Each score in the list must be in range 0-100
    #[koruma(each(GenericRangeValidation::<_>::min(0.0).max(100.0)))]
    pub scores: Vec<f64>,
}

/// Example struct demonstrating optional collection validation with `each`.
#[derive(Koruma)]
pub struct OptionalOrder {
    // The collection is optional, but each present score still must be in range 0-100.
    #[koruma(each(GenericRangeValidation::<_>::min(0.0).max(100.0)))]
    pub scores: Option<Vec<f64>>,
}

/// Example struct demonstrating that qualified Option/Vec paths keep the same semantics.
#[derive(Koruma)]
pub struct QualifiedPathProfile {
    #[koruma(StringLengthValidation::min(1).max(200))]
    pub bio: std::option::Option<String>,

    #[koruma(each(GenericRangeValidation::<_>::min(0.0).max(100.0)))]
    pub scores: core::option::Option<std::vec::Vec<f64>>,
}

/// Example struct demonstrating borrowed slice validation with `each`.
#[derive(Koruma)]
pub struct BorrowedOrder<'a> {
    #[koruma(each(GenericRangeValidation::<_>::min(0.0).max(100.0)))]
    pub scores: &'a [f64],
}

/// Example struct demonstrating optional borrowed slice validation with `each`.
#[derive(Koruma)]
pub struct OptionalBorrowedOrder<'a> {
    #[koruma(each(GenericRangeValidation::<_>::min(0.0).max(100.0)))]
    pub scores: Option<&'a [f64]>,
}

/// Example struct demonstrating array validation with `each`.
#[derive(Koruma)]
pub struct ArrayOrder {
    #[koruma(each(GenericRangeValidation::<_>::min(0).max(100)))]
    pub scores: [i32; 3],
}

/// Example struct demonstrating borrowed direct-field validation.
#[derive(Koruma, koruma::KorumaAllDisplay)]
pub struct BorrowedUsername<'a> {
    #[koruma(StartsWithValidation::<_>::prefix("user:"))]
    pub username: &'a str,
}

/// Example struct demonstrating explicit reference-wrapper type inference.
#[derive(Koruma, koruma::KorumaAllDisplay)]
pub struct BorrowedUsernameExplicitInfer<'a> {
    #[koruma(StartsWithValidation::<&_>::prefix("user:"))]
    pub username: &'a str,
}

/// Example struct demonstrating borrowed string element validation with `each`.
#[derive(Koruma, koruma::KorumaAllDisplay)]
pub struct BorrowedTags<'a> {
    #[koruma(each(StartsWithValidation::<_>::prefix("tag:")))]
    pub tags: &'a [&'a str],
}

/// Example struct demonstrating explicit cross-field setter arguments.
#[derive(Koruma)]
pub struct PasswordConfirmation {
    pub password: String,

    #[koruma(MatchesStringValidation::expected(self.password.clone()))]
    pub confirm: String,
}

/// Example struct demonstrating the Rust-native direct-chain validator syntax.
#[derive(Koruma)]
pub struct DirectSyntaxItem {
    #[koruma(GenericRangeValidation::<_>::min(-10.0).max(100.0))]
    pub score: f64,
}

/// Example struct demonstrating explicit cross-field references inside direct-chain syntax.
#[derive(Koruma)]
pub struct DirectPasswordConfirmation {
    pub password: String,

    #[koruma(MatchesStringValidation::expected(self.password.clone()))]
    pub confirm: String,
}

/// Example struct demonstrating that bare identifiers which are not fields remain untouched.
#[derive(Koruma)]
pub struct StaticSecretConfirmation {
    #[koruma(MatchesStaticStrValidation::expected(STATIC_CONFIRM_SECRET))]
    pub confirm: String,
}

/// Example struct demonstrating optional field validation.
/// Optional fields skip validation when None.
#[derive(Koruma)]
pub struct UserProfile {
    #[koruma(StringLengthValidation::min(1).max(50))]
    pub username: String,

    // Optional field - only validated when Some
    #[koruma(StringLengthValidation::min(1).max(200))]
    pub bio: Option<String>,

    // Optional field with range validation
    #[koruma(NumberRangeValidation::min(0).max(150))]
    pub age: Option<i32>,
}

/// Example payload without Clone to exercise skip-capture validators.
pub struct NonCloneSecret {
    pub raw: String,
}

/// Example struct demonstrating required optional-field validation.
#[derive(Koruma)]
pub struct ExplicitRequiredProfile {
    #[koruma(full(RequiredValidation::<_>))]
    pub bio: Option<String>,
}

/// Example struct demonstrating required optional-element validation.
#[derive(Koruma)]
pub struct OptionalElementPresenceOrder {
    #[koruma(each(full(RequiredValidation::<_>)))]
    pub values: Vec<Option<i32>>,
}

/// Example struct demonstrating mixed full-type and unwrapped element validators.
#[derive(Koruma)]
pub struct OptionalElementMixedValidators {
    #[koruma(each(full(RequiredValidation::<_>), GenericRangeValidation::<_>::min(0).max(10)))]
    pub values: Vec<Option<i32>>,
}

/// Example struct demonstrating direct element validation for required elements.
#[derive(Koruma)]
pub struct RequiredElementFullTypeOrder {
    #[koruma(each(GenericRangeValidation::<_>::min(0).max(10)))]
    pub values: Vec<i32>,
}

/// Example struct showing that RequiredValidation can validate Option<NonClone>
/// without forcing clone capture in the derive expansion.
#[derive(Koruma)]
pub struct PresenceOnlyNonClone {
    #[koruma(full(RequiredValidation::<_>))]
    pub token: Option<NonCloneSecret>,
}

/// Example struct demonstrating COMBINED collection-level AND per-element validation.
/// The Vec length is validated, AND each element is also validated.
#[derive(Koruma)]
pub struct OrderWithLenCheck {
    // Vec must have 1-5 elements, AND each score must be in range 0-100
    // Note: VecLenValidation<T> expects T to be the inner type (f64), not Vec<f64>.
    // Use explicit type when the validator's generic param differs from the field type.
    #[koruma(VecLenValidation::<f64>::min(1).max(5), each(GenericRangeValidation::<_>::min(0.0).max(100.0)))]
    pub scores: Vec<f64>,
}

/// Example struct demonstrating nested validation.
/// Address is a nested struct that also derives Koruma.
#[derive(Clone, Koruma)]
pub struct Address {
    #[koruma(StringLengthValidation::min(1).max(100))]
    pub street: String,

    #[koruma(StringLengthValidation::min(1).max(50))]
    pub city: String,

    #[koruma(StringLengthValidation::min(2).max(10))]
    pub zip_code: String,
}

/// Example struct with a nested Koruma struct.
#[derive(Koruma)]
pub struct Customer {
    #[koruma(StringLengthValidation::min(1).max(100))]
    pub name: String,

    // Nested struct - will call Address::validate() automatically
    #[koruma(nested)]
    pub address: Address,
}

/// Example struct with an optional nested Koruma struct.
#[derive(Koruma)]
pub struct CustomerWithOptionalAddress {
    #[koruma(StringLengthValidation::min(1).max(100))]
    pub name: String,

    // Optional nested struct - skipped when None, validated when Some
    #[koruma(nested)]
    pub shipping_address: Option<Address>,
}

/// Example struct with deeply nested validation (nested within nested).
#[derive(Clone, Koruma)]
pub struct Company {
    #[koruma(StringLengthValidation::min(1).max(200))]
    pub company_name: String,

    #[koruma(nested)]
    pub headquarters: Address,
}

/// Example struct with multiple levels of nesting.
#[derive(Koruma)]
pub struct Employee {
    #[koruma(StringLengthValidation::min(1).max(100))]
    pub employee_name: String,

    #[koruma(nested)]
    pub employer: Company,
}

/// Example newtype struct with validators on the inner field.
/// The `newtype` option allows the error struct to deref to the inner field's error.
#[derive(Clone, Debug, Koruma)]
#[koruma(newtype)]
pub struct PositiveNumber {
    #[koruma(NumberRangeValidation::min(0).max(1000))]
    pub value: i32,
}

/// Example newtype struct wrapping a nested Koruma type.
/// The error struct derefs to the inner type's error struct.
#[derive(Koruma)]
#[koruma(newtype)]
pub struct AddressWrapper {
    #[koruma(nested)]
    pub inner: Address,
}

/// Example struct containing a field-level newtype.
/// The field uses `#[koruma(newtype)]` for transparent error access.
#[derive(Koruma)]
pub struct ContainsNewtype {
    #[koruma(StringLengthValidation::min(1).max(100))]
    pub name: String,

    /// This field is a newtype - errors deref to the inner type's errors
    #[koruma(newtype)]
    pub number: PositiveNumber,
}

/// Example struct containing an optional newtype field with RequiredValidation.
/// This tests the new functionality where newtype fields can have additional validators.
#[derive(Koruma)]
pub struct ContainsRequiredNewtype {
    #[koruma(newtype, full(RequiredValidation::<_>))]
    pub age: Option<PositiveNumber>,
}

/// Example struct containing an optional newtype field with multiple validators.
#[derive(Koruma)]
pub struct ContainsNewtypeWithValidators {
    #[koruma(newtype, full(RequiredValidation::<_>))]
    pub age: Option<PositiveNumber>,
    #[allow(dead_code)]
    pub name: String,
}
