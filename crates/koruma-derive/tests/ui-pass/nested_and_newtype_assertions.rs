use koruma_derive::{validator, Koruma};
use renamed_koruma::Validate;

#[validator]
#[derive(Debug)]
pub struct PositiveValidation {
    actual: i32,
}

impl Validate<i32> for PositiveValidation {
    fn validate(&self, value: &i32) -> bool {
        *value > 0
    }
}

#[derive(Koruma)]
struct Child {
    #[koruma(PositiveValidation)]
    value: i32,
}

#[derive(Koruma)]
#[koruma(newtype)]
pub struct ChildId {
    #[koruma(PositiveValidation)]
    value: i32,
}

#[derive(Koruma)]
struct Parent {
    #[koruma(nested)]
    required_child: Child,
    #[koruma(nested)]
    optional_child: Option<Child>,
    #[koruma(newtype)]
    required_id: ChildId,
    #[koruma(newtype)]
    optional_id: Option<ChildId>,
}

fn main() {
    let _ = Parent {
        required_child: Child { value: 1 },
        optional_child: None,
        required_id: ChildId { value: 1 },
        optional_id: None,
    }
    .validate();
}
