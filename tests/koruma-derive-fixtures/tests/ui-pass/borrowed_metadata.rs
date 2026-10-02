use koruma_derive::validator;
use renamed_koruma::{ValidatorMetadata as _, ValidatorParamValue};

#[validator]
pub struct BorrowedParameters<'a> {
    enabled: &'a bool,
    signed: &'a &'a i32,
    unsigned: &'a u16,
    ratio: &'a f32,
    optional_enabled: Option<&'a &'a bool>,
    optional_signed: Option<&'a i8>,
    optional_unsigned: Option<&'a &'a u32>,
    optional_ratio: Option<&'a f64>,
    borrowed_optional: &'a Option<&'a bool>,
    text: &'a String,
    bytes: &'a [u8],
    #[koruma(skip_capture)]
    actual: Option<i32>,
}

#[validator]
pub struct GenericParameters<'a, T: ?Sized> {
    parameter: &'a T,
    #[koruma(skip_capture)]
    actual: Option<i32>,
}

pub fn main() {
    let enabled = true;
    let signed = -42;
    let unsigned = 17;
    let ratio = 0.5;
    let optional_signed = -7;
    let optional_unsigned = 23;
    let optional_ratio = 0.25;
    let signed_ref = &signed;
    let enabled_ref = &enabled;
    let unsigned_ref = &optional_unsigned;
    let borrowed_optional = Some(&enabled);
    let text = String::from("configured");
    let bytes = [1, 2, 3];

    let mut validator = BorrowedParameters::enabled(&enabled)
        .signed(&signed_ref)
        .unsigned(&unsigned)
        .ratio(&ratio)
        .optional_enabled(&enabled_ref)
        .optional_signed(&optional_signed)
        .optional_unsigned(&unsigned_ref)
        .optional_ratio(&optional_ratio)
        .borrowed_optional(&borrowed_optional)
        .text(&text)
        .bytes(&bytes)
        .build();
    assert!(validator.actual().is_none());

    let params = validator.validator_params();
    let values: Vec<_> = params.iter().map(|param| param.value()).collect();
    assert_eq!(
        values,
        [
            &ValidatorParamValue::Bool(true),
            &ValidatorParamValue::I64(-42),
            &ValidatorParamValue::U64(17),
            &ValidatorParamValue::F64(0.5),
            &ValidatorParamValue::Bool(true),
            &ValidatorParamValue::I64(-7),
            &ValidatorParamValue::U64(23),
            &ValidatorParamValue::F64(0.25),
            &ValidatorParamValue::Opaque {
                type_name: core::any::type_name::<&Option<&bool>>(),
            },
            &ValidatorParamValue::String(text.clone()),
            &ValidatorParamValue::Opaque {
                type_name: core::any::type_name::<&[u8]>(),
            },
        ]
    );

    validator.optional_enabled = None;
    validator.optional_signed = None;
    validator.optional_unsigned = None;
    validator.optional_ratio = None;
    let params = validator.validator_params();
    assert!(
        params[4..8]
            .iter()
            .all(|param| param.value() == &ValidatorParamValue::None)
    );

    struct NonCloneParameter;
    let parameter = NonCloneParameter;
    let generic = GenericParameters::parameter(&parameter).build();
    assert!(generic.actual().is_none());
    assert_eq!(
        generic.validator_params()[0].value(),
        &ValidatorParamValue::Opaque {
            type_name: core::any::type_name::<&NonCloneParameter>(),
        }
    );
}
