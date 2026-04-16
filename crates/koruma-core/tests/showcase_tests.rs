#![cfg(feature = "internal-showcase")]

use koruma_core::showcase::{InputType, validators};

#[test]
fn showcase_registry_access_is_available() {
    let _ = validators();
    assert_eq!(InputType::Text, InputType::Text);
    assert_eq!(InputType::Numeric, InputType::Numeric);
}
