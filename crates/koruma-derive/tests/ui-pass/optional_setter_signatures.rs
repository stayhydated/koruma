use koruma_derive::validator;
use renamed_koruma::Validate;

#[validator]
#[derive(Debug)]
pub struct OptionSetterValidation {
    optional_limit: Option<usize>,
    #[koruma(setter(into))]
    optional_text: Option<String>,
    #[koruma(setter(required))]
    required_limit: Option<usize>,
    #[koruma(value)]
    actual: Option<String>,
}

impl Validate<String> for OptionSetterValidation {
    fn validate(&self, value: &String) -> bool {
        self.required_limit.is_some() || value == self.actual.as_ref().unwrap()
    }
}

#[validator]
#[derive(Debug)]
pub struct DefaultedRangeValidation {
    min: usize,
    #[koruma(setter(default = false))]
    exclusive_max: bool,
    #[koruma(value)]
    actual: usize,
}

impl Validate<usize> for DefaultedRangeValidation {
    fn validate(&self, value: &usize) -> bool {
        if self.exclusive_max {
            *value > self.min
        } else {
            *value >= self.min
        }
    }
}

fn main() {
    let _validator = OptionSetterValidation::optional_limit(5)
        .maybe_optional_limit(Some(8))
        .optional_text("configured")
        .maybe_optional_text(Some("configured".to_string()))
        .required_limit(None)
        .with_value("configured".to_string())
        .build();

    let _range = DefaultedRangeValidation::min(10)
        .exclusive_max(true)
        .with_value(11)
        .build();
}
