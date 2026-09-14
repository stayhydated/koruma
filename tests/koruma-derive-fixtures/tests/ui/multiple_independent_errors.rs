use koruma_derive::Koruma;

#[derive(Koruma)]
struct Demo {
    #[koruma(skip, nested)]
    first: String,
    #[koruma(newtype, each(RequiredValidation))]
    second: String,
}

fn main() {}
