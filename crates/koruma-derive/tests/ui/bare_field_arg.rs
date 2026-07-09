use koruma_derive::Koruma;

#[derive(Koruma)]
struct Demo {
    other: String,
    #[koruma(MatchesValidation.expected(other))]
    value: String,
}

fn main() {}
