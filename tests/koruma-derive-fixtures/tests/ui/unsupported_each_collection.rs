use koruma_derive::Koruma;

#[derive(Koruma)]
struct Demo {
    #[koruma(each(RequiredValidation::<_>))]
    value: std::collections::HashMap<String, String>,
}

fn main() {}
