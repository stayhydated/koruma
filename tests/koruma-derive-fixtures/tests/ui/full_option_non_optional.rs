use koruma_derive::Koruma;

#[derive(Koruma)]
struct Demo {
    #[koruma(GenericValidation::<Option<_>>)]
    value: String,
}

fn main() {}
