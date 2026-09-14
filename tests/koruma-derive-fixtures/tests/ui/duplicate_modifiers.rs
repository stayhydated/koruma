use koruma_derive::Koruma;

#[derive(Koruma)]
struct Demo {
    #[koruma(nested, nested)]
    value: String,
}

fn main() {}
