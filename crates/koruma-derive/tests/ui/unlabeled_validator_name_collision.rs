use koruma_derive::Koruma;

#[derive(Koruma)]
struct Demo {
    #[koruma(foo::Baz, bar::Baz)]
    value: String,
}

fn main() {}
