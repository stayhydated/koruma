use proptest::prelude::*;

fn optional_element() -> impl Strategy<Value = Option<i32>> {
    // Construct each semantic class; every shrink remains a meaningful Option value.
    prop_oneof![
        Just(None),
        (0_i32..=10).prop_map(Some),
        (-100_i32..0).prop_map(Some),
        (11_i32..=100).prop_map(Some),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 128, ..ProptestConfig::default() })]

    #[test]
    fn optional_each_errors_and_issues_match_independent_element_model(
        values in prop_oneof![
            prop::collection::vec((0_i32..=10).prop_map(Some), 0..128),
            prop::collection::vec(optional_element(), 0..128),
        ],
    ) {
        // Recompute indices after shrinking the vector, rather than shrinking stale indices.
        let expected: Vec<_> = values.iter().enumerate()
            .filter_map(|(index, value)| match value {
                Some(0..=10) => None,
                _ => Some(index),
            })
            .collect();
        let order = OptionalElementMixedValidators { values };

        match order.validate() {
            Ok(()) => prop_assert!(expected.is_empty()),
            Err(error) => {
                prop_assert!(!expected.is_empty());
                prop_assert!(!error.is_empty());
                let element_errors = error.values().element_errors();
                let actual: Vec<_> = element_errors.iter().map(|(index, _)| *index).collect();
                prop_assert_eq!(&actual, &expected);

                for (index, element_error) in element_errors {
                    let value = order.values[*index];
                    prop_assert_eq!(element_error.required_validation().is_some(), value.is_none());
                    prop_assert_eq!(element_error.generic_range_validation().is_some(), value.is_some());
                    prop_assert_eq!(element_error.all().count(), 1);
                    if let Some(range_error) = element_error.generic_range_validation() {
                        prop_assert_eq!(Some(*range_error.actual()), value);
                    }
                }

                let issues = error.issues();
                prop_assert_eq!(issues.len(), expected.len());
                for (issue, index) in issues.iter().zip(expected) {
                    prop_assert_eq!(issue.field_name_str(), Some("values"));
                    prop_assert_eq!(issue.scope(), ValidationIssueScope::Element);
                    prop_assert_eq!(issue.element_index(), Some(index));
                }
            }
        }
    }
}

#[test]
fn optional_each_distinguishes_absent_invalid_and_valid_elements() {
    let order = OptionalElementMixedValidators {
        values: vec![Some(0), None, Some(11), Some(10), Some(-1)],
    };
    let error = order.validate().unwrap_err();
    let indices: Vec<_> = error.issues().iter().map(|issue| issue.element_index()).collect();
    assert_eq!(indices, [Some(1), Some(2), Some(4)]);
    assert!(OptionalElementMixedValidators { values: vec![] }.validate().is_ok());
}
