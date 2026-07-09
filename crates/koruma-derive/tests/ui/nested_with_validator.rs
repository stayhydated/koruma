use koruma_derive::Koruma;

#[derive(Koruma)]
struct Demo {
    #[koruma(nested, RequiredValidation::<_>)]
    value: String,
}

fn main() {}
