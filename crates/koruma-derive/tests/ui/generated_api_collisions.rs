use koruma_derive::{Koruma, validator};

#[derive(Koruma)]
struct FieldNameCollision {
    #[koruma(other = RequiredValidation)]
    value: String,
    other: String,
}

#[derive(Koruma)]
struct ElementErrorsCollision {
    #[koruma(element_errors = RequiredValidation)]
    value: String,
}

#[validator]
pub struct BuildMethodValidation {
    #[koruma(setter(name = build))]
    min: usize,
    #[koruma(value)]
    actual: Option<usize>,
}

#[validator]
pub struct MaybeMethodValidation {
    maybe_min: i32,
    min: Option<i32>,
    #[koruma(value)]
    actual: Option<i32>,
}

#[validator]
pub struct StateGenericValidation<__KorumaMinState> {
    min: i32,
    #[koruma(value)]
    actual: Option<i32>,
}

fn main() {}
