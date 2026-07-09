use koruma_derive::Koruma;

#[derive(Koruma)]
struct Demo {
    #[koruma(all = RequiredValidation, inner = RequiredValidation)]
    value: String,
}

fn main() {}
