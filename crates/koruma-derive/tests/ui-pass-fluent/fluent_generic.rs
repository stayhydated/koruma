use es_fluent::registry::{StaticFluentDomain, StaticFluentEntryId};
use es_fluent::{FluentArgs, FluentMessage};
use koruma_derive::{validator, Koruma, KorumaAllFluent};
use renamed_koruma::Validate;

#[validator]
#[derive(Debug)]
pub struct PositiveValidation<T> {
    #[koruma(value)]
    actual: T,
}

impl Validate<i32> for PositiveValidation<i32> {
    fn validate(&self, value: &i32) -> bool {
        *value > 0
    }
}

impl FluentMessage for PositiveValidation<i32> {
    fn to_fluent_string_with(
        &self,
        _localize: &mut dyn for<'a> FnMut(
            StaticFluentDomain,
            StaticFluentEntryId,
            Option<&FluentArgs<'a>>,
        ) -> String,
    ) -> String {
        "must be positive".to_string()
    }
}

#[derive(Koruma, KorumaAllFluent)]
struct Item {
    #[koruma(PositiveValidation::<_>)]
    value: i32,
}

fn main() {
    let _ = Item { value: 1 }.validate();
}
