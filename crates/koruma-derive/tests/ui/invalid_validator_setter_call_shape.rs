use koruma_derive::Koruma;

#[derive(Koruma)]
struct Demo {
    #[koruma(RangeValidation::min())]
    missing_arg: i32,

    #[koruma(RangeValidation::min(0, 1))]
    too_many_args: i32,

    #[koruma(RangeValidation::min::<u8>(0))]
    method_generic: i32,

    #[koruma(each(RangeValidation::<_>::min::<u8>(0)))]
    element_method_generic: Vec<i32>,
}

fn main() {}
