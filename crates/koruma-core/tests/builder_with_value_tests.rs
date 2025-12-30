//! Tests for the BuilderWithValue trait.

use koruma_core::BuilderWithValue;

struct TestBuilder {
    min: i32,
    max: i32,
    value: Option<i32>,
}

impl BuilderWithValue<i32> for TestBuilder {
    fn with_value(mut self, value: i32) -> Self {
        self.value = Some(value);
        self
    }
}

#[test]
fn test_builder_with_value() {
    let builder = TestBuilder {
        min: 0,
        max: 100,
        value: None,
    };

    let builder_with_value = builder.with_value(42);
    assert_eq!(builder_with_value.value, Some(42));
}

#[test]
fn test_builder_with_value_chaining() {
    let builder = TestBuilder {
        min: 0,
        max: 100,
        value: None,
    }
    .with_value(99);

    assert_eq!(builder.value, Some(99));
    assert_eq!(builder.min, 0);
    assert_eq!(builder.max, 100);
}
