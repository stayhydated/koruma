use koruma::Validate as _;
use koruma_collection::collection::{HasLen, LenValidation};
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig { cases: 128, ..ProptestConfig::default() })]

    #[test]
    fn unicode_lengths_use_the_generated_scalar_count(
        scalars in prop::collection::vec(any::<char>(), 0..128),
        min in 0_usize..=129,
        max in 0_usize..=129,
    ) {
        // The source vector supplies the oracle, without counting the resulting string again.
        let expected_len = scalars.len();
        let value: String = scalars.into_iter().collect();
        let borrowed = value.as_str();
        prop_assert_eq!(HasLen::len(&value), expected_len);
        prop_assert_eq!(HasLen::len(borrowed), expected_len);
        prop_assert_eq!(HasLen::len(&borrowed), expected_len);

        let validator = LenValidation::<String>::min(min).max(max).build();
        let borrowed_validator = LenValidation::<&str>::min(min).max(max).build();
        let expected = (min..=max).contains(&expected_len);
        prop_assert_eq!(validator.validate(&value), expected);
        prop_assert_eq!(borrowed_validator.validate(&borrowed), expected);
    }
}

#[test]
fn length_counts_combining_marks_and_non_bmp_scalars_individually() {
    let value = "e\u{301}💀".to_string();
    let exactly_three = LenValidation::<String>::min(3).max(3).build();
    let reversed = LenValidation::<String>::min(4).max(2).build();
    assert_eq!(HasLen::len(&value), 3);
    assert!(exactly_three.validate(&value));
    assert!(!reversed.validate(&value));
}
