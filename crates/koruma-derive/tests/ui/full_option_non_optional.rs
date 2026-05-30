use koruma_derive::Koruma;

#[derive(Koruma)]
struct Demo {
    #[koruma(RequiredValidation::<Option<_>>)]
    value: String,
}

fn main() {}
