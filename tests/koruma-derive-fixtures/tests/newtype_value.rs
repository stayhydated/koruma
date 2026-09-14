use koruma_derive::{Koruma, validator};
use renamed_koruma::{NewtypeTryFromInner, NewtypeValue, Validate, ValidationError as _};

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

#[derive(Debug, Koruma)]
#[koruma(newtype)]
struct PrivateTuple(#[koruma(PositiveValidation)] i32);

#[derive(Debug, Koruma)]
#[koruma(newtype)]
struct PrivateNamed {
    #[koruma(PositiveValidation)]
    value: i32,
}

fn assert_newtype_traits<T>()
where
    T: NewtypeValue + NewtypeTryFromInner,
{
    renamed_koruma::__private::assert_newtype_value::<T>();
    renamed_koruma::__private::assert_newtype_try_from_inner::<T>();
}

#[test]
fn tuple_newtype_exposes_private_inner_value_contract() {
    assert_newtype_traits::<PrivateTuple>();

    let value = PrivateTuple::try_from_inner(3).expect("positive value should be valid");
    assert_eq!(*value.as_inner(), 3);
    assert_eq!(value.into_inner(), 3);

    let err = PrivateTuple::validate_inner(&0).expect_err("zero should fail validation");
    assert!(err.has_errors());

    let err = PrivateTuple::try_from_inner(-1).expect_err("negative value should fail validation");
    assert!(err.has_errors());
}

#[test]
fn named_newtype_exposes_private_inner_value_contract() {
    assert_newtype_traits::<PrivateNamed>();

    let value = PrivateNamed::try_from_inner(2).expect("positive value should be valid");
    assert_eq!(*value.as_inner(), 2);
    assert_eq!(value.into_inner(), 2);

    let err = PrivateNamed::validate_inner(&0).expect_err("zero should fail validation");
    assert!(err.has_errors());

    let err = PrivateNamed::try_from_inner(-1).expect_err("negative value should fail validation");
    assert!(err.has_errors());
}
