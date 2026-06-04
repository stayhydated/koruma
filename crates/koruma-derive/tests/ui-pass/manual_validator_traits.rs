use koruma_derive::Koruma;
use renamed_koruma::{__private, Validate};

#[derive(Debug)]
struct ManualValidation {
    actual: i32,
}

struct ManualValidationBuilder;

impl ManualValidation {
    fn __koruma_builder() -> ManualValidationBuilder {
        ManualValidationBuilder
    }
}

impl __private::CaptureValueRef<i32> for ManualValidationBuilder {
    type Output = ManualValidation;

    fn capture_value_ref(self, value: &i32) -> Self::Output {
        ManualValidation { actual: *value }
    }
}

impl __private::BuildValidator for ManualValidation {
    type Validator = ManualValidation;

    fn build_validator(self) -> Self::Validator {
        self
    }
}

impl Validate<i32> for ManualValidation {
    fn validate(&self, value: &i32) -> bool {
        *value == self.actual
    }
}

#[derive(Koruma)]
struct Item {
    #[koruma(ManualValidation)]
    value: i32,
}

fn main() {
    let _ = Item { value: 5 }.validate();
}
