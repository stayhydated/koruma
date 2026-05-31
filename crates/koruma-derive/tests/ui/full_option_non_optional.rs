use koruma_derive::Koruma;

#[derive(Koruma)]
struct Demo {
    #[koruma(RequiredValidation::<Option<_>>)]
    value: Option<String>,
}

fn main() {}
