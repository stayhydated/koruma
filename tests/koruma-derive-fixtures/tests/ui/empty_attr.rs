use koruma_derive::Koruma;

#[derive(Koruma)]
struct Demo {
    #[koruma()]
    value: String,
}

fn main() {}
