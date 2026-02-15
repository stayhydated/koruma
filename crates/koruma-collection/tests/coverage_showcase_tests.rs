#![cfg(feature = "internal-showcase")]

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque};

use koruma::showcase::validators;
use koruma_collection::{collection::HasLen, format::IpKind, general::RequiredValidation};

fn showcase_input(name: &str) -> &'static str {
    match name {
        "Length" => "hello",
        "NonEmpty" => "x",
        "Credit Card" => "4111111111111111",
        "Email" => "user@example.com",
        "IP Address" => "127.0.0.1",
        "Phone Number" => "+14155552671",
        "URL" => "https://example.com",
        "Negative Number" => "-1",
        "Non-Negative Number" => "0",
        "Non-Positive Number" => "-1",
        "Positive Number" => "1",
        "Range [0, 100]" => "50",
        "Alphanumeric" => "abc123",
        "ASCII" => "ASCII123",
        "Contains 'test'" => "pretestpost",
        "Matches Value" => "expected",
        "Regex Pattern" => "abc_123",
        "Prefix 'hello'" => "hello_world",
        "Suffix '.rs'" => "lib.rs",
        other => panic!("no showcase input fixture for {other}"),
    }
}

#[test]
fn showcase_validators_can_be_created_and_rendered() {
    let all = validators();
    assert!(!all.is_empty(), "expected internal showcase validators");

    for validator in &all {
        let input = showcase_input(validator.name);
        let instance = (validator.create_validator)(input)
            .unwrap_or_else(|e| panic!("failed to create validator '{}': {e}", validator.name));

        assert!(
            instance.is_valid(),
            "showcase sample input should be valid for '{}'",
            validator.name
        );
        assert!(
            !instance.display_string().is_empty(),
            "display message should not be empty for '{}'",
            validator.name
        );
        assert!(
            !instance.fluent_string().is_empty(),
            "fluent message should not be empty for '{}'",
            validator.name
        );
    }
}

#[test]
fn ip_kind_display_covers_all_variants() {
    assert_eq!(IpKind::Any.to_string(), "IP");
    assert_eq!(IpKind::V4.to_string(), "IPv4");
    assert_eq!(IpKind::V6.to_string(), "IPv6");
}

#[test]
fn required_validation_display_smoke_test() {
    let validator = RequiredValidation::<Option<String>> { actual: None };
    assert_eq!(
        validator.to_string(),
        "This field is required and must not be empty."
    );
}

#[test]
fn has_len_impls_cover_standard_collections() {
    let vec_values = vec![1_u8, 2, 3];
    assert_eq!(HasLen::len(&vec_values), 3);

    let deque_values = VecDeque::from([1_u8, 2, 3, 4]);
    assert_eq!(HasLen::len(&deque_values), 4);

    let mut hash_map = HashMap::new();
    hash_map.insert("a", 1);
    hash_map.insert("b", 2);
    assert_eq!(HasLen::len(&hash_map), 2);

    let mut btree_map = BTreeMap::new();
    btree_map.insert("a", 1);
    assert_eq!(HasLen::len(&btree_map), 1);

    let mut hash_set = HashSet::new();
    hash_set.insert("x");
    hash_set.insert("y");
    assert_eq!(HasLen::len(&hash_set), 2);

    let mut btree_set = BTreeSet::new();
    btree_set.insert("x");
    assert_eq!(HasLen::len(&btree_set), 1);

    let text = String::from("abc");
    assert_eq!(HasLen::len(&text), 3);

    let str_ref = "abcd";
    assert_eq!(HasLen::len(str_ref), 4);

    let slice_ref: &[u8] = &[1, 2, 3, 4, 5];
    assert_eq!(HasLen::len(slice_ref), 5);

    let array_ref = [1_u8, 2, 3, 4];
    assert_eq!(HasLen::len(&array_ref), 4);
}
