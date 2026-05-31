use koruma_derive::Koruma;

#[derive(Koruma)]
struct Demo {
    #[koruma(all = RequiredValidation)]
    value: String,
}

fn main() {}
