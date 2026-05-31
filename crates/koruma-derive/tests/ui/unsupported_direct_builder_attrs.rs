use koruma_derive::validator;

#[validator]
pub struct SkipValidation {
    #[builder(skip = 0)]
    limit: usize,
    #[koruma(value)]
    actual: Option<usize>,
}

#[validator]
pub struct FieldBackedValidation {
    #[builder(field(value: usize))]
    limit: usize,
    #[koruma(value)]
    actual: Option<usize>,
}

#[validator]
pub struct StartFnValidation {
    #[builder(start_fn = custom)]
    limit: usize,
    #[koruma(value)]
    actual: Option<usize>,
}

#[validator]
pub struct UnknownBuilderAttrValidation {
    #[builder(getter)]
    limit: usize,
    #[koruma(value)]
    actual: Option<usize>,
}

fn main() {}
