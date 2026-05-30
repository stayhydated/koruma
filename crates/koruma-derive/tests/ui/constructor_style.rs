use koruma_derive::Koruma;

#[derive(Koruma)]
struct Demo {
    #[koruma(Validator(value = 1))]
    value: i32,
}

fn main() {}
