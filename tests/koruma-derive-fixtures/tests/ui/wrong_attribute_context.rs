use koruma_derive::Koruma;

#[derive(Koruma)]
struct Demo {
    #[koruma(value)]
    value: String,
    #[koruma(setter(required))]
    setter: String,
    #[koruma(try_new)]
    try_new: String,
}

fn main() {}
