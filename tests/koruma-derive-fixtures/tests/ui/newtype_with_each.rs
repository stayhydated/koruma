use koruma_derive::Koruma;

#[derive(Koruma)]
struct Demo {
    #[koruma(newtype, each(RequiredValidation::<_>))]
    value: Vec<String>,
}

fn main() {}
