use es_fluent::{FluentMessage, FluentMessageLookup};
use koruma_derive::{validator, KorumaAllFluent};
use renamed_koruma::Validate;

#[validator]
#[derive(Debug)]
pub struct PositiveValidation {
    #[koruma(value)]
    actual: i32,
}

impl Validate<i32> for PositiveValidation {
    fn validate(&self, value: &i32) -> bool {
        *value > 0
    }
}

impl FluentMessage for PositiveValidation {
    fn to_fluent_string_with(
        &self,
        _localize: &mut FluentMessageLookup<'_>,
    ) -> String {
        "must be positive".to_string()
    }
}

#[derive(KorumaAllFluent)]
struct Item {
    #[koruma(PositiveValidation)]
    value: i32,
}

fn main() {}
