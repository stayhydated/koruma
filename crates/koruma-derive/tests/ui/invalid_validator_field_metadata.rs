use koruma_derive::validator;

#[validator]
pub struct UnknownMarkerValidation {
    #[koruma(nested)]
    limit: usize,
    #[koruma(value)]
    actual: Option<usize>,
}

#[validator]
pub struct RequiredDefaultValidation {
    #[koruma(setter(required, default = 0))]
    limit: usize,
    #[koruma(value)]
    actual: Option<usize>,
}

#[validator]
pub struct ValueSetterValidation {
    #[koruma(value, setter(required))]
    actual: Option<usize>,
}

fn main() {}
