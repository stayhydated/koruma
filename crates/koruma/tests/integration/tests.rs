//! Test cases for koruma validation.

use koruma::{
    Validate as _, ValidationError as _, ValidationFieldName, ValidationIssueScope,
    ValidationIssues as _, ValidatorMetadata, ValidatorParamValue,
};

use super::fixtures::{
    Address, AddressWrapper, ArrayOrder, BorrowedOrder, BorrowedTags, BorrowedUsername,
    BorrowedUsernameExplicitInfer, Company, ContainsNewtype, Customer, CustomerWithOptionalAddress,
    DirectPasswordConfirmation, DirectSyntaxItem, Employee, ExplicitRequiredProfile, GenericItem,
    Item, MultiValidatorItem, NonCloneSecret, NonCloneValidatorItem, OptionalBorrowedOrder,
    OptionalElementMixedValidators, OptionalElementPresenceOrder, OptionalOrder, Order,
    OrderWithLenCheck, PasswordConfirmation, PositiveNumber, PresenceOnlyNonClone,
    QualifiedPathProfile, RequiredElementFullTypeOrder, StaticSecretConfirmation, UserProfile,
};
use super::validators::{GenericRangeValidation, NumberRangeValidation, PrefixBytesValidation};

include!("tests/basic.rs");
include!("tests/generic.rs");
include!("tests/metadata_issues.rs");
include!("tests/collections.rs");
include!("tests/borrowing.rs");
include!("tests/optional_full_targets.rs");
include!("tests/nesting.rs");
include!("tests/newtypes.rs");
